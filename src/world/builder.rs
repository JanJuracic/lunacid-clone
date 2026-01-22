//! Level construction from data definitions.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;

use super::data::{GeometryKind, LevelDefinition, ResolvedCeilingTile, ResolvedGeometryTile, ResolvedMonsterSpawn};
use crate::combat::Health;
use crate::enemies::animation::NeedsAnimationSetup;
use crate::enemies::data::EnemyRegistry;
use crate::enemies::{AiState, AttackTimer, Enemy, EnemyType};

/// Marker for all level geometry that should be cleaned up.
#[derive(Component)]
pub struct LevelGeometry;

/// Material registry mapping material names to handles.
pub struct MaterialRegistry {
    materials: HashMap<String, Handle<StandardMaterial>>,
    ceilings: HashMap<String, Handle<StandardMaterial>>,
    pub pillar: Handle<StandardMaterial>,
}

impl MaterialRegistry {
    pub fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        let mut registry = HashMap::new();

        // Stone material (default)
        registry.insert(
            "stone".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.3, 0.35),
                perceptual_roughness: 0.9,
                ..default()
            }),
        );

        // Stone wall material
        registry.insert(
            "stone_wall".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.35, 0.3),
                perceptual_roughness: 0.8,
                ..default()
            }),
        );

        // Wood material
        registry.insert(
            "wood".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.45, 0.32, 0.2),
                perceptual_roughness: 0.7,
                ..default()
            }),
        );

        // Metal material
        registry.insert(
            "metal".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 0.55),
                perceptual_roughness: 0.3,
                metallic: 0.8,
                ..default()
            }),
        );

        let mut ceilings = HashMap::new();

        // Default ceiling material
        ceilings.insert(
            "ceiling".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.25, 0.25, 0.3),
                perceptual_roughness: 0.9,
                ..default()
            }),
        );

        // Stone ceiling material
        ceilings.insert(
            "stone_ceiling".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.35, 0.4),
                perceptual_roughness: 0.85,
                ..default()
            }),
        );

        // Wood ceiling material
        ceilings.insert(
            "wood_ceiling".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.28, 0.18),
                perceptual_roughness: 0.75,
                ..default()
            }),
        );

        // Skylight material (brighter, slightly emissive)
        ceilings.insert(
            "skylight".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.65, 0.7),
                perceptual_roughness: 0.5,
                emissive: LinearRgba::new(0.1, 0.12, 0.15, 1.0),
                ..default()
            }),
        );

        let pillar = materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.45, 0.4),
            perceptual_roughness: 0.7,
            ..default()
        });

        Self {
            materials: registry,
            ceilings,
            pillar,
        }
    }

    /// Get material for floor by name.
    pub fn get_floor(&self, material_name: &str) -> Handle<StandardMaterial> {
        self.materials
            .get(material_name)
            .cloned()
            .unwrap_or_else(|| {
                self.materials.get("stone").cloned().unwrap()
            })
    }

    /// Get material for walls by name.
    pub fn get_wall(&self, material_name: &str) -> Handle<StandardMaterial> {
        // Use _wall variant if available, else use base material
        let wall_name = format!("{}_wall", material_name);
        self.materials
            .get(&wall_name)
            .or_else(|| self.materials.get(material_name))
            .cloned()
            .unwrap_or_else(|| {
                self.materials.get("stone_wall").cloned().unwrap()
            })
    }

    /// Get material for ceilings by name.
    pub fn get_ceiling(&self, material_name: &str) -> Handle<StandardMaterial> {
        self.ceilings
            .get(material_name)
            .cloned()
            .unwrap_or_else(|| {
                self.ceilings.get("ceiling").cloned().unwrap()
            })
    }
}

