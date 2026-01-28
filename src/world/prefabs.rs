//! Prefab spawning for complex structures like stairs.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::builder::LevelGeometry;
use super::data::{PrefabInstance, PrefabKind};

/// Spawn a prefab instance.
pub fn spawn_prefab(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    prefab: &PrefabInstance,
    tile_size: f32,
    stair_material: Handle<StandardMaterial>,
) {
    match prefab.kind {
        PrefabKind::StepStairs => spawn_step_stairs(
            commands, meshes, prefab, tile_size, stair_material
        ),
    }
}

/// Spawn step stairs (cube steps that work with autostep).
fn spawn_step_stairs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    prefab: &PrefabInstance,
    tile_size: f32,
    material: Handle<StandardMaterial>,
) {
    let height_diff = prefab.to_elevation - prefab.from_elevation;
    let length_tiles = prefab.length.unwrap_or(1) as f32;
    let total_length = length_tiles * tile_size;

    // Step height must be <= autostep max_height (0.4)
    let step_height = 0.35;
    let num_steps = (height_diff / step_height).ceil() as i32;

    if num_steps <= 0 {
        warn!("StepStairs prefab has no height difference, skipping");
        return;
    }

    let actual_step_height = height_diff / num_steps as f32;
    let step_depth = total_length / num_steps as f32;

    // Grid to world position (center of the starting tile)
    let base_x = prefab.position.0 as f32 * tile_size + tile_size / 2.0;
    let base_z = prefab.position.1 as f32 * tile_size + tile_size / 2.0;

    // Rotation quaternion
    let rotation = Quat::from_rotation_y(prefab.rotation.to_radians());

    for i in 0..num_steps {
        // Calculate local offset along the stair direction
        // Steps go in +Z direction (forward) relative to rotation
        let local_offset = Vec3::new(0.0, 0.0, (i as f32 + 0.5) * step_depth - total_length / 2.0);
        let rotated_offset = rotation * local_offset;

        // Each step's Y position: bottom of step i is at from_elevation + i * actual_step_height
        // Center of step is at that + actual_step_height / 2
        let step_y = prefab.from_elevation + (i as f32 + 0.5) * actual_step_height;

        let step_pos = Vec3::new(
            base_x + rotated_offset.x,
            step_y,
            base_z + rotated_offset.z,
        );

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(tile_size, actual_step_height, step_depth))),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(step_pos).with_rotation(rotation),
            Collider::cuboid(tile_size / 2.0, actual_step_height / 2.0, step_depth / 2.0),
            LevelGeometry,
        ));
    }

    info!(
        "Spawned {} step stairs from elevation {} to {} at ({}, {})",
        num_steps, prefab.from_elevation, prefab.to_elevation,
        prefab.position.0, prefab.position.1
    );
}
