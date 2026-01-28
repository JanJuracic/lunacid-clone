//! Material definitions and registry for level geometry.

use bevy::prelude::*;
use std::collections::HashMap;

/// Material registry mapping material names to handles.
pub struct MaterialRegistry {
    materials: HashMap<String, Handle<StandardMaterial>>,
    ceilings: HashMap<String, Handle<StandardMaterial>>,
    pub pillar: Handle<StandardMaterial>,
}

impl MaterialRegistry {
    pub fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        let mut registry = HashMap::new();

        // Stone material (default) - desaturated grey-brown
        registry.insert(
            "stone".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.28, 0.27, 0.26),
                perceptual_roughness: 0.9,
                ..default()
            }),
        );

        // Stone wall material - desaturated grey-brown
        registry.insert(
            "stone_wall".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.32, 0.30, 0.28),
                perceptual_roughness: 0.8,
                ..default()
            }),
        );

        // Wood material - muted brown
        registry.insert(
            "wood".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.30, 0.25),
                perceptual_roughness: 0.7,
                ..default()
            }),
        );

        // Metal material - desaturated grey
        registry.insert(
            "metal".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.42, 0.42, 0.44),
                perceptual_roughness: 0.3,
                metallic: 0.8,
                ..default()
            }),
        );

        let mut ceilings = HashMap::new();

        // Default ceiling material - dark desaturated
        ceilings.insert(
            "ceiling".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.22, 0.21, 0.20),
                perceptual_roughness: 0.9,
                ..default()
            }),
        );

        // Stone ceiling material - desaturated grey
        ceilings.insert(
            "stone_ceiling".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.28, 0.27, 0.26),
                perceptual_roughness: 0.85,
                ..default()
            }),
        );

        // Wood ceiling material - muted brown
        ceilings.insert(
            "wood_ceiling".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.32, 0.28, 0.24),
                perceptual_roughness: 0.75,
                ..default()
            }),
        );

        // Skylight material - desaturated, dimmer
        ceilings.insert(
            "skylight".to_string(),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.45, 0.44, 0.43),
                perceptual_roughness: 0.5,
                emissive: LinearRgba::new(0.08, 0.08, 0.08, 1.0),
                ..default()
            }),
        );

        // Pillar material - desaturated grey-brown
        let pillar = materials.add(StandardMaterial {
            base_color: Color::srgb(0.38, 0.36, 0.34),
            perceptual_roughness: 0.7,
            ..default()
        });

        Self {
            materials: registry,
            ceilings,
            pillar,
        }
    }

    /// Get material for floor by name.
    pub fn get_floor(&self, material_name: &str) -> Handle<StandardMaterial> {
        self.materials
            .get(material_name)
            .cloned()
            .unwrap_or_else(|| {
                self.materials.get("stone").cloned().unwrap()
            })
    }

    /// Get material for walls by name.
    pub fn get_wall(&self, material_name: &str) -> Handle<StandardMaterial> {
        // Use _wall variant if available, else use base material
        let wall_name = format!("{}_wall", material_name);
        self.materials
            .get(&wall_name)
            .or_else(|| self.materials.get(material_name))
            .cloned()
            .unwrap_or_else(|| {
                self.materials.get("stone_wall").cloned().unwrap()
            })
    }

    /// Get material for ceilings by name.
    pub fn get_ceiling(&self, material_name: &str) -> Handle<StandardMaterial> {
        self.ceilings
            .get(material_name)
            .cloned()
            .unwrap_or_else(|| {
                self.ceilings.get("ceiling").cloned().unwrap()
            })
    }
}
