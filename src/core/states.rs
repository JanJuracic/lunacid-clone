//! Game state definitions that control the overall flow of the game.
//!
//! States determine which systems run at any given time. For example,
//! player movement only runs in the InGame state, while menu systems
//! only run in the MainMenu state.

use bevy::prelude::*;

/// Main game states - controls overall game flow.
///
/// The game transitions between these states based on player actions:
/// - Start in `Loading` to load assets
/// - Move to `MainMenu` when loading completes
/// - Enter `InGame` when player starts/continues
/// - `Paused` freezes gameplay but keeps the world visible
/// - `GameOver` when player dies
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    /// Initial state - loading assets and data files
    #[default]
    Loading,
    /// Main menu / title screen
    MainMenu,
    /// Active gameplay
    InGame,
    /// Game is paused (overlay on gameplay)
    Paused,
    /// Player has died
    GameOver,
}

/// Sub-states for gameplay - only active when GameState::InGame.
///
/// These control what the player can do during active gameplay:
/// - `Exploring`: Normal movement, combat, and interaction
/// - `Inventory`: Inventory screen is open, gameplay paused
/// - `Dialogue`: Talking to an NPC (future feature)
#[derive(SubStates, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
#[source(GameState = GameState::InGame)]
pub enum PlayState {
    /// Normal gameplay - movement, combat, exploration
    #[default]
    Exploring,
    /// Inventory screen is open
    Inventory,
    /// Dialogue with NPC (future)
    Dialogue,
}
