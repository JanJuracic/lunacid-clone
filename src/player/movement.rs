//! First-person player movement and camera control.

use bevy::prelude::*;
use bevy::core_pipeline::core_3d::Camera3dDepthLoadOp;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::FogFalloff;
use bevy::render::camera::ClearColorConfig;
use bevy::render::view::RenderLayers;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;

use super::components::*;
use crate::combat::{create_starter_weapon, CombatState, Health, Resistances, Stamina};
use crate::core::{GameState, PlayState};
use crate::rendering::{PostProcessSettings, VisualConfig};

/// Marker component for the player's camera.
#[derive(Component)]
pub struct PlayerCamera {
    /// Current pitch angle in radians (looking up/down)
    pub pitch: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self { pitch: 0.0 }
    }
}

/// Marker for the weapon-only camera (renders viewmodel on separate layer).
#[derive(Component)]
pub struct WeaponCamera;

/// Set up player movement systems.
pub fn setup_movement_systems(app: &mut App) {
    app
        .init_resource::<PlayerConfig>()
        .add_systems(OnEnter(GameState::InGame), grab_cursor)
        .add_systems(OnExit(GameState::InGame), release_cursor)
        .add_systems(
            Update,
            (
                mouse_look,
                player_movement,
            )
            .run_if(in_state(GameState::InGame))
            .run_if(in_state(PlayState::Exploring))
        );
}

/// Grab and hide cursor when entering gameplay.
fn grab_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

/// Release cursor when leaving gameplay.
fn release_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

/// Handle mouse movement for looking around.
///
/// Rotates the player entity horizontally (yaw) and the camera vertically (pitch).
/// The camera is a child of the player, so horizontal rotation affects both.
pub fn mouse_look(
    mut mouse_motion: EventReader<MouseMotion>,
    config: Res<PlayerConfig>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), (With<Camera3d>, Without<Player>)>,
) {
    // Accumulate mouse movement
    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    // Get player and camera transforms
    let Ok(mut player_transform) = player_query.get_single_mut() else {
        return;
    };
    let Ok((mut camera_transform, mut camera)) = camera_query.get_single_mut() else {
        return;
    };

    let sensitivity = config.mouse_sensitivity * 0.001;
    let y_invert = if config.invert_y { -1.0 } else { 1.0 };

    // Rotate player horizontally (yaw)
    player_transform.rotate_y(-delta.x * sensitivity);

    // Rotate camera vertically (pitch), clamped to prevent flipping
    camera.pitch -= delta.y * sensitivity * y_invert;
    camera.pitch = camera.pitch.clamp(-1.4, 1.4); // About 80 degrees

    camera_transform.rotation = Quat::from_rotation_x(camera.pitch);
}

/// Handle WASD movement and jumping.
///
/// Uses Rapier's KinematicCharacterController for collision detection.
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<PlayerConfig>,
    rapier_context: Query<&RapierContext>,
    mut player_query: Query<(
        Entity,
        &Transform,
        &mut MovementState,
        &mut KinematicCharacterController,
    ), With<Player>>,
) {
    let Ok((player_entity, transform, mut movement_state, mut controller)) = player_query.get_single_mut() else {
        return;
    };

    // Ground check using raycast (more reliable than KinematicCharacterControllerOutput)
    // Player capsule is capsule_y(0.5, 0.3), so bottom is 0.8 units below center
    let is_grounded = if let Ok(context) = rapier_context.get_single() {
        let ray_origin = transform.translation - Vec3::Y * 0.75; // Just above capsule bottom
        let ray_dir = Vec3::NEG_Y;
        let max_dist = 0.15; // Small distance to check for ground

        context.cast_ray(
            ray_origin,
            ray_dir,
            max_dist,
            true,
            QueryFilter::default().exclude_collider(player_entity),
        ).is_some()
    } else {
        // Fallback: assume grounded if no physics context
        true
    };
    movement_state.is_grounded = is_grounded;

    // Handle jumping
    if is_grounded {
        // Only reset velocity if we're actually falling/landed
        if movement_state.vertical_velocity < 0.0 {
            movement_state.vertical_velocity = 0.0;
        }
        if keyboard.just_pressed(KeyCode::Space) {
            movement_state.vertical_velocity = config.jump_force;
        }
    } else {
        // Apply gravity
        movement_state.vertical_velocity -= config.gravity * time.delta_secs();
    }

    // Build input direction from WASD
    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    // Normalize to prevent faster diagonal movement
    if direction != Vec3::ZERO {
        direction = direction.normalize();
    }

    // Rotate direction to face where player is looking (only horizontal)
    let yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
    let rotation = Quat::from_rotation_y(yaw);
    let movement = rotation * direction;

    // Apply sprint if shift is held
    let speed = if keyboard.pressed(KeyCode::ShiftLeft) {
        config.move_speed * config.sprint_multiplier
    } else {
        config.move_speed
    };

    // Calculate final translation
    let horizontal = movement * speed * time.delta_secs();
    let vertical = Vec3::new(0.0, movement_state.vertical_velocity * time.delta_secs(), 0.0);

    controller.translation = Some(horizontal + vertical);
}

