//! Core plugin that sets up game states, events, and fundamental systems.

use bevy::prelude::*;

use super::events::*;
use super::states::*;
use super::tween::*;

/// Core plugin - must be added first as other plugins depend on it.
///
/// This plugin sets up:
/// - Game states (Loading, MainMenu, InGame, etc.)
/// - Global events (DamageEvent, DeathEvent, etc.)
/// - Basic game flow systems
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize game states
            .init_state::<GameState>()
            .add_sub_state::<PlayState>()

            // Register global events
            .add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_event::<ItemPickupEvent>()
            .add_event::<LevelUpEvent>()

            // Loading state - transition to MainMenu when ready
            // For now, immediately transition since we have no assets to load
            .add_systems(OnEnter(GameState::Loading), transition_to_main_menu)

            // Pause/unpause with Escape key
            .add_systems(
                Update,
                handle_pause_input.run_if(in_state(GameState::InGame).or(in_state(GameState::Paused)))
            )

            // Smooth transform interpolation (runs for all game states)
            .add_systems(Update, update_smooth_transforms);
    }
}

/// Immediately transition from Loading to MainMenu.
/// Later this will wait for assets to load.
fn transition_to_main_menu(mut next_state: ResMut<NextState<GameState>>) {
    // TODO: Add actual asset loading checks here
    next_state.set(GameState::MainMenu);
}

/// Handle Escape key to pause/unpause the game.
fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::InGame => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::InGame),
            _ => {}
        }
    }
}
