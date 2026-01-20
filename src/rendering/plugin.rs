//! Rendering plugin - PSX-style visual effects.
//!
//! This module will eventually include:
//! - Vertex jitter (snapping vertices to a lower-resolution grid)
//! - Affine texture mapping
//! - Low-resolution render target
//! - Dithering and color banding
//!
//! For now, we set up basic rendering with a retro-friendly configuration.

use bevy::prelude::*;

/// Rendering plugin - configures PSX-style visuals.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenderConfig::default());
        // Note: Fog is now a per-camera component in Bevy 0.15
        // We'll add it to cameras when they're spawned
    }
}

/// Configuration for PSX-style rendering.
#[derive(Resource)]
pub struct RenderConfig {
    /// Resolution scale (1.0 = native, lower = more pixelated)
    pub resolution_scale: f32,
    /// Vertex jitter intensity (0.0 = none, 1.0 = full PS1 wobble)
    pub vertex_jitter: f32,
    /// Enable fog for atmosphere
    pub fog_enabled: bool,
    /// Fog start distance
    pub fog_start: f32,
    /// Fog end distance (fully opaque)
    pub fog_end: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            resolution_scale: 1.0,
            vertex_jitter: 0.5,
            fog_enabled: true,
            fog_start: 5.0,
            fog_end: 20.0,
        }
    }
}
