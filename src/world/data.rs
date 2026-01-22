//! Level data structures and RON loading.

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// === External Palette File Types ===

/// External geometry palette file structure.
#[derive(Debug, Clone, Deserialize)]
pub struct GeometryPaletteFile {
    pub tiles: HashMap<char, GeometryTileDef>,
}

/// External ambient palette file structure.
#[derive(Debug, Clone, Deserialize)]
pub struct AmbientPaletteFile {
    pub tiles: HashMap<char, AmbientTileDef>,
}

/// External monster palette file structure.
#[derive(Debug, Clone, Deserialize)]
pub struct MonsterPaletteFile {
    pub entries: HashMap<char, String>,
}

/// External ceiling palette file structure.
#[derive(Debug, Clone, Deserialize)]
pub struct CeilingPaletteFile {
    pub tiles: HashMap<char, CeilingTileDef>,
}

/// Definition of a ceiling tile in the palette.
#[derive(Debug, Clone, Deserialize)]
pub struct CeilingTileDef {
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub thickness: Option<f32>,
}

/// A resolved ceiling tile with all properties.
#[derive(Debug, Clone)]
pub struct ResolvedCeilingTile {
    pub material: String,
    pub height: f32,
    pub thickness: f32,
}

/// Registry storing loaded external palette files.
#[derive(Resource, Default)]
pub struct PaletteRegistry {
    pub geometry: HashMap<String, GeometryPaletteFile>,
    pub ambient: HashMap<String, AmbientPaletteFile>,
    pub monster: HashMap<String, MonsterPaletteFile>,
    pub ceiling: HashMap<String, CeilingPaletteFile>,
}

impl PaletteRegistry {
    /// Get a geometry palette by filename.
    pub fn get_geometry(&self, filename: &str) -> Option<&GeometryPaletteFile> {
        self.geometry.get(filename)
    }

    /// Get an ambient palette by filename.
    pub fn get_ambient(&self, filename: &str) -> Option<&AmbientPaletteFile> {
        self.ambient.get(filename)
    }

    /// Get a monster palette by filename.
    pub fn get_monster(&self, filename: &str) -> Option<&MonsterPaletteFile> {
        self.monster.get(filename)
    }

    /// Get a ceiling palette by filename.
    pub fn get_ceiling(&self, filename: &str) -> Option<&CeilingPaletteFile> {
        self.ceiling.get(filename)
    }
}

// === Geometry Types ===

/// The kind of geometry tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum GeometryKind {
    Floor,
    Wall,
    Pillar,
    Doorway,
    Void,
}

impl GeometryKind {
    /// Whether this tile kind has a floor.
    pub fn has_floor(&self) -> bool {
        matches!(self, GeometryKind::Floor | GeometryKind::Pillar | GeometryKind::Doorway)
    }

    /// Whether this tile kind is solid (blocks movement).
    pub fn is_solid(&self) -> bool {
        matches!(self, GeometryKind::Wall)
    }
}

/// Definition of a geometry tile in the palette.
#[derive(Debug, Clone, Deserialize)]
pub struct GeometryTileDef {
    pub kind: GeometryKind,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub floor_depth: Option<f32>,
}

// === Ambient Types ===

fn default_light_color() -> (f32, f32, f32) {
    (1.0, 0.8, 0.6)
}

fn default_light_range() -> f32 {
    15.0
}

fn default_volume() -> f32 {
    0.5
}

fn default_radius() -> f32 {
    5.0
}

fn default_particle_rate() -> f32 {
    5.0
}

/// Light definition for ambient tiles.
#[derive(Debug, Clone, Deserialize)]
pub struct LightDef {
    pub height: f32,
    pub intensity: f32,
    #[serde(default)]
    pub shadows: bool,
    #[serde(default = "default_light_color")]
    pub color: (f32, f32, f32),
    #[serde(default = "default_light_range")]
    pub range: f32,
}

/// Particle definition for ambient tiles.
#[derive(Debug, Clone, Deserialize)]
pub struct ParticleDef {
    pub kind: String,
    pub height: f32,
    #[serde(default = "default_particle_rate")]
    pub rate: f32,
    #[serde(default)]
    pub color: Option<(f32, f32, f32, f32)>,
}