/// Build a level from a level definition.
pub fn build_level_from_data(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level: &LevelDefinition,
    asset_server: &AssetServer,
    enemy_registry: &EnemyRegistry,
) -> Vec3 {
    let mat_registry = MaterialRegistry::new(materials);
    let tile_size = level.tile_size;
    let wall_thickness = 0.2;

    // Set up global ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(
            level.global_ambient.color.0,
            level.global_ambient.color.1,
            level.global_ambient.color.2,
        ),
        brightness: level.global_ambient.brightness,
    });

    // Set up directional light (moonlight from above)
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.6, 0.7, 0.9), // Pale blue-silver moonlight
            illuminance: 2000.0,               // Dim moonlight intensity
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

    // Set up sky sphere (gradient from horizon to zenith)
    let level_center_x = (level.width as f32 * tile_size) / 2.0;
    let level_center_z = (level.height as f32 * tile_size) / 2.0;
    spawn_sky_sphere(commands, meshes, materials, Vec3::new(level_center_x, 0.0, level_center_z));

    // Process each tile
    for z in 0..level.height as i32 {
        for x in 0..level.width as i32 {
            let geo_tile = level.get_geometry(x, z);
            let world_pos = level.grid_to_world(x, z);

            match geo_tile.kind {
                GeometryKind::Floor | GeometryKind::Doorway => {
                    spawn_floor_tile(
                        commands, meshes, &mat_registry,
                        world_pos, tile_size, geo_tile,
                    );

                    // Generate walls for floor tiles (not doorways)
                    if geo_tile.kind == GeometryKind::Floor {
                        spawn_walls_for_tile(
                            commands, meshes, &mat_registry, level, x, z, world_pos,
                            tile_size, wall_thickness,
                        );
                    }
                }
                GeometryKind::Pillar => {
                    spawn_floor_tile(
                        commands, meshes, &mat_registry,
                        world_pos, tile_size, geo_tile,
                    );

                    spawn_pillar(
                        commands, meshes, &mat_registry,
                        world_pos, tile_size, geo_tile.height,
                    );

                    spawn_walls_for_tile(
                        commands, meshes, &mat_registry, level, x, z, world_pos,
                        tile_size, wall_thickness,
                    );
                }
                GeometryKind::Wall => {
                    spawn_wall_cube(commands, meshes, &mat_registry, world_pos, tile_size, geo_tile);
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
                spawn_ceiling_tile(commands, meshes, &mat_registry, world_pos, tile_size, ceiling_tile);
            }
        }
    }

    // Spawn monsters from grid
    spawn_monsters_from_grid(
        commands,
        level,
        &level.monster_spawns,
        asset_server,
        enemy_registry,
    );

    // Return player spawn position
    let player_world_pos = level.grid_to_world(level.player_start.0, level.player_start.1);
    Vec3::new(player_world_pos.x, 1.0, player_world_pos.z)
}

/// Spawn a floor tile (without ceiling - ceiling is handled separately).
fn spawn_floor_tile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat_registry: &MaterialRegistry,
    world_pos: Vec3,
    tile_size: f32,
    geo_tile: &ResolvedGeometryTile,
) {
    let floor_material = mat_registry.get_floor(&geo_tile.material);
    let floor_depth = geo_tile.floor_depth;

    // Floor as a box extending downward
    // Top surface at y=0, bottom at y=-floor_depth
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(tile_size, floor_depth, tile_size))),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(world_pos.x, -floor_depth / 2.0, world_pos.z),
        Collider::cuboid(tile_size / 2.0, floor_depth / 2.0, tile_size / 2.0),
        LevelGeometry,
    ));
}

/// Spawn a ceiling tile at the specified position.
/// Bottom face is at ceiling_tile.height, thickness extends upward.
fn spawn_ceiling_tile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat_registry: &MaterialRegistry,
    world_pos: Vec3,
    tile_size: f32,
    ceiling_tile: &ResolvedCeilingTile,
) {
    // Ceiling as a box: bottom face at height, extends upward by thickness
    // Center is at height + thickness/2
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(tile_size, ceiling_tile.thickness, tile_size))),
        MeshMaterial3d(mat_registry.get_ceiling(&ceiling_tile.material)),
        Transform::from_xyz(
            world_pos.x,
            ceiling_tile.height + ceiling_tile.thickness / 2.0,
            world_pos.z,
        ),
        LevelGeometry,
    ));
}

/// Spawn walls around a floor tile based on neighbors.
fn spawn_walls_for_tile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat_registry: &MaterialRegistry,
    level: &LevelDefinition,
    x: i32,
    z: i32,
    world_pos: Vec3,
    tile_size: f32,
    wall_thickness: f32,
) {
    let current_tile = level.get_geometry(x, z);
    let wall_height = current_tile.height;
    let wall_material = mat_registry.get_wall(&current_tile.material);
    let half_tile = tile_size / 2.0;

    // North neighbor (z - 1)
    if needs_wall(level, x, z - 1) {
        spawn_wall(
            commands, meshes, wall_material.clone(),
            Vec3::new(world_pos.x, wall_height / 2.0, world_pos.z - half_tile),
            Vec3::new(tile_size, wall_height, wall_thickness),
        );
    }

    // South neighbor (z + 1)
    if needs_wall(level, x, z + 1) {
        spawn_wall(
            commands, meshes, wall_material.clone(),
            Vec3::new(world_pos.x, wall_height / 2.0, world_pos.z + half_tile),
            Vec3::new(tile_size, wall_height, wall_thickness),
        );
    }

    // West neighbor (x - 1)
    if needs_wall(level, x - 1, z) {
        spawn_wall(
            commands, meshes, wall_material.clone(),
            Vec3::new(world_pos.x - half_tile, wall_height / 2.0, world_pos.z),
            Vec3::new(wall_thickness, wall_height, tile_size),
        );
    }

    // East neighbor (x + 1)
    if needs_wall(level, x + 1, z) {
        spawn_wall(
            commands, meshes, wall_material,
            Vec3::new(world_pos.x + half_tile, wall_height / 2.0, world_pos.z),
            Vec3::new(wall_thickness, wall_height, tile_size),
        );
    }
}

