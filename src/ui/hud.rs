//! In-game HUD - health and stamina display.

use bevy::prelude::*;

use crate::combat::{Health, Stamina};
use crate::core::GameState;
use crate::player::Player;

/// Marker for HUD root entity.
#[derive(Component)]
pub struct HudRoot;

/// Marker for health bar fill.
#[derive(Component)]
pub struct HealthBar;

/// Marker for stamina bar fill.
#[derive(Component)]
pub struct StaminaBar;

/// Setup HUD systems.
pub fn setup_hud_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::InGame), spawn_hud)
        .add_systems(OnExit(GameState::InGame), cleanup_hud)
        .add_systems(
            Update,
            (update_health_bar, update_stamina_bar)
                .run_if(in_state(GameState::InGame)),
        );
}

/// Spawn the HUD UI.
fn spawn_hud(mut commands: Commands) {
    // HUD root container (bottom-left corner)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            // Stamina bar
            spawn_bar(
                parent,
                "Stamina",
                Color::srgb(0.2, 0.8, 0.3),
                StaminaBar,
                None::<StaminaBar>,
            );

            // Health bar
            spawn_bar(
                parent,
                "Health",
                Color::srgb(0.8, 0.2, 0.2),
                HealthBar,
                None::<HealthBar>,
            );
        });

    // Crosshair (center of screen)
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        HudRoot,
    )).with_children(|parent| {
        // Crosshair dot
        parent.spawn((
            Node {
                width: Val::Px(4.0),
                height: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        ));
    });
}

/// Helper to spawn a status bar.
fn spawn_bar<M: Component, C: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    color: Color,
    bar_marker: M,
    container_marker: Option<C>,
) {
    let mut container = parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(5.0)),
            ..default()
        },
    ));

    // Add container marker if provided
    if let Some(marker) = container_marker {
        container.insert(marker);
    }

    container.with_children(|bar_parent| {
        // Label
        bar_parent.spawn((
            Text::new(label),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                width: Val::Px(60.0),
                ..default()
            },
        ));

        // Bar background
        bar_parent
            .spawn((
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            ))
            .with_children(|bg| {
                // Bar fill
                bg.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(color),
                    bar_marker,
                ));
            });
    });
}

/// Update health bar based on player health.
fn update_health_bar(
    player_query: Query<&Health, With<Player>>,
    mut bar_query: Query<&mut Node, With<HealthBar>>,
) {
    let Ok(health) = player_query.get_single() else {
        return;
    };
    let Ok(mut bar) = bar_query.get_single_mut() else {
        return;
    };

    bar.width = Val::Percent(health.percentage() * 100.0);
}

/// Update stamina bar based on player stamina.
fn update_stamina_bar(
    player_query: Query<&Stamina, With<Player>>,
    mut bar_query: Query<&mut Node, With<StaminaBar>>,
) {
    let Ok(stamina) = player_query.get_single() else {
        return;
    };
    let Ok(mut bar) = bar_query.get_single_mut() else {
        return;
    };

    let percentage = stamina.current / stamina.maximum;
    bar.width = Val::Percent(percentage * 100.0);
}

/// Clean up HUD entities.
fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