/// Audio definition for ambient tiles.
#[derive(Debug, Clone, Deserialize)]
pub struct AudioDef {
    pub sound: String,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_radius")]
    pub radius: f32,
}

/// Definition of an ambient tile in the palette.
/// Supports stacking multiple lights, particles, and audio zones.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AmbientTileDef {
    #[serde(default)]
    pub lights: Vec<LightDef>,
    #[serde(default)]
    pub particles: Vec<ParticleDef>,
    #[serde(default)]
    pub audio: Vec<AudioDef>,
}

// === Global Ambient ===

/// Global ambient light settings.
#[derive(Debug, Clone, Deserialize)]
pub struct GlobalAmbientDef {
    pub color: (f32, f32, f32),
    pub brightness: f32,
}

impl Default for GlobalAmbientDef {
    fn default() -> Self {
        Self {
            color: (0.4, 0.4, 0.5),
            brightness: 30.0,
        }
    }
}

// === Spawn Zones (deprecated - kept for backwards compatibility) ===

/// Spawn zone definition (deprecated - use monster grid instead).
#[derive(Debug, Clone, Deserialize)]
pub struct SpawnZoneDef {
    pub pos: (i32, i32),
    pub half_extents: (i32, i32),
    pub enemy_type: String,
    pub max_enemies: usize,
    #[serde(default)]
    pub spawn_delay: f32,
}

// === Monster Spawns ===

/// A resolved monster spawn point from the monster grid.
#[derive(Debug, Clone)]
pub struct ResolvedMonsterSpawn {
    /// Grid position (x, z).
    pub grid_pos: (i32, i32),
    /// Enemy type identifier (matches EnemyRegistry key).
    pub enemy_type: String,
}

// === Level Definition ===

fn default_tile_size() -> f32 {
    2.0
}

fn default_wall_height() -> f32 {
    4.0
}

fn default_floor_depth() -> f32 {
    0.5
}

fn default_ceiling_thickness() -> f32 {
    0.3
}

/// Raw level definition as read from RON.
#[derive(Debug, Clone, Deserialize)]
pub struct LevelDefinitionRaw {
    pub name: String,
    #[serde(default = "default_tile_size")]
    pub tile_size: f32,
    #[serde(default = "default_wall_height")]
    pub default_wall_height: f32,
    #[serde(default = "default_floor_depth")]
    pub default_floor_depth: f32,
    #[serde(default = "default_wall_height")]
    pub default_ceiling_height: f32,
    #[serde(default = "default_ceiling_thickness")]
    pub default_ceiling_thickness: f32,
    #[serde(default)]
    pub global_ambient: GlobalAmbientDef,
    pub player_start: (i32, i32),

    // External palette file references (optional)
    #[serde(default)]
    pub geometry_palette_file: Option<String>,
    #[serde(default)]
    pub ambient_palette_file: Option<String>,
    #[serde(default)]
    pub monster_palette_file: Option<String>,
    #[serde(default)]
    pub ceiling_palette_file: Option<String>,

    // Inline palettes (optional - used as fallback if no external file)
    #[serde(default)]
    pub geometry_palette: HashMap<char, GeometryTileDef>,
    #[serde(default)]
    pub ambient_palette: HashMap<char, AmbientTileDef>,
    #[serde(default)]
    pub monster_palette: HashMap<char, String>,
    #[serde(default)]
    pub ceiling_palette: HashMap<char, CeilingTileDef>,

    // Grids
    pub geometry: Vec<String>,
    pub ambient: Vec<String>,
    #[serde(default)]
    pub monsters: Vec<String>,
    #[serde(default)]
    pub ceiling: Vec<String>,

    // Legacy spawn zones (deprecated)
    #[serde(default)]
    pub spawn_zones: Vec<SpawnZoneDef>,
}

/// A resolved geometry tile with all properties.
#[derive(Debug, Clone)]
pub struct ResolvedGeometryTile {
    pub kind: GeometryKind,
    pub material: String,
    pub height: f32,
    pub floor_depth: f32,
}

impl Default for ResolvedGeometryTile {
    fn default() -> Self {
        Self {
            kind: GeometryKind::Void,
            material: "stone".to_string(),
            height: 4.0,
            floor_depth: 0.5,
        }
    }
}

