//! Enemies module - enemy entities, AI, and spawning.

pub mod animation;
mod ai;
mod components;
pub mod data;
mod plugin;
mod spawning;

pub use animation::AttackHitEvent;
pub use components::*;
pub use data::EnemyRegistry;
pub use plugin::EnemyPlugin;
// SpawnZone is deprecated - use monster grid in level files instead
