//! UI plugin - menus, HUD, and interface elements.

use bevy::prelude::*;

use crate::core::GameState;
use super::hud;

/// UI plugin - handles all user interface.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Setup HUD systems
        hud::setup_hud_systems(app);

        app
            // Main menu
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(Update, main_menu_input.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)

            // Pause menu
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(Update, pause_menu_input.run_if(in_state(GameState::Paused)))
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu)

            // Game over
            .add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(Update, game_over_input.run_if(in_state(GameState::GameOver)))
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over);
    }
}

/// Marker for main menu UI entities.
#[derive(Component)]
struct MainMenuUi;

/// Marker for the menu camera (used when no game camera exists).
#[derive(Component)]
struct MenuCamera;

/// Marker for pause menu UI entities.
#[derive(Component)]
struct PauseMenuUi;

/// Marker for game over UI entities.
#[derive(Component)]
struct GameOverUi;

/// Marker for menu buttons.
#[derive(Component)]
enum MenuButton {
    NewGame,
    #[allow(dead_code)]
    Continue,
    #[allow(dead_code)]
    Options,
    Quit,
    Resume,
    MainMenu,
    Retry,
}

/// Set up the main menu.
fn setup_main_menu(mut commands: Commands) {
    // Spawn a camera for UI rendering in menu state
    commands.spawn((
        Camera2d,
        MenuCamera,
    ));

    // Root container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
            MainMenuUi,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("LUNACID"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.7, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("A Bevy Clone"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.55)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // New Game button
            spawn_menu_button(parent, "New Game", MenuButton::NewGame);

            // Quit button
            spawn_menu_button(parent, "Quit", MenuButton::Quit);
        });
}

/// Helper to spawn a menu button.
fn spawn_menu_button(parent: &mut ChildBuilder, text: &str, button: MenuButton) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                margin: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            button,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.85)),
            ));
        });
}

/// Handle main menu button interactions.
fn main_menu_input(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *bg_color = Color::srgb(0.3, 0.3, 0.35).into();
                match button {
                    MenuButton::NewGame => {
                        next_state.set(GameState::InGame);
                    }
                    MenuButton::Quit => {
                        exit.send(AppExit::Success);
                    }
                    _ => {}
                }
            }
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.25, 0.25, 0.3).into();
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.15, 0.15, 0.2).into();
            }
        }
    }
}

/// Clean up main menu entities.
fn cleanup_main_menu(
    mut commands: Commands,
    ui_query: Query<Entity, With<MainMenuUi>>,
    camera_query: Query<Entity, With<MenuCamera>>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in camera_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Set up the pause menu.
fn setup_pause_menu(mut commands: Commands) {
    // Semi-transparent overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuUi,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.85)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Resume button
            spawn_menu_button(parent, "Resume", MenuButton::Resume);

            // Main Menu button
            spawn_menu_button(parent, "Main Menu", MenuButton::MainMenu);
        });
}

/// Handle pause menu button interactions.
fn pause_menu_input(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *bg_color = Color::srgb(0.3, 0.3, 0.35).into();
                match button {
                    MenuButton::Resume => {
                        next_state.set(GameState::InGame);
                    }
                    MenuButton::MainMenu => {
                        next_state.set(GameState::MainMenu);
                    }
                    _ => {}
                }
            }
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.25, 0.25, 0.3).into();
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.15, 0.15, 0.2).into();
            }
        }
    }
}

/// Clean up pause menu entities.
fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Set up the game over screen.
fn setup_game_over(mut commands: Commands) {
    // Spawn a camera for UI rendering
    commands.spawn((
        Camera2d,
        MenuCamera,
    ));

    // Dark overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.0, 0.0, 0.9)),
            GameOverUi,
        ))
        .with_children(|parent| {
            // YOU DIED text
            parent.spawn((
                Text::new("YOU DIED"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.2, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // Retry button
            spawn_menu_button(parent, "Retry", MenuButton::Retry);

            // Main Menu button
            spawn_menu_button(parent, "Main Menu", MenuButton::MainMenu);
        });
}

/// Handle game over button interactions.
fn game_over_input(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *bg_color = Color::srgb(0.3, 0.3, 0.35).into();
                match button {
                    MenuButton::Retry => {
                        next_state.set(GameState::InGame);
                    }
                    MenuButton::MainMenu => {
                        next_state.set(GameState::MainMenu);
                    }
                    _ => {}
                }
            }
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.25, 0.25, 0.3).into();
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.15, 0.15, 0.2).into();
            }
        }
    }
}

/// Clean up game over entities.
fn cleanup_game_over(
    mut commands: Commands,
    ui_query: Query<Entity, With<GameOverUi>>,
    camera_query: Query<Entity, With<MenuCamera>>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in camera_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
