//! Level construction from data definitions.

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;

use super::data::{GeometryKind, LevelDefinition};
use super::geometry::{
    spawn_ceiling_tile, spawn_floor_tile, spawn_pillar, spawn_wall_cube, spawn_walls_for_tile,
};
use super::materials::MaterialRegistry;
use super::prefabs::spawn_prefab;
use super::spawning::{spawn_light, spawn_monsters_from_grid};
use crate::enemies::data::EnemyRegistry;
use crate::rendering::VisualConfig;

/// Marker for all level geometry that should be cleaned up.
#[derive(Component)]
pub struct LevelGeometry;

/// Marker component for the sky sphere (doesn't cast shadows).
#[derive(Component)]
pub struct SkySphere;

/// Build a level from a level definition.
pub fn build_level_from_data(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level: &LevelDefinition,
    asset_server: &AssetServer,
    enemy_registry: &EnemyRegistry,
    visual_config: &VisualConfig,
) -> Vec3 {
    let mat_registry = MaterialRegistry::new(materials);
    let tile_size = level.tile_size;
    let wall_thickness = 0.2;

    // Set up environment
    setup_environment(commands, level, visual_config);

    // Set up sky sphere
    let level_center_x = (level.width as f32 * tile_size) / 2.0;
    let level_center_z = (level.height as f32 * tile_size) / 2.0;
    spawn_sky_sphere(
        commands,
        meshes,
        materials,
        Vec3::new(level_center_x, 0.0, level_center_z),
        visual_config.sky_color,
    );

    // Build geometry and ambient elements
    build_geometry(
        commands,
        meshes,
        &mat_registry,
        level,
        tile_size,
        wall_thickness,
    );

    // Spawn entities
    spawn_entities(
        commands,
        meshes,
        &mat_registry,
        level,
        tile_size,
        asset_server,
        enemy_registry,
    );

    // Return player spawn position
    let player_world_pos = level.grid_to_world(level.player_start.0, level.player_start.1);
    Vec3::new(player_world_pos.x, 1.0, player_world_pos.z)
}

/// Set up global ambient light and directional light.
fn setup_environment(commands: &mut Commands, level: &LevelDefinition, _visual_config: &VisualConfig) {
    // Set up global ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(
            level.global_ambient.color.0,
            level.global_ambient.color.1,
            level.global_ambient.color.2,
        ),
        brightness: level.global_ambient.brightness,
    });

    // Set up directional light (moonlight)
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.7, 0.7, 0.75), // Desaturated pale grey-blue
            illuminance: 2000.0,                 // Subtle moonlight level
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_3, // ~60 degrees down from horizontal
            std::f32::consts::FRAC_PI_6,  // Slight angle for more interesting shadows
            0.0,
        )),
        LevelGeometry,
    ));
}

