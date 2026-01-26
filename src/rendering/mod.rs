//! Rendering module - horror visual effects.

mod plugin;
mod post_process;
pub mod visual_config;

pub use plugin::{RenderConfig, RenderingPlugin};
pub use post_process::{HorrorPostProcessPlugin, PostProcessSettings};
pub use visual_config::VisualConfig;
