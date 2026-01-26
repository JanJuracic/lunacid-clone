//! Rendering plugin - horror visual effects.
//!
//! This module includes:
//! - Atmospheric distance fog
//! - Film grain post-processing
//! - CRT scanlines
//! - Vignette effect
//!
//! All effects configurable via assets/data/rendering/visual_config.ron.

use bevy::prelude::*;

use super::post_process::HorrorPostProcessPlugin;
use super::visual_config::{load_visual_config, VisualConfig};

/// Rendering plugin - configures horror-style visuals.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        // Load visual config from RON file
        let visual_config = VisualConfig::load();
        app.insert_resource(visual_config);
        app.insert_resource(RenderConfig::default());
        // Load visual config system (for potential hot-reloading in future)
        app.add_systems(Startup, load_visual_config);
        // Add horror post-processing effects
        app.add_plugins(HorrorPostProcessPlugin);
    }
}

/// Configuration for rendering effects.
#[derive(Resource)]
pub struct RenderConfig {
    /// Resolution scale (1.0 = native, lower = more pixelated)
    pub resolution_scale: f32,
    /// Vertex jitter intensity (0.0 = none, 1.0 = full PS1 wobble)
    pub vertex_jitter: f32,
    /// Enable fog for atmosphere
    pub fog_enabled: bool,
    /// Fog density (exponential squared)
    pub fog_density: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            resolution_scale: 1.0,
            vertex_jitter: 0.0,
            fog_enabled: true,
            fog_density: 0.025,
        }
    }
}
