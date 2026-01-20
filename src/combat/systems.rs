//! Combat systems - attack, block, damage handling.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::components::*;
use crate::core::{GameState, PlayState};
use crate::player::{Player, PlayerCamera};

/// System set ordering for combat.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CombatSet {
    Input,
    Action,
    Damage,
    Feedback,
}

/// Configure combat systems.
pub fn setup_combat_systems(app: &mut App) {
    app
        // Resources
        .init_resource::<ScreenShake>()
        .init_resource::<HitStop>()

        // Events
        .add_event::<AttackEvent>()
        .add_event::<DamageEvent>()
        .add_event::<DeathEvent>()

        // System ordering
        .configure_sets(
            Update,
            (
                CombatSet::Input,
                CombatSet::Action,
                CombatSet::Damage,
                CombatSet::Feedback,
            )
                .chain()
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(PlayState::Exploring)),
        )

        // Input systems
        .add_systems(
            Update,
            (
                combat_input,
                stamina_regen,
            )
                .in_set(CombatSet::Input),
        )

        // Action systems
        .add_systems(
            Update,
            (
                execute_attack,
                handle_blocking,
                update_cooldowns,
            )
                .in_set(CombatSet::Action),
        )

        // Damage systems
        .add_systems(
            Update,
            (
                process_attack_hits,
                apply_damage,
                check_deaths,
            )
                .in_set(CombatSet::Damage),
        )

        // Feedback systems
        .add_systems(
            Update,
            (
                update_screen_shake,
                update_hit_stop,
            )
                .in_set(CombatSet::Feedback),
        );
}

/// Handle combat input from the player.
fn combat_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut CombatState, &Stamina), With<Player>>,
    hit_stop: Res<HitStop>,
) {
    // Don't process input during hit stop
    if hit_stop.is_active() {
        return;
    }

    let Ok((mut combat, stamina)) = query.get_single_mut() else {
        return;
    };

    // Left click - light attack
    if mouse.just_pressed(MouseButton::Left) && combat.can_attack() && stamina.current > 0.0 {
        combat.is_attacking = true;
    }

    // Right click - block
    combat.is_blocking = mouse.pressed(MouseButton::Right) && combat.can_block();
}

/// Regenerate stamina over time.
fn stamina_regen(time: Res<Time>, mut query: Query<&mut Stamina>) {
    for mut stamina in query.iter_mut() {
        stamina.regenerate(time.delta_secs());
    }
}

/// Execute attack when attack animation triggers.
fn execute_attack(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut CombatState, &mut Stamina, &Weapon), With<Player>>,
    mut attack_events: EventWriter<AttackEvent>,
    rapier_context: Query<&RapierContext>,
) {
    let Ok((player_entity, transform, mut combat, mut stamina, weapon)) = query.get_single_mut()
    else {
        return;
    };

    if !combat.is_attacking {
        return;
    }

    // Only consume stamina and do hit detection once per attack
    if combat.attack_executed {
        return;
    }

    // Check stamina
    if !stamina.use_stamina(weapon.stamina_cost) {
        combat.is_attacking = false;
        return;
    }

    // Mark attack as executed so we don't consume stamina again
    combat.attack_executed = true;

    let damage = weapon.base_damage;

    // Get attack direction (forward)
    let direction = transform.forward().as_vec3();

    // Send attack event
    attack_events.send(AttackEvent {
        attacker: player_entity,
        damage,
        element: weapon.element,
        direction,
    });

    // Raycast for hit detection
    if let Ok(context) = rapier_context.get_single() {
        let ray_origin = transform.translation + Vec3::Y * 0.5; // Eye level
        let ray_dir = direction;
        let max_dist = weapon.reach;

        if let Some((hit_entity, _toi)) = context.cast_ray(
            ray_origin,
            ray_dir,
            max_dist,
            true,
            QueryFilter::default().exclude_collider(player_entity),
        ) {
            // We hit something - spawn a damage event
            commands.send_event(DamageEvent {
                target: hit_entity,
                source: player_entity,
                amount: damage,
                element: weapon.element,
                knockback: direction * 2.0,
            });
        }
    }

    // Set cooldown - is_attacking will be reset by update_cooldowns
    // when cooldown drops below half (giving time for attack animation)
    combat.attack_cooldown = weapon.attack_cooldown;
}

