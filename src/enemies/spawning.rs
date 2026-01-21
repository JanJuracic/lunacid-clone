//! Enemy spawning system.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use super::animation::NeedsAnimationSetup;
use super::components::{AiState, AttackTimer, Enemy, EnemyType};
use super::data::EnemyRegistry;
use crate::combat::Health;

/// Minimum distance between spawned enemies to prevent overlap.
const MIN_SPAWN_SEPARATION: f32 = 2.0;
/// Maximum attempts to find a valid spawn position before giving up.
const MAX_SPAWN_ATTEMPTS: usize = 10;

/// Spawn zone component that defines where enemies spawn.
#[derive(Component)]
pub struct SpawnZone {
    /// Type of enemy to spawn (matches RON file name).
    pub enemy_type: String,
    /// Half extents of the spawn area (x, y, z).
    pub half_extents: Vec3,
    /// Maximum number of enemies from this zone.
    pub max_enemies: usize,
    /// Initial delay before spawning (0.0 for immediate).
    pub spawn_delay: f32,
}

/// Spawn enemies within spawn zones (runs once at level start).
pub fn spawn_enemies_in_zones(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<EnemyRegistry>,
    zone_query: Query<(&Transform, &SpawnZone)>,
) {
    let mut rng = rand::thread_rng();

    for (zone_transform, zone) in zone_query.iter() {
        let Some(definition) = registry.get(&zone.enemy_type) else {
            warn!("Unknown enemy type: {}", zone.enemy_type);
            continue;
        };

        // Track spawned positions within this zone to ensure separation
        let mut spawned_positions: Vec<Vec3> = Vec::new();

        for _ in 0..zone.max_enemies {
            // Try to find a valid spawn position with separation from existing spawns
            let mut spawn_pos = None;

            for _ in 0..MAX_SPAWN_ATTEMPTS {
                let offset = Vec3::new(
                    rng.gen_range(-zone.half_extents.x..zone.half_extents.x),
                    0.0,
                    rng.gen_range(-zone.half_extents.z..zone.half_extents.z),
                );
                let candidate_pos = zone_transform.translation + offset;

                // Check if this position has enough separation from all existing spawns
                let has_separation = spawned_positions.iter().all(|existing| {
                    candidate_pos.distance(*existing) >= MIN_SPAWN_SEPARATION
                });

                if has_separation {
                    spawn_pos = Some(candidate_pos);
                    break;
                }
            }

            // If no valid position found after max attempts, skip this spawn
            let Some(spawn_pos) = spawn_pos else {
                warn!(
                    "Could not find valid spawn position for {} after {} attempts",
                    definition.name, MAX_SPAWN_ATTEMPTS
                );
                continue;
            };

            // Track this position for future separation checks
            spawned_positions.push(spawn_pos);

            let collider_config = definition.collider.clone().unwrap_or_default();

            commands.spawn((
                Enemy,
                EnemyType(zone.enemy_type.clone()),
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
            ));

            info!("Spawned {} at {:?}", definition.name, spawn_pos);
        }
    }
}
