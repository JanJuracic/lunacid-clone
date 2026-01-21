//! Enemy-related components.

use bevy::prelude::*;

/// Marker component for all enemies.
#[derive(Component)]
pub struct Enemy;

/// Enemy type identifier (matches RON file name).
#[derive(Component, Clone)]
pub struct EnemyType(pub String);

/// AI state machine for enemy behavior.
#[derive(Component, Default, PartialEq, Clone, Debug)]
pub enum AiState {
    /// Standing still, waiting for player to enter detection range.
    #[default]
    Idle,
    /// Moving toward the player.
    Chasing,
    /// Performing an attack.
    Attacking,
    /// Playing death animation before despawn.
    Dying,
}

/// Enemy stats loaded from RON data files.
#[derive(Component, Clone)]
pub struct EnemyStats {
    pub max_health: f32,
    pub damage: f32,
    pub move_speed: f32,
    pub detection_range: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
}

impl Default for EnemyStats {
    fn default() -> Self {
        Self {
            max_health: 50.0,
            damage: 10.0,
            move_speed: 3.0,
            detection_range: 8.0,
            attack_range: 2.0,
            attack_cooldown: 1.5,
        }
    }
}

/// Timer for attack cooldown between enemy attacks.
#[derive(Component)]
pub struct AttackTimer(pub Timer);

impl Default for AttackTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.5, TimerMode::Once))
    }
}

/// Timer for death animation before despawn.
#[derive(Component)]
pub struct DeathTimer(pub Timer);

impl Default for DeathTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Once))
    }
}

/// Marker component to signal that an enemy is ready to attack.
/// Added by AI when attack timer finishes, removed by animation system after triggering.
#[derive(Component)]
pub struct AttackReady;
