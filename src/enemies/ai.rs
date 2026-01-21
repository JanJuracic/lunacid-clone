//! Enemy AI behavior systems.

use bevy::prelude::*;

use super::components::{AiState, AttackReady, AttackTimer, DeathTimer, Enemy, EnemyStats};
use crate::combat::Health;
use crate::player::Player;

/// Detect player and transition from Idle to Chasing.
pub fn ai_detection(
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&Transform, &EnemyStats, &mut AiState), (With<Enemy>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for (enemy_transform, stats, mut ai_state) in enemy_query.iter_mut() {
        // Only check detection when idle
        if *ai_state != AiState::Idle {
            continue;
        }

        // Use horizontal distance (consistent with ai_chase)
        let player_pos = player_transform.translation;
        let enemy_pos = enemy_transform.translation;
        let horizontal_distance = Vec3::new(
            player_pos.x - enemy_pos.x,
            0.0,
            player_pos.z - enemy_pos.z,
        ).length();

        if horizontal_distance <= stats.detection_range {
            *ai_state = AiState::Chasing;
        }
    }
}

/// Chase player and transition to Attacking when in range.
pub fn ai_chase(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&mut Transform, &EnemyStats, &mut AiState), (With<Enemy>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for (mut enemy_transform, stats, mut ai_state) in enemy_query.iter_mut() {
        // Only move when chasing
        if *ai_state != AiState::Chasing {
            continue;
        }

        let player_pos = player_transform.translation;
        let enemy_pos = enemy_transform.translation;

        // Calculate direction to player (horizontal only)
        let direction = Vec3::new(
            player_pos.x - enemy_pos.x,
            0.0,
            player_pos.z - enemy_pos.z,
        );

        let distance = direction.length();

        // Check if in attack range
        if distance <= stats.attack_range {
            *ai_state = AiState::Attacking;
            continue;
        }

        // Check if player escaped detection range (with some buffer)
        if distance > stats.detection_range * 1.5 {
            *ai_state = AiState::Idle;
            continue;
        }

        // Move toward player
        if distance > 0.1 {
            let move_direction = direction.normalize();
            let movement = move_direction * stats.move_speed * time.delta_secs();
            enemy_transform.translation += movement;

            // Face the player (rotate around Y axis)
            let look_target = Vec3::new(player_pos.x, enemy_transform.translation.y, player_pos.z);
            enemy_transform.look_at(look_target, Vec3::Y);
            // Rotate 180° because model's forward is +Z, not -Z
            enemy_transform.rotate_y(std::f32::consts::PI);
        }
    }
}

/// Handle attack state and cooldown.
pub fn ai_attack(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<
        (Entity, &mut Transform, &EnemyStats, &mut AiState, &mut AttackTimer),
        (With<Enemy>, Without<Player>, Without<AttackReady>),
    >,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for (entity, mut enemy_transform, stats, mut ai_state, mut attack_timer) in enemy_query.iter_mut() {
        // Only process when attacking
        if *ai_state != AiState::Attacking {
            continue;
        }

        // Face the player (rotate around Y axis)
        let player_pos = player_transform.translation;
        let look_target = Vec3::new(player_pos.x, enemy_transform.translation.y, player_pos.z);
        enemy_transform.look_at(look_target, Vec3::Y);
        // Rotate 180° because model's forward is +Z, not -Z
        enemy_transform.rotate_y(std::f32::consts::PI);

        // Tick the attack timer
        attack_timer.0.tick(time.delta());

        // When attack timer finishes, signal ready to attack
        if attack_timer.0.finished() {
            // Add AttackReady marker before resetting timer
            commands.entity(entity).insert(AttackReady);

            // Reset timer for next attack
            attack_timer
                .0
                .set_duration(std::time::Duration::from_secs_f32(stats.attack_cooldown));
            attack_timer.0.reset();

            // Use horizontal distance (consistent with ai_chase)
            let player_pos = player_transform.translation;
            let enemy_pos = enemy_transform.translation;
            let horizontal_distance = Vec3::new(
                player_pos.x - enemy_pos.x,
                0.0,
                player_pos.z - enemy_pos.z,
            ).length();

            // If player moved out of attack range, go back to chasing
            if horizontal_distance > stats.attack_range {
                *ai_state = AiState::Chasing;
            }
            // Otherwise stay in Attacking state (attack again)
        }
    }
}

/// Handle enemy death transition.
pub fn handle_enemy_death(
    mut commands: Commands,
    mut enemy_query: Query<
        (Entity, &Health, &mut AiState),
        (With<Enemy>, Without<DeathTimer>),
    >,
) {
    for (entity, health, mut ai_state) in enemy_query.iter_mut() {
        if health.is_dead() && *ai_state != AiState::Dying {
            *ai_state = AiState::Dying;
            commands.entity(entity).insert(DeathTimer::default());
        }
    }
}

/// Despawn enemies after death animation completes.
pub fn despawn_dead_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DeathTimer)>,
) {
    for (entity, mut death_timer) in query.iter_mut() {
        death_timer.0.tick(time.delta());

        // Wait for death animation to complete before despawning
        if death_timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