/// A resolved ambient tile with all its elements.
#[derive(Debug, Clone, Default)]
pub struct ResolvedAmbientTile {
    pub lights: Vec<LightDef>,
    pub particles: Vec<ParticleDef>,
    pub audio: Vec<AudioDef>,
}

/// Processed level definition with resolved tiles.
#[derive(Debug, Clone)]
pub struct LevelDefinition {
    pub name: String,
    pub tile_size: f32,
    pub default_wall_height: f32,
    pub default_floor_depth: f32,
    pub default_ceiling_height: f32,
    pub default_ceiling_thickness: f32,
    pub global_ambient: GlobalAmbientDef,
    pub player_start: (i32, i32),
    pub width: usize,
    pub height: usize,
    pub geometry: Vec<Vec<ResolvedGeometryTile>>,
    pub ambient: Vec<Vec<ResolvedAmbientTile>>,
    /// Ceiling grid (None = open sky/void).
    pub ceiling: Vec<Vec<Option<ResolvedCeilingTile>>>,
    /// Monster spawn points resolved from the monster grid.
    pub monster_spawns: Vec<ResolvedMonsterSpawn>,
    /// Legacy spawn zones (deprecated - use monster_spawns).
    pub spawn_zones: Vec<SpawnZoneDef>,
}

