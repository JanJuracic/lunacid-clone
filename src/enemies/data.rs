//! Enemy data loading from RON files.

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::components::EnemyStats;

/// Animation clip indices for an enemy type.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct AnimationIndices {
    pub idle: u32,
    pub walk: u32,
    pub combat_idle: Option<u32>, // Falls back to idle
    pub attack: u32,
    pub hurt: Option<u32>,
    pub death: u32,
}

/// Animation configuration for an enemy type.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct AnimationConfig {
    pub indices: AnimationIndices,
    pub attack_hit_frame: f32, // 0.0-1.0, when damage applies
    pub hurt_duration: f32,    // seconds
}

/// Collider configuration for an enemy type.
#[derive(Deserialize, Clone, Debug)]
pub struct ColliderConfig {
    pub half_height: f32,
    pub radius: f32,
}

impl Default for ColliderConfig {
    fn default() -> Self {
        Self {
            half_height: 0.5,
            radius: 0.3,
        }
    }
}

/// Enemy definition loaded from RON file.
#[derive(Deserialize, Clone, Debug)]
pub struct EnemyDefinition {
    pub name: String,
    pub max_health: f32,
    pub damage: f32,
    pub move_speed: f32,
    pub detection_range: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub model_path: String,
    pub scale: f32,
    #[serde(default)]
    pub collider: Option<ColliderConfig>,
    #[serde(default)]
    pub animations: Option<AnimationConfig>,
}

impl EnemyDefinition {
    /// Convert to EnemyStats component.
    pub fn to_stats(&self) -> EnemyStats {
        EnemyStats {
            max_health: self.max_health,
            damage: self.damage,
            move_speed: self.move_speed,
            detection_range: self.detection_range,
            attack_range: self.attack_range,
            attack_cooldown: self.attack_cooldown,
        }
    }
}

/// Resource holding all loaded enemy definitions.
#[derive(Resource, Default)]
pub struct EnemyRegistry {
    pub definitions: HashMap<String, EnemyDefinition>,
}

impl EnemyRegistry {
    /// Get an enemy definition by type name.
    pub fn get(&self, enemy_type: &str) -> Option<&EnemyDefinition> {
        self.definitions.get(enemy_type)
    }
}

/// Load all enemy definitions from the assets/data/enemies/ directory.
pub fn load_enemy_definitions(mut registry: ResMut<EnemyRegistry>) {
    let enemies_dir = Path::new("assets/data/enemies");

    if !enemies_dir.exists() {
        warn!("Enemy definitions directory not found: {:?}", enemies_dir);
        return;
    }

    let Ok(entries) = fs::read_dir(enemies_dir) else {
        warn!("Failed to read enemy definitions directory");
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "ron") {
            let enemy_type = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            match fs::read_to_string(&path) {
                Ok(contents) => match ron::from_str::<EnemyDefinition>(&contents) {
                    Ok(definition) => {
                        info!("Loaded enemy definition: {} ({})", definition.name, enemy_type);
                        registry.definitions.insert(enemy_type, definition);
                    }
                    Err(e) => {
                        error!("Failed to parse enemy definition {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    error!("Failed to read enemy definition {:?}: {}", path, e);
                }
            }
        }
    }

    info!(
        "Loaded {} enemy definitions",
        registry.definitions.len()
    );
}
