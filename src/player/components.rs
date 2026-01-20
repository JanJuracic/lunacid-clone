//! Player-related components.

use bevy::prelude::*;

/// Marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// Player's core statistics.
#[derive(Component)]
pub struct PlayerStats {
    pub max_health: f32,
    pub current_health: f32,
    pub max_mana: f32,
    pub current_mana: f32,
    pub mana_regen_rate: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_health: 100.0,
            current_health: 100.0,
            max_mana: 50.0,
            current_mana: 50.0,
            mana_regen_rate: 1.0,
        }
    }
}

/// Character attributes that affect gameplay.
#[derive(Component)]
pub struct Attributes {
    /// Affects melee damage
    pub strength: u32,
    /// Affects spell damage and mana pool
    pub magic: u32,
    /// Affects attack speed
    pub dexterity: u32,
    /// Affects movement speed
    pub speed: u32,
    /// Affects damage reduction
    pub defense: u32,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            strength: 10,
            magic: 10,
            dexterity: 10,
            speed: 10,
            defense: 10,
        }
    }
}

/// Tracks player movement state for physics.
#[derive(Component)]
pub struct MovementState {
    pub is_grounded: bool,
    pub vertical_velocity: f32,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            is_grounded: true,
            vertical_velocity: 0.0,
        }
    }
}

/// Configuration for the first-person camera controller.
#[derive(Resource)]
pub struct PlayerConfig {
    /// Mouse sensitivity multiplier
    pub mouse_sensitivity: f32,
    /// Invert Y-axis for mouse look
    pub invert_y: bool,
    /// Base movement speed in units per second
    pub move_speed: f32,
    /// Sprint speed multiplier
    pub sprint_multiplier: f32,
    /// Jump velocity
    pub jump_force: f32,
    /// Gravity acceleration
    pub gravity: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.5,
            invert_y: false,
            move_speed: 5.0,
            sprint_multiplier: 1.5,
            jump_force: 6.0,
            gravity: 15.0,
        }
    }
}
