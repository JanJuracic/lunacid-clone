//! Visual configuration loaded from external RON file.
//!
//! Allows tweaking all visual parameters without recompilation.

use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

/// Visual configuration loaded from assets/data/rendering/visual_config.ron.
#[derive(Resource, Clone, Deserialize)]
pub struct VisualConfig {
    // Post-processing
    pub grain_intensity: f32,
    pub grain_speed: f32,
    pub grain_coarseness: f32,
    pub scanline_intensity: f32,
    pub scanline_count: f32,
    pub vignette_intensity: f32,
    pub vignette_radius: f32,
    // Atmosphere
    pub fog_enabled: bool,
    pub fog_density: f32,
    pub fog_color: (f32, f32, f32),
    pub sky_color: (f32, f32, f32),
    pub clear_color: (f32, f32, f32),
}

impl Default for VisualConfig {
    fn default() -> Self {
        Self {
            // Post-processing defaults (subtle)
            grain_intensity: 0.006,
            grain_speed: 0.8,
            grain_coarseness: 180.0,
            scanline_intensity: 0.08,
            scanline_count: 320.0,
            vignette_intensity: 0.20,
            vignette_radius: 0.60,
            // Atmosphere defaults
            fog_enabled: true,
            fog_density: 0.025,
            fog_color: (0.15, 0.14, 0.13),
            sky_color: (0.12, 0.11, 0.10),
            clear_color: (0.08, 0.07, 0.06),
        }
    }
}

impl VisualConfig {
    /// Load visual config from RON file.
    pub fn load() -> Self {
        let path = "assets/data/rendering/visual_config.ron";
        match fs::read_to_string(path) {
            Ok(contents) => match ron::from_str(&contents) {
                Ok(config) => {
                    info!("Loaded visual config from {}", path);
                    config
                }
                Err(e) => {
                    error!("Failed to parse {}: {}. Using defaults.", path, e);
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Could not read {}: {}. Using defaults.", path, e);
                Self::default()
            }
        }
    }
}

/// System to load visual config at startup.
pub fn load_visual_config(mut commands: Commands) {
    let config = VisualConfig::load();
    commands.insert_resource(config);
}
