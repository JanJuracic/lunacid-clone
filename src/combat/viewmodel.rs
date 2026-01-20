//! Weapon viewmodel - first-person weapon display.
//!
//! The viewmodel is spawned as a child of the player's camera, eliminating
//! the one-frame trailing issue that occurs with manual position tracking.

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::scene::SceneRoot;

use super::components::*;
use crate::core::{GameState, SmoothTransform};
use crate::player::{Player, PlayerCamera};

/// Marker for the weapon viewmodel entity.
#[derive(Component)]
pub struct WeaponViewmodel;

/// Setup weapon viewmodel systems.
pub fn setup_viewmodel_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_viewmodel,
            propagate_viewmodel_render_layers,
            update_viewmodel_position,
            update_viewmodel_animation,
        )
            .chain()
            .run_if(in_state(GameState::InGame)),
    );
}

/// Spawn the weapon viewmodel as a child of the camera.
///
/// This system checks if a viewmodel already exists - if not, it spawns one
/// as a child of the player's camera using the weapon's model_path.
fn spawn_viewmodel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    player_query: Query<&Weapon, With<Player>>,
    viewmodel_query: Query<&WeaponViewmodel>,
) {
    // Don't spawn if viewmodel already exists
    if viewmodel_query.iter().next().is_some() {
        return;
    }

    let Ok(camera_entity) = camera_query.get_single() else {
        return;
    };

    let Ok(weapon) = player_query.get_single() else {
        return;
    };

    // Don't spawn viewmodel for weapons without a model
    if weapon.model_path.is_empty() {
        return;
    }

    // Spawn viewmodel as child of camera
    // Use RenderLayers::layer(1) so weapon renders on weapon camera only
    commands.entity(camera_entity).with_children(|parent| {
        parent
            .spawn((
                WeaponViewmodel,
                Transform::from_xyz(0.3, -0.2, -0.5),
                SmoothTransform::new(15.0, 12.0),
                Visibility::default(),
                RenderLayers::layer(1),
            ))
            .with_children(|weapon_parent| {
                weapon_parent.spawn((
                    SceneRoot(asset_server.load(&weapon.model_path)),
                    Transform::from_scale(Vec3::splat(0.15)),
                    RenderLayers::layer(1),
                ));
            });
    });
}

/// Propagate RenderLayers to all descendants of the viewmodel.
///
/// When a glTF scene loads, it creates child entities (meshes, etc.) that don't
/// inherit RenderLayers. This system adds layer 1 to all descendants so they
/// render on the weapon camera.
fn propagate_viewmodel_render_layers(
    mut commands: Commands,
    viewmodel_query: Query<Entity, With<WeaponViewmodel>>,
    children_query: Query<&Children>,
    render_layers_query: Query<&RenderLayers>,
) {
    let Ok(viewmodel_entity) = viewmodel_query.get_single() else {
        return;
    };

    // Recursively collect all descendants
    let mut to_process = vec![viewmodel_entity];
    while let Some(entity) = to_process.pop() {
        // Add RenderLayers if missing
        if render_layers_query.get(entity).is_err() {
            commands.entity(entity).insert(RenderLayers::layer(1));
        }

        // Queue children for processing
        if let Ok(children) = children_query.get(entity) {
            to_process.extend(children.iter());
        }
    }
}

/// Update viewmodel position based on combat state.
///
/// Since the viewmodel is parented to the camera, we only need to adjust
/// the local offset for combat states (blocking, attacking).
fn update_viewmodel_position(
    combat_query: Query<&CombatState, With<Player>>,
    mut viewmodel_query: Query<&mut SmoothTransform, With<WeaponViewmodel>>,
) {
    let Ok(combat) = combat_query.get_single() else {
        return;
    };
    let Ok(mut smooth) = viewmodel_query.get_single_mut() else {
        return;
    };

    // Base position (local to camera)
    let offset = if combat.is_blocking {
        // Raise weapon for blocking stance
        Vec3::new(0.1, 0.0, -0.4)
    } else if combat.is_attacking {
        // Thrust forward during attack
        Vec3::new(0.2, -0.1, -0.7)
    } else {
        // Default idle position
        Vec3::new(0.3, -0.2, -0.5)
    };

    smooth.target_translation = Some(offset);
}

/// Animate viewmodel based on combat state.
///
/// Sets target rotation for smooth interpolation, then applies idle bob
/// additively to the current transform.
fn update_viewmodel_animation(
    time: Res<Time>,
    combat_query: Query<&CombatState, With<Player>>,
    mut viewmodel_query: Query<(&mut Transform, &mut SmoothTransform), With<WeaponViewmodel>>,
) {
    let Ok(combat) = combat_query.get_single() else {
        return;
    };
    let Ok((mut transform, mut smooth)) = viewmodel_query.get_single_mut() else {
        return;
    };

    // Determine base rotation based on combat state
    let base_rotation = if combat.is_blocking {
        // Horizontal blocking position
        Quat::from_euler(EulerRot::XYZ, -0.3, 0.0, 1.2)
    } else if combat.is_attacking {
        // Swing forward
        Quat::from_euler(EulerRot::XYZ, -0.8, -0.3, 0.0)
    } else {
        // Idle base rotation (identity)
        Quat::IDENTITY
    };

    smooth.target_rotation = Some(base_rotation);

    // Apply idle bob additively (only when not in combat state)
    if !combat.is_blocking && !combat.is_attacking {
        let idle_bob = (time.elapsed_secs() * 2.0).sin() * 0.005;
        let idle_sway = (time.elapsed_secs() * 1.5).cos() * 0.003;
        let idle_rotation = Quat::from_euler(EulerRot::XYZ, idle_bob, idle_sway, 0.0);
        transform.rotation = transform.rotation * idle_rotation;
    }
}
