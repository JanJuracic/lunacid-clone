//! Player plugin - movement, camera, and player-related systems.

use bevy::prelude::*;

use super::components::*;
use super::movement;

/// Player plugin - handles player spawning, movement, and camera.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Set up movement systems
        movement::setup_movement_systems(app);

        // Initialize resources
        app.init_resource::<PlayerConfig>();
    }
}
