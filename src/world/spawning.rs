//! Entity spawning functions for level construction.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::builder::LevelGeometry;
use super::data::{LevelDefinition, ResolvedMonsterSpawn};
use crate::combat::Health;
use crate::enemies::animation::NeedsAnimationSetup;
use crate::enemies::data::EnemyRegistry;
use crate::enemies::{AiState, AttackTimer, Enemy, EnemyType};

/// Spawn a point light.
pub fn spawn_light(
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
pub fn spawn_monsters_from_grid(
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
