//! Core game module - states, events, and fundamental systems.
//!
//! This module provides the foundation that all other game systems build upon.

mod events;
mod plugin;
mod states;
mod tween;

pub use events::*;
pub use plugin::CorePlugin;
pub use states::*;
pub use tween::*;
