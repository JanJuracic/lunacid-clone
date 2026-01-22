//! World plugin - level loading, environment, and interactables.

use bevy::prelude::*;

use crate::core::GameState;
use crate::enemies::data::EnemyRegistry;
use crate::player::spawn_player;

use super::builder::{build_level_from_data, LevelGeometry};
use super::data::{load_level_definitions, load_palette_files, CurrentLevel, LevelRegistry};

/// World plugin - handles level loading and world setup.
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (load_palette_files, load_level_definitions).chain(),
        )
        .add_systems(OnEnter(GameState::InGame), setup_level)
        .add_systems(OnExit(GameState::InGame), cleanup_level);
    }
}

/// Set up the level from data.
pub fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_registry: Res<LevelRegistry>,
    current_level: Res<CurrentLevel>,
    asset_server: Res<AssetServer>,
    enemy_registry: Res<EnemyRegistry>,
) {
    let Some(level) = level_registry.get(&current_level.name) else {
        error!("Level '{}' not found in registry!", current_level.name);
        return;
    };

    info!("Building level: {}", level.name);

    let player_pos = build_level_from_data(
        &mut commands,
        &mut meshes,
        &mut materials,
        level,
        &asset_server,
        &enemy_registry,
    );

    spawn_player(&mut commands, player_pos);
}

/// Clean up level entities when leaving InGame state.
fn cleanup_level(
    mut commands: Commands,
    level_query: Query<Entity, With<LevelGeometry>>,
    player_query: Query<Entity, With<crate::player::Player>>,
) {
    for entity in level_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in player_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