impl LevelDefinition {
    /// Create from raw definition by resolving palette references.
    /// Uses PaletteRegistry to look up external palette files.
    pub fn from_raw(raw: LevelDefinitionRaw, palette_registry: &PaletteRegistry) -> Result<Self, String> {
        let geo_height = raw.geometry.len();
        let geo_width = raw.geometry.iter().map(|row| row.chars().count()).max().unwrap_or(0);

        let amb_height = raw.ambient.len();
        let amb_width = raw.ambient.iter().map(|row| row.chars().count()).max().unwrap_or(0);

        // Validate grid dimensions match
        if geo_height != amb_height || geo_width != amb_width {
            return Err(format!(
                "Grid dimension mismatch: geometry is {}x{}, ambient is {}x{}",
                geo_width, geo_height, amb_width, amb_height
            ));
        }

        // Resolve geometry palette: prefer external file, fallback to inline
        let geometry_palette: HashMap<char, GeometryTileDef> = if let Some(ref filename) = raw.geometry_palette_file {
            palette_registry
                .get_geometry(filename)
                .map(|f| f.tiles.clone())
                .unwrap_or_else(|| {
                    warn!("Geometry palette file '{}' not found, using inline", filename);
                    raw.geometry_palette.clone()
                })
        } else {
            raw.geometry_palette.clone()
        };

        // Resolve ambient palette: prefer external file, fallback to inline
        let ambient_palette: HashMap<char, AmbientTileDef> = if let Some(ref filename) = raw.ambient_palette_file {
            palette_registry
                .get_ambient(filename)
                .map(|f| f.tiles.clone())
                .unwrap_or_else(|| {
                    warn!("Ambient palette file '{}' not found, using inline", filename);
                    raw.ambient_palette.clone()
                })
        } else {
            raw.ambient_palette.clone()
        };

        // Resolve monster palette: prefer external file, fallback to inline
        let monster_palette: HashMap<char, String> = if let Some(ref filename) = raw.monster_palette_file {
            palette_registry
                .get_monster(filename)
                .map(|f| f.entries.clone())
                .unwrap_or_else(|| {
                    warn!("Monster palette file '{}' not found, using inline", filename);
                    raw.monster_palette.clone()
                })
        } else {
            raw.monster_palette.clone()
        };

        // Resolve ceiling palette: prefer external file, fallback to inline
        let ceiling_palette: HashMap<char, CeilingTileDef> = if let Some(ref filename) = raw.ceiling_palette_file {
            palette_registry
                .get_ceiling(filename)
                .map(|f| f.tiles.clone())
                .unwrap_or_else(|| {
                    warn!("Ceiling palette file '{}' not found, using inline", filename);
                    raw.ceiling_palette.clone()
                })
        } else {
            raw.ceiling_palette.clone()
        };

        // Resolve geometry grid
        let geometry: Vec<Vec<ResolvedGeometryTile>> = raw
            .geometry
            .iter()
            .map(|row| {
                let mut tile_row: Vec<ResolvedGeometryTile> = row
                    .chars()
                    .map(|c| {
                        if let Some(def) = geometry_palette.get(&c) {
                            ResolvedGeometryTile {
                                kind: def.kind,
                                material: def.material.clone().unwrap_or_else(|| "stone".to_string()),
                                height: def.height.unwrap_or(raw.default_wall_height),
                                floor_depth: def.floor_depth.unwrap_or(raw.default_floor_depth),
                            }
                        } else {
                            // Unknown character defaults to void
                            ResolvedGeometryTile::default()
                        }
                    })
                    .collect();
                // Pad to consistent width
                tile_row.resize(geo_width, ResolvedGeometryTile::default());
                tile_row
            })
            .collect();

        // Resolve ambient grid
        let ambient: Vec<Vec<ResolvedAmbientTile>> = raw
            .ambient
            .iter()
            .map(|row| {
                let mut tile_row: Vec<ResolvedAmbientTile> = row
                    .chars()
                    .map(|c| {
                        if c == '.' || c == ' ' {
                            // No ambient elements
                            ResolvedAmbientTile::default()
                        } else if let Some(def) = ambient_palette.get(&c) {
                            ResolvedAmbientTile {
                                lights: def.lights.clone(),
                                particles: def.particles.clone(),
                                audio: def.audio.clone(),
                            }
                        } else {
                            // Unknown character means no ambient
                            ResolvedAmbientTile::default()
                        }
                    })
                    .collect();
                // Pad to consistent width
                tile_row.resize(amb_width, ResolvedAmbientTile::default());
                tile_row
            })
            .collect();

        // Resolve monster grid
        let mut monster_spawns = Vec::new();
        if !raw.monsters.is_empty() {
            // Validate monster grid dimensions
            let mon_height = raw.monsters.len();
            let mon_width = raw.monsters.iter().map(|row| row.chars().count()).max().unwrap_or(0);
            if mon_height != geo_height || mon_width != geo_width {
                return Err(format!(
                    "Monster grid dimension mismatch: geometry is {}x{}, monsters is {}x{}",
                    geo_width, geo_height, mon_width, mon_height
                ));
            }

            for (z, row) in raw.monsters.iter().enumerate() {
                for (x, c) in row.chars().enumerate() {
                    if c != '.' && c != ' ' {
                        if let Some(enemy_type) = monster_palette.get(&c) {
                            monster_spawns.push(ResolvedMonsterSpawn {
                                grid_pos: (x as i32, z as i32),
                                enemy_type: enemy_type.clone(),
                            });
                        } else {
                            warn!("Unknown monster character '{}' at ({}, {})", c, x, z);
                        }
                    }
                }
            }
        }

        // Resolve ceiling grid
        let ceiling: Vec<Vec<Option<ResolvedCeilingTile>>> = if !raw.ceiling.is_empty() {
            // Validate ceiling grid dimensions
            let ceil_height = raw.ceiling.len();
            let ceil_width = raw.ceiling.iter().map(|row| row.chars().count()).max().unwrap_or(0);
            if ceil_height != geo_height || ceil_width != geo_width {
                return Err(format!(
                    "Ceiling grid dimension mismatch: geometry is {}x{}, ceiling is {}x{}",
                    geo_width, geo_height, ceil_width, ceil_height
                ));
            }

            raw.ceiling
                .iter()
                .map(|row| {
                    let mut tile_row: Vec<Option<ResolvedCeilingTile>> = row
                        .chars()
                        .map(|c| {
                            if c == '.' || c == ' ' {
                                // No ceiling (open sky/void)
                                None
                            } else if let Some(def) = ceiling_palette.get(&c) {
                                Some(ResolvedCeilingTile {
                                    material: def.material.clone().unwrap_or_else(|| "ceiling".to_string()),
                                    height: def.height.unwrap_or(raw.default_ceiling_height),
                                    thickness: def.thickness.unwrap_or(raw.default_ceiling_thickness),
                                })
                            } else {
                                // Unknown character: default ceiling
                                Some(ResolvedCeilingTile {
                                    material: "ceiling".to_string(),
                                    height: raw.default_ceiling_height,
                                    thickness: raw.default_ceiling_thickness,
                                })
                            }
                        })
                        .collect();
                    // Pad to consistent width with None (open)
                    tile_row.resize(geo_width, None);
                    tile_row
                })
                .collect()
        } else {
            // No ceiling grid provided: generate default ceiling for all floor tiles
            geometry
                .iter()
                .map(|geo_row| {
                    geo_row
                        .iter()
                        .map(|geo_tile| {
                            if geo_tile.kind.has_floor() {
                                Some(ResolvedCeilingTile {
                                    material: "ceiling".to_string(),
                                    height: raw.default_ceiling_height,
                                    thickness: raw.default_ceiling_thickness,
                                })
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .collect()
        };

        Ok(Self {
            name: raw.name,
            tile_size: raw.tile_size,
            default_wall_height: raw.default_wall_height,
            default_floor_depth: raw.default_floor_depth,
            default_ceiling_height: raw.default_ceiling_height,
            default_ceiling_thickness: raw.default_ceiling_thickness,
            global_ambient: raw.global_ambient,
            player_start: raw.player_start,
            width: geo_width,
            height: geo_height,
            geometry,
            ambient,
            ceiling,
            monster_spawns,
            spawn_zones: raw.spawn_zones,
        })
    }

    /// Get geometry tile at grid position. Returns default (Void) if out of bounds.
    pub fn get_geometry(&self, x: i32, z: i32) -> &ResolvedGeometryTile {
        static DEFAULT: ResolvedGeometryTile = ResolvedGeometryTile {
            kind: GeometryKind::Void,
            material: String::new(),
            height: 4.0,
            floor_depth: 0.5,
        };

        if x < 0 || z < 0 {
            return &DEFAULT;
        }
        let ux = x as usize;
        let uz = z as usize;
        if uz >= self.height || ux >= self.width {
            return &DEFAULT;
        }
        &self.geometry[uz][ux]
    }

    /// Get ambient tile at grid position. Returns empty ambient if out of bounds.
    pub fn get_ambient(&self, x: i32, z: i32) -> &ResolvedAmbientTile {
        static DEFAULT: ResolvedAmbientTile = ResolvedAmbientTile {
            lights: Vec::new(),
            particles: Vec::new(),
            audio: Vec::new(),
        };

        if x < 0 || z < 0 {
            return &DEFAULT;
        }
        let ux = x as usize;
        let uz = z as usize;
        if uz >= self.height || ux >= self.width {
            return &DEFAULT;
        }
        &self.ambient[uz][ux]
    }

    /// Get ceiling tile at grid position. Returns None if out of bounds or open sky.
    pub fn get_ceiling(&self, x: i32, z: i32) -> Option<&ResolvedCeilingTile> {
        if x < 0 || z < 0 {
            return None;
        }
        let ux = x as usize;
        let uz = z as usize;
        if uz >= self.height || ux >= self.width {
            return None;
        }
        self.ceiling[uz][ux].as_ref()
    }

    /// Convert grid coordinates to world position (center of tile).
    pub fn grid_to_world(&self, x: i32, z: i32) -> Vec3 {
        Vec3::new(
            x as f32 * self.tile_size + self.tile_size / 2.0,
            0.0,
            z as f32 * self.tile_size + self.tile_size / 2.0,
        )
    }
}

/// Resource storing all loaded level definitions.
#[derive(Resource, Default)]
pub struct LevelRegistry {
    pub levels: HashMap<String, LevelDefinition>,
}

impl LevelRegistry {
    /// Get a level by name.
    pub fn get(&self, name: &str) -> Option<&LevelDefinition> {
        self.levels.get(name)
    }
}

/// Resource indicating which level to load.
#[derive(Resource)]
pub struct CurrentLevel {
    pub name: String,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self {
            name: "level1".to_string(),
        }
    }
}

/// Load all external palette files from assets/data/palettes/.
pub fn load_palette_files(mut commands: Commands) {
    let mut registry = PaletteRegistry::default();
    let palettes_path = Path::new("assets/data/palettes");

    if palettes_path.exists() {
        if let Ok(entries) = fs::read_dir(palettes_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "ron") {
                    let filename = path.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();

                    if let Ok(contents) = fs::read_to_string(&path) {
                        // Try to determine palette type by filename convention first
                        let is_ceiling = filename.contains("ceiling");
                        let is_geometry = filename.contains("geometry");
                        let is_ambient = filename.contains("ambient");
                        let is_monster = filename.contains("monster");

                        if is_ceiling {
                            if let Ok(ceil_palette) = ron::from_str::<CeilingPaletteFile>(&contents) {
                                info!("Loaded ceiling palette: {}", filename);
                                registry.ceiling.insert(filename.clone(), ceil_palette);
                            } else {
                                warn!("Failed to parse ceiling palette {:?}", path);
                            }
                        } else if is_geometry {
                            if let Ok(geo_palette) = ron::from_str::<GeometryPaletteFile>(&contents) {
                                info!("Loaded geometry palette: {}", filename);
                                registry.geometry.insert(filename.clone(), geo_palette);
                            } else {
                                warn!("Failed to parse geometry palette {:?}", path);
                            }
                        } else if is_ambient {
                            if let Ok(amb_palette) = ron::from_str::<AmbientPaletteFile>(&contents) {
                                info!("Loaded ambient palette: {}", filename);
                                registry.ambient.insert(filename.clone(), amb_palette);
                            } else {
                                warn!("Failed to parse ambient palette {:?}", path);
                            }
                        } else if is_monster {
                            if let Ok(mon_palette) = ron::from_str::<MonsterPaletteFile>(&contents) {
                                info!("Loaded monster palette: {}", filename);
                                registry.monster.insert(filename.clone(), mon_palette);
                            } else {
                                warn!("Failed to parse monster palette {:?}", path);
                            }
                        } else {
                            // Fallback: try each format in order
                            if let Ok(geo_palette) = ron::from_str::<GeometryPaletteFile>(&contents) {
                                info!("Loaded geometry palette: {}", filename);
                                registry.geometry.insert(filename.clone(), geo_palette);
                            } else if let Ok(mon_palette) = ron::from_str::<MonsterPaletteFile>(&contents) {
                                info!("Loaded monster palette: {}", filename);
                                registry.monster.insert(filename.clone(), mon_palette);
                            } else if let Ok(ceil_palette) = ron::from_str::<CeilingPaletteFile>(&contents) {
                                info!("Loaded ceiling palette: {}", filename);
                                registry.ceiling.insert(filename.clone(), ceil_palette);
                            } else if let Ok(amb_palette) = ron::from_str::<AmbientPaletteFile>(&contents) {
                                info!("Loaded ambient palette: {}", filename);
                                registry.ambient.insert(filename.clone(), amb_palette);
                            } else {
                                warn!("Unknown palette format in {:?}", path);
                            }
                        }
                    } else {
                        error!("Failed to read palette file {:?}", path);
                    }
                }
            }
        }
    } else {
        info!("Palettes directory not found, using inline palettes only");
    }

    info!(
        "Loaded {} geometry, {} ambient, {} monster, {} ceiling palettes",
        registry.geometry.len(),
        registry.ambient.len(),
        registry.monster.len(),
        registry.ceiling.len()
    );
    commands.insert_resource(registry);
}

/// Load all level definitions from assets/data/levels/.
pub fn load_level_definitions(mut commands: Commands, palette_registry: Res<PaletteRegistry>) {
    let mut registry = LevelRegistry::default();

    let levels_path = Path::new("assets/data/levels");

    if levels_path.exists() {
        if let Ok(entries) = fs::read_dir(levels_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "ron") {
                    if let Some(stem) = path.file_stem() {
                        let name = stem.to_string_lossy();
                        let level_name = name.strip_suffix(".level").unwrap_or(&name).to_string();

                        match fs::read_to_string(&path) {
                            Ok(contents) => match ron::from_str::<LevelDefinitionRaw>(&contents) {
                                Ok(raw) => match LevelDefinition::from_raw(raw, &palette_registry) {
                                    Ok(level) => {
                                        info!("Loaded level: {}", level_name);
                                        registry.levels.insert(level_name, level);
                                    }
                                    Err(e) => {
                                        error!("Failed to process level {:?}: {}", path, e);
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to parse level {:?}: {}", path, e);
                                }
                            },
                            Err(e) => {
                                error!("Failed to read level file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
    } else {
        warn!("Levels directory not found: {:?}", levels_path);
    }

    info!("Loaded {} level(s)", registry.levels.len());
    commands.insert_resource(registry);
    commands.insert_resource(CurrentLevel::default());
}
