//! Player module - player entity, movement, and camera control.

mod components;
mod movement;
mod plugin;

pub use components::*;
pub use movement::{spawn_player, PlayerCamera, WeaponCamera};
pub use plugin::PlayerPlugin;