/// Check if a wall is needed against the neighboring tile.
fn needs_wall(level: &LevelDefinition, x: i32, z: i32) -> bool {
    let neighbor = level.get_geometry(x, z);
    // Only need edge wall against Void (Wall tiles are now solid cubes)
    neighbor.kind == GeometryKind::Void
}

/// Spawn a wall segment.
fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
        LevelGeometry,
    ));
}

/// Spawn a solid wall cube filling the entire tile.
fn spawn_wall_cube(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat_registry: &MaterialRegistry,
    world_pos: Vec3,
    tile_size: f32,
    geo_tile: &ResolvedGeometryTile,
) {
    let wall_material = mat_registry.get_wall(&geo_tile.material);
    let wall_height = geo_tile.height;

    // Solid cube: bottom at y=0, top at y=wall_height
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(tile_size, wall_height, tile_size))),
        MeshMaterial3d(wall_material),
        Transform::from_xyz(world_pos.x, wall_height / 2.0, world_pos.z),
        Collider::cuboid(tile_size / 2.0, wall_height / 2.0, tile_size / 2.0),
        LevelGeometry,
    ));
}

/// Spawn a pillar.
fn spawn_pillar(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat_registry: &MaterialRegistry,
    world_pos: Vec3,
    tile_size: f32,
    wall_height: f32,
) {
    let pillar_size = tile_size * 0.4;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(pillar_size, wall_height, pillar_size))),
        MeshMaterial3d(mat_registry.pillar.clone()),
        Transform::from_xyz(world_pos.x, wall_height / 2.0, world_pos.z),
        Collider::cuboid(pillar_size / 2.0, wall_height / 2.0, pillar_size / 2.0),
        LevelGeometry,
    ));
}

/// Spawn a point light.
fn spawn_light(
    commands: &mut Commands,
    position: Vec3,
    intensity: f32,
    shadows: bool,
    color: (f32, f32, f32),
    range: f32,
) {
    commands.spawn((
        PointLight {
            color: Color::srgb(color.0, color.1, color.2),
            intensity,
            range,
            shadows_enabled: shadows,
            ..default()
        },
        Transform::from_translation(position),
        LevelGeometry,
    ));
}

/// Spawn monsters from the resolved monster grid.
fn spawn_monsters_from_grid(
    commands: &mut Commands,
    level: &LevelDefinition,
    monster_spawns: &[ResolvedMonsterSpawn],
    asset_server: &AssetServer,
    enemy_registry: &EnemyRegistry,
) {
    for spawn in monster_spawns {
        let Some(definition) = enemy_registry.get(&spawn.enemy_type) else {
            warn!("Unknown enemy type in monster grid: {}", spawn.enemy_type);
            continue;
        };

        let world_pos = level.grid_to_world(spawn.grid_pos.0, spawn.grid_pos.1);
        let spawn_pos = Vec3::new(world_pos.x, 0.0, world_pos.z);

        let collider_config = definition.collider.clone().unwrap_or_default();

        commands.spawn((
            Enemy,
            EnemyType(spawn.enemy_type.clone()),
            AiState::default(),
            definition.to_stats(),
            Health::new(definition.max_health),
            AttackTimer::default(),
            NeedsAnimationSetup,
            SceneRoot(asset_server.load(&definition.model_path)),
            Transform::from_translation(spawn_pos)
                .with_scale(Vec3::splat(definition.scale)),
            Collider::capsule_y(collider_config.half_height, collider_config.radius),
            RigidBody::KinematicPositionBased,
            LevelGeometry, // Mark as level geometry so enemies get cleaned up with the level
        ));

        info!("Spawned {} at grid ({}, {})", definition.name, spawn.grid_pos.0, spawn.grid_pos.1);
    }
}

/// Marker component for the sky sphere (doesn't cast shadows).
#[derive(Component)]
pub struct SkySphere;

/// Spawn a sky sphere for the background.
/// Uses an inverted sphere with an emissive unlit material for the night sky gradient.
fn spawn_sky_sphere(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: Vec3,
) {
    let sky_radius = 500.0;

    // Night sky material - dark blue with slight emission so it's visible
    // Unlit appearance achieved through high emissive, zero base color
    let sky_material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: LinearRgba::new(0.15, 0.12, 0.2, 1.0), // Purple-ish twilight
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
        // NotShadowCaster is applied via the unlit material - shadows won't be cast
    ));
}
