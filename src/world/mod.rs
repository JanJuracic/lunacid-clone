//! World module - levels, environments, and interactables.

mod builder;
mod data;
mod plugin;

pub use builder::LevelGeometry;
pub use data::{CurrentLevel, GeometryKind, LevelDefinition, LevelRegistry, PaletteRegistry};
pub use plugin::{setup_level, WorldPlugin};