/// Build all geometry tiles (floors, walls, ceilings) and process ambient elements.
fn build_geometry(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat_registry: &MaterialRegistry,
    level: &LevelDefinition,
    tile_size: f32,
    wall_thickness: f32,
) {
    for z in 0..level.height as i32 {
        for x in 0..level.width as i32 {
            let geo_tile = level.get_geometry(x, z);
            let world_pos = level.grid_to_world(x, z);

            match geo_tile.kind {
                GeometryKind::Floor | GeometryKind::Doorway => {
                    spawn_floor_tile(commands, meshes, mat_registry, world_pos, tile_size, geo_tile);

                    // Generate walls for floor tiles (not doorways)
                    if geo_tile.kind == GeometryKind::Floor {
                        spawn_walls_for_tile(
                            commands,
                            meshes,
                            mat_registry,
                            level,
                            x,
                            z,
                            world_pos,
                            tile_size,
                            wall_thickness,
                        );
                    }
                }
                GeometryKind::Pillar => {
                    spawn_floor_tile(commands, meshes, mat_registry, world_pos, tile_size, geo_tile);
                    spawn_pillar(commands, meshes, mat_registry, world_pos, tile_size, geo_tile.height);
                    spawn_walls_for_tile(
                        commands,
                        meshes,
                        mat_registry,
                        level,
                        x,
                        z,
                        world_pos,
                        tile_size,
                        wall_thickness,
                    );
                }
                GeometryKind::Wall => {
                    spawn_wall_cube(commands, meshes, mat_registry, world_pos, tile_size, geo_tile);
                }
                GeometryKind::Void => {
                    // Nothing to spawn
                }
            }

            // Process ambient tile at this position
            let ambient_tile = level.get_ambient(x, z);

            // Spawn lights
            for light_def in &ambient_tile.lights {
                spawn_light(
                    commands,
                    world_pos + Vec3::new(0.0, light_def.height, 0.0),
                    light_def.intensity,
                    light_def.shadows,
                    light_def.color,
                    light_def.range,
                );
            }

            // Log placeholder warnings for particles
            for particle_def in &ambient_tile.particles {
                warn!(
                    "Particle system '{}' at ({}, {}) not yet implemented",
                    particle_def.kind, x, z
                );
            }

            // Log placeholder warnings for audio
            for audio_def in &ambient_tile.audio {
                warn!(
                    "Audio zone '{}' at ({}, {}) not yet implemented",
                    audio_def.sound, x, z
                );
            }

            // Spawn ceiling tile if present (None means open sky/void)
            if let Some(ceiling_tile) = level.get_ceiling(x, z) {
                spawn_ceiling_tile(commands, meshes, mat_registry, world_pos, tile_size, ceiling_tile);
            }
        }
    }
}

/// Spawn monsters and prefabs.
fn spawn_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat_registry: &MaterialRegistry,
    level: &LevelDefinition,
    tile_size: f32,
    asset_server: &AssetServer,
    enemy_registry: &EnemyRegistry,
) {
    // Spawn monsters from grid
    spawn_monsters_from_grid(
        commands,
        level,
        &level.monster_spawns,
        asset_server,
        enemy_registry,
    );

    // Spawn prefabs (stairs, etc.)
    let stair_material = mat_registry.get_floor("stone");
    for prefab in &level.prefabs {
        spawn_prefab(commands, meshes, prefab, tile_size, stair_material.clone());
    }
}

/// Spawn a sky sphere for the background.
/// Uses an inverted sphere with an emissive unlit material for the night sky gradient.
fn spawn_sky_sphere(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: Vec3,
    sky_color: (f32, f32, f32),
) {
    let sky_radius = 500.0;

    // Horror sky material - color from config for seamless blend with fog
    // Unlit appearance achieved through high emissive, zero base color
    let sky_material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: LinearRgba::new(sky_color.0, sky_color.1, sky_color.2, 1.0),
        unlit: true,
        cull_mode: None, // Render both sides so inside of sphere is visible
        ..default()
    });

    // Create inverted sphere mesh (normals pointing inward)
    let mut sky_mesh = Sphere::new(sky_radius).mesh().build();
    // Flip normals by negating them
    if let Some(normals) = sky_mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        if let bevy::render::mesh::VertexAttributeValues::Float32x3(ref mut values) = normals {
            for normal in values.iter_mut() {
                normal[0] = -normal[0];
                normal[1] = -normal[1];
                normal[2] = -normal[2];
            }
        }
    }
    // Flip triangle winding order
    if let Some(indices) = sky_mesh.indices_mut() {
        match indices {
            bevy::render::mesh::Indices::U16(ref mut idx) => {
                for chunk in idx.chunks_exact_mut(3) {
                    chunk.swap(1, 2);
                }
            }
            bevy::render::mesh::Indices::U32(ref mut idx) => {
                for chunk in idx.chunks_exact_mut(3) {
                    chunk.swap(1, 2);
                }
            }
        }
    }

    commands.spawn((
        Mesh3d(meshes.add(sky_mesh)),
        MeshMaterial3d(sky_material),
        Transform::from_translation(center),
        SkySphere,
        LevelGeometry,
        NotShadowCaster, // Prevent sky sphere from blocking directional light shadows
    ));
}
