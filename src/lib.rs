//! Lunacid Clone - A PS1-style first-person dungeon crawler in Bevy.
//!
//! This is a recreation of Lunacid, a dark fantasy ARPG inspired by King's Field.
//!
//! # Architecture
//!
//! The game is organized into plugins, each handling a specific aspect:
//!
//! - **Core**: Game states, global events, fundamental systems
//! - **Player**: First-person movement, camera, player stats
//! - **Combat**: Melee attacks, blocking, damage calculation
//! - **Magic**: Spells, mana, projectiles
//! - **Inventory**: Items, equipment, pickups
//! - **Progression**: XP, leveling, attributes
//! - **World**: Levels, interactables, triggers
//! - **Rendering**: PSX-style visual effects
//! - **Audio**: Sound management
//! - **UI**: Menus, HUD, inventory screen
//! - **Persistence**: Save/load system

pub mod combat;
pub mod core;
pub mod enemies;
pub mod player;
pub mod rendering;
pub mod ui;
pub mod world;

// These modules will be implemented in later phases:
// pub mod magic;
// pub mod inventory;
// pub mod progression;
// pub mod audio;
// pub mod persistence;

use bevy::prelude::*;

/// Main game plugin that adds all sub-plugins.
pub struct LunacidPlugin;

impl Plugin for LunacidPlugin {
    fn build(&self, app: &mut App) {
        app
            // Core systems (must be first)
            .add_plugins(core::CorePlugin)

            // Player systems
            .add_plugins(player::PlayerPlugin)

            // Combat systems
            .add_plugins(combat::CombatPlugin)

            // Enemy systems
            .add_plugins(enemies::EnemyPlugin)

            // World systems
            .add_plugins(world::WorldPlugin)

            // Rendering systems
            .add_plugins(rendering::RenderingPlugin)

            // UI systems
            .add_plugins(ui::UiPlugin);
    }
}
