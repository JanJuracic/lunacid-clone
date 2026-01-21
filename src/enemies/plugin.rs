//! Enemy plugin - registers all enemy systems.

use bevy::prelude::*;

use super::ai;
use super::animation;
use super::data::{load_enemy_definitions, EnemyRegistry};
use super::spawning::spawn_enemies_in_zones;
use crate::core::GameState;
use crate::world::setup_dungeon;

/// Enemy plugin - handles enemy spawning, AI, death, and animations.
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<EnemyRegistry>()
            // Register animation events
            .add_event::<animation::AttackHitEvent>()
            // Load definitions and spawn enemies once when entering the game
            // Must run after setup_dungeon so spawn zones exist
            .add_systems(
                OnEnter(GameState::InGame),
                (load_enemy_definitions, spawn_enemies_in_zones)
                    .chain()
                    .after(setup_dungeon),
            )
            // AI systems run during gameplay
            .add_systems(
                Update,
                (
                    ai::ai_detection,
                    ai::ai_chase,
                    ai::ai_attack,
                    ai::handle_enemy_death,
                    ai::despawn_dead_enemies,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            )
            // Animation systems run after AI systems
            .add_systems(
                Update,
                (
                    animation::setup_enemy_animations,
                    animation::sync_animation_state,
                    animation::trigger_attack_animation,
                    animation::trigger_hurt_animation,
                    animation::trigger_death_animation,
                    animation::play_animations,
                    animation::update_previous_animation_state,
                    animation::update_oneshot_timers,
                    animation::detect_attack_hit,
                )
                    .chain()
                    .after(ai::ai_attack)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}
