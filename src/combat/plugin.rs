//! Combat plugin - weapon combat, blocking, and damage.

use bevy::prelude::*;

use super::components::*;
use super::systems;
use super::viewmodel;

/// Combat plugin - handles all combat systems.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // Setup combat systems
        systems::setup_combat_systems(app);

        // Setup viewmodel systems
        viewmodel::setup_viewmodel_systems(app);
    }
}

/// Create a default starter weapon.
pub fn create_starter_weapon() -> Weapon {
    Weapon {
        name: "Short Sword".to_string(),
        base_damage: 15.0,
        element: Element::Physical,
        reach: 2.5,
        block_efficiency: 0.5,
        stamina_cost: 0.6,
        attack_cooldown: 0.4,
        model_path: "models/weapons/Sword.glb#Scene0".to_string(),
    }
}
