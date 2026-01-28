//! World module - levels, environments, and interactables.

mod builder;
mod data;
mod error;
mod geometry;
mod materials;
mod prefabs;
mod plugin;
mod spawning;

pub use builder::LevelGeometry;
pub use data::{CurrentLevel, GeometryKind, LevelDefinition, LevelRegistry, PaletteRegistry, PrefabInstance, PrefabKind};
pub use error::DataLoadError;
pub use plugin::{setup_level, WorldPlugin};
