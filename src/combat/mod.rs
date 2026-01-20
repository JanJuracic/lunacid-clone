//! Combat module - weapons, attacks, blocking, and damage.

mod components;
mod plugin;
mod systems;
mod viewmodel;

pub use components::*;
pub use plugin::{create_starter_weapon, CombatPlugin};
pub use viewmodel::WeaponViewmodel;
