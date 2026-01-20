//! Lunacid Clone - Entry Point
//!
//! A PS1-style first-person dungeon crawler inspired by Lunacid and King's Field.
//!
//! Controls:
//! - WASD: Move
//! - Mouse: Look around
//! - Shift: Sprint
//! - Escape: Pause/Unpause

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lunacid Clone".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))

        // Physics
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())

        // Our game plugin
        .add_plugins(lunacid_clone::LunacidPlugin)

        .run();
}