/// Handle blocking state.
fn handle_blocking(
    mut query: Query<(&CombatState, &mut Stamina), With<Player>>,
    time: Res<Time>,
) {
    for (combat, mut stamina) in query.iter_mut() {
        if combat.is_blocking {
            // Blocking drains stamina slowly
            stamina.current = (stamina.current - 5.0 * time.delta_secs()).max(0.0);
            stamina.regen_timer = stamina.regen_delay;
        }
    }
}

/// Update combat cooldowns.
fn update_cooldowns(time: Res<Time>, mut query: Query<(&mut CombatState, &Weapon)>) {
    for (mut combat, weapon) in query.iter_mut() {
        if combat.attack_cooldown > 0.0 {
            combat.attack_cooldown -= time.delta_secs();

            // Reset is_attacking after the attack animation portion (first 60% of cooldown)
            // This gives visual feedback while still preventing spam attacks
            let attack_anim_threshold = weapon.attack_cooldown * 0.4;
            if combat.is_attacking && combat.attack_cooldown <= attack_anim_threshold {
                combat.is_attacking = false;
                combat.attack_executed = false;
            }
        }
        if combat.i_frames > 0.0 {
            combat.i_frames -= time.delta_secs();
        }
    }
}

/// Process hits from attacks.
fn process_attack_hits(
    mut damage_events: EventReader<DamageEvent>,
    mut screen_shake: ResMut<ScreenShake>,
    mut hit_stop: ResMut<HitStop>,
) {
    for _event in damage_events.read() {
        // Trigger combat feedback
        screen_shake.shake(0.1, 0.15);
        hit_stop.trigger(0.05);
    }
}

/// Apply damage to entities.
fn apply_damage(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    mut health_query: Query<(&mut Health, Option<&Resistances>, Option<&CombatState>, Option<&Dead>)>,
    mut death_events: EventWriter<DeathEvent>,
) {
    // Track entities that died this frame to avoid duplicate death events
    let mut died_this_frame = std::collections::HashSet::new();

    for event in damage_events.read() {
        // Skip if already processed death this frame
        if died_this_frame.contains(&event.target) {
            continue;
        }

        if let Ok((mut health, resistances, combat_state, dead)) = health_query.get_mut(event.target) {
            // Skip if already dead (from previous frames)
            if dead.is_some() {
                continue;
            }

            // Check for i-frames
            if let Some(combat) = combat_state {
                if combat.i_frames > 0.0 {
                    continue;
                }
            }

            // Calculate resistance
            let resistance = resistances.map_or(0.0, |r| r.get(event.element));

            // Check for blocking (reduces damage further)
            let block_reduction = if let Some(combat) = combat_state {
                if combat.is_blocking {
                    0.5 // 50% reduction when blocking
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let final_damage = event.amount * (1.0 - resistance) * (1.0 - block_reduction);
            health.take_damage(final_damage);

            if health.is_dead() {
                // Mark as dead to prevent multiple death events
                died_this_frame.insert(event.target);
                commands.entity(event.target).insert(Dead);
                death_events.send(DeathEvent {
                    entity: event.target,
                    killed_by: Some(event.source),
                });
            }
        }
    }
}

/// Check for entity deaths.
fn check_deaths(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    player_query: Query<Entity, With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in death_events.read() {
        // Check if player died
        if player_query.get(event.entity).is_ok() {
            info!("Player died! Transitioning to Game Over...");
            next_state.set(GameState::GameOver);
        } else {
            // Non-player entity died - despawn
            commands.entity(event.entity).despawn_recursive();
        }
    }
}

/// Update screen shake effect.
fn update_screen_shake(
    time: Res<Time>,
    mut screen_shake: ResMut<ScreenShake>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
) {
    let offset = screen_shake.update(time.delta_secs());

    if let Ok(_transform) = camera_query.get_single() {
        // Apply shake offset to camera
        // Note: This is additive to the base position, so we need to
        // store the original position or apply it differently
        // For simplicity, we'll apply it as a rotation wobble
        if offset != Vec3::ZERO {
            let _shake_rotation =
                Quat::from_euler(EulerRot::XYZ, offset.y * 0.1, offset.x * 0.1, 0.0);
            // We need to preserve the existing pitch, so this is simplified
            // In a full implementation, you'd separate shake from look rotation
        }
    }
}

/// Update hit stop effect.
fn update_hit_stop(time: Res<Time>, mut hit_stop: ResMut<HitStop>) {
    hit_stop.update(time.delta_secs());
}
