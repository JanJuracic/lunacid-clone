//! Geometry spawning functions for level construction.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::builder::LevelGeometry;
use super::data::{GeometryKind, LevelDefinition, ResolvedCeilingTile, ResolvedGeometryTile};
use super::materials::MaterialRegistry;

/// Spawn a floor tile (without ceiling - ceiling is handled separately).
pub fn spawn_floor_tile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat_registry: &MaterialRegistry,
    world_pos: Vec3,
    tile_size: f32,
    geo_tile: &ResolvedGeometryTile,
) {
    let floor_material = mat_registry.get_floor(&geo_tile.material);
    let floor_depth = geo_tile.floor_depth;
    let floor_y = geo_tile.elevation;

    // Floor as a box extending downward from elevation
    // Top surface at y=elevation, bottom at y=elevation-floor_depth
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(tile_size, floor_depth, tile_size))),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(world_pos.x, floor_y - floor_depth / 2.0, world_pos.z),
        Collider::cuboid(tile_size / 2.0, floor_depth / 2.0, tile_size / 2.0),
        LevelGeometry,
    ));
}

/// Spawn a ceiling tile at the specified position.
/// Bottom face is at ceiling_tile.height, thickness extends upward.
pub fn spawn_ceiling_tile(
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
pub fn spawn_walls_for_tile(
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

    // Wall directions: (neighbor offset, position offset, dimensions)
    let wall_configs = [
        // North (z - 1)
        (
            (0, -1),
            Vec3::new(world_pos.x, wall_height / 2.0, world_pos.z - half_tile),
            Vec3::new(tile_size, wall_height, wall_thickness),
        ),
        // South (z + 1)
        (
            (0, 1),
            Vec3::new(world_pos.x, wall_height / 2.0, world_pos.z + half_tile),
            Vec3::new(tile_size, wall_height, wall_thickness),
        ),
        // West (x - 1)
        (
            (-1, 0),
            Vec3::new(world_pos.x - half_tile, wall_height / 2.0, world_pos.z),
            Vec3::new(wall_thickness, wall_height, tile_size),
        ),
        // East (x + 1)
        (
            (1, 0),
            Vec3::new(world_pos.x + half_tile, wall_height / 2.0, world_pos.z),
            Vec3::new(wall_thickness, wall_height, tile_size),
        ),
    ];

    for ((dx, dz), position, dimensions) in wall_configs {
        if needs_wall(level, x + dx, z + dz) {
            spawn_wall(commands, meshes, wall_material.clone(), position, dimensions);
        }
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
pub fn spawn_wall_cube(
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
pub fn spawn_pillar(
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