/// Spawn the player entity with camera.
pub fn spawn_player(commands: &mut Commands, position: Vec3, visual_config: &VisualConfig) -> Entity {
    // Spawn player body
    let player = commands
        .spawn((
            Player,
            PlayerStats::default(),
            Attributes::default(),
            MovementState::default(),
            // Combat components
            Health::new(100.0),
            Stamina::default(),
            CombatState::default(),
            Resistances::default(),
            create_starter_weapon(),
            // Transform
            Transform::from_translation(position),
            GlobalTransform::default(),
            Visibility::default(),
            // Rapier physics components
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(0.5, 0.3),
            KinematicCharacterController {
                offset: CharacterLength::Absolute(0.01),
                // Enable automatic stair climbing
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(0.4),  // ~40cm step height
                    min_width: CharacterLength::Absolute(0.3),   // Minimum landing space
                    include_dynamic_bodies: false,
                }),
                // Slope handling
                max_slope_climb_angle: 45_f32.to_radians(),
                min_slope_slide_angle: 30_f32.to_radians(),
                // Snap to ground when going down slopes/stairs
                snap_to_ground: Some(CharacterLength::Absolute(0.5)),
                ..default()
            },
        ))
        .id();

    // Build fog settings from config
    let fog_falloff = if visual_config.fog_enabled {
        FogFalloff::ExponentialSquared { density: visual_config.fog_density }
    } else {
        FogFalloff::ExponentialSquared { density: 0.0 }
    };

    // Spawn camera as child of player
    commands.entity(player).with_children(|parent| {
        parent
            .spawn((
                Camera3d::default(),
                Camera {
                    // Clear color from config
                    clear_color: ClearColorConfig::Custom(Color::srgb(
                        visual_config.clear_color.0,
                        visual_config.clear_color.1,
                        visual_config.clear_color.2,
                    )),
                    ..default()
                },
                // Atmospheric fog from config
                DistanceFog {
                    color: Color::srgba(
                        visual_config.fog_color.0,
                        visual_config.fog_color.1,
                        visual_config.fog_color.2,
                        1.0,
                    ),
                    falloff: fog_falloff,
                    directional_light_color: Color::NONE,
                    directional_light_exponent: 8.0,
                },
                // Horror post-processing from config
                PostProcessSettings::from_config(visual_config),
                PlayerCamera::default(),
                // Position camera at "eye level" relative to player
                Transform::from_xyz(0.0, 0.4, 0.0),
                // Main camera renders world on layer 0
                RenderLayers::layer(0),
            ))
            .with_children(|camera_parent| {
                // Weapon camera renders viewmodel on layer 1
                camera_parent
                    .spawn((
                        WeaponCamera,
                        Camera3d {
                            depth_load_op: Camera3dDepthLoadOp::Clear(0.0),
                            ..default()
                        },
                        Camera {
                            order: 1,
                            clear_color: ClearColorConfig::None,
                            ..default()
                        },
                        Transform::default(),
                        RenderLayers::layer(1),
                    ))
                    .with_children(|weapon_camera| {
                        // Dedicated light for weapon viewmodel (no shadows from world geometry)
                        weapon_camera.spawn((
                            PointLight {
                                color: Color::srgb(1.0, 0.9, 0.8),
                                intensity: 100000.0,
                                range: 10.0,
                                shadows_enabled: false,
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.5, 0.5),
                            RenderLayers::layer(1),
                        ));
                    });
            });
    });

    player
}
