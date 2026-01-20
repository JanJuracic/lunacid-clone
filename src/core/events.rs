//! Global events used for cross-system communication.
//!
//! Events allow decoupled systems to communicate. For example, the combat
//! system sends DamageEvents, and the health system receives them to
//! apply damage. This keeps systems independent and testable.

use bevy::prelude::*;

/// Element types for damage calculation.
///
/// Each element has strengths and weaknesses against others.
/// For example, Fire is strong against Ice enemies but weak against Water.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Element {
    #[default]
    Physical,
    Fire,
    Ice,
    Lightning,
    Poison,
    Holy,
    Dark,
}

/// Sent when an entity takes damage.
///
/// The damage system listens for these events and applies the actual
/// health reduction, taking resistances into account.
#[derive(Event)]
pub struct DamageEvent {
    /// Entity receiving damage
    pub target: Entity,
    /// Entity that caused the damage
    pub source: Entity,
    /// Base damage amount before resistances
    pub amount: f32,
    /// Element type for resistance calculation
    pub element: Element,
    /// Knockback direction and force
    pub knockback: Vec3,
}

/// Sent when an entity dies (health reaches 0).
///
/// Systems can listen for this to trigger death animations,
/// spawn loot, award XP, etc.
#[derive(Event)]
pub struct DeathEvent {
    /// Entity that died
    pub entity: Entity,
    /// Entity that killed them (if any)
    pub killed_by: Option<Entity>,
}

/// Sent when the player picks up an item.
#[derive(Event)]
pub struct ItemPickupEvent {
    /// The item entity being picked up
    pub item: Entity,
    /// The player entity
    pub player: Entity,
}

/// Sent when the player levels up.
#[derive(Event)]
pub struct LevelUpEvent {
    /// The player entity
    pub player: Entity,
    /// New level
    pub new_level: u32,
}
