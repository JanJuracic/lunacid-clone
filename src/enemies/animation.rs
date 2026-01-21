//! Enemy animation systems.

use bevy::prelude::*;
use bevy::animation::{AnimationClip, AnimationPlayer, RepeatAnimation, graph::AnimationNodeIndex};

use super::components::{AiState, AttackReady, Enemy, EnemyType, EnemyStats};
use super::data::{AnimationConfig, EnemyRegistry};
use crate::combat::DamageEvent;

/// Visual animation state (separate from AI state for animation control).
#[derive(Component, Default, Clone, Copy, PartialEq, Debug)]
pub enum AnimationState {
    #[default]
    Idle,
    Walking,
    CombatIdle,
    Attacking,
    Hurt,
    Dying,
}

/// Stores animation graph handle and node indices for an enemy.
#[derive(Component)]
pub struct EnemyAnimations {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub combat_idle: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: Option<AnimationNodeIndex>,
    pub death: AnimationNodeIndex,
}

/// Links an enemy entity to its child AnimationPlayer entity.
#[derive(Component)]
pub struct AnimationLink(pub Entity);

/// Timer for one-shot animations to return to looping state.
#[derive(Component)]
pub struct OneShotTimer {
    pub timer: Timer,
    pub return_to: AnimationState,
}

/// Marker for enemies awaiting AnimationPlayer discovery.
#[derive(Component)]
pub struct NeedsAnimationSetup;

/// Tracks the previous animation state to detect transitions.
#[derive(Component, Default)]
pub struct PreviousAnimationState(pub AnimationState);

/// Tracks attack animation progress for hit detection.
#[derive(Component)]
pub struct AttackAnimationProgress {
    pub hit_fired: bool,
    pub hit_frame: f32,
}

/// Event sent when enemy attack animation reaches its hit frame.
#[derive(Event)]
pub struct AttackHitEvent {
    pub attacker: Entity,
    pub damage: f32,
}

/// Finds AnimationPlayer in scene hierarchy and builds AnimationGraph.
pub fn setup_enemy_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<EnemyRegistry>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    enemy_query: Query<(Entity, &EnemyType, &Children), (With<NeedsAnimationSetup>, With<Enemy>)>,
    children_query: Query<&Children>,
    mut animation_player_query: Query<(Entity, &mut AnimationPlayer)>,
) {
    for (enemy_entity, enemy_type, children) in enemy_query.iter() {
        // Find the AnimationPlayer in the hierarchy
        let Some(player_entity) = find_animation_player_entity(
            children,
            &children_query,
            &animation_player_query,
        ) else {
            continue;
        };

        // Get the animation config from the registry
        let Some(definition) = registry.get(&enemy_type.0) else {
            warn!("No definition found for enemy type: {}", enemy_type.0);
            commands.entity(enemy_entity).remove::<NeedsAnimationSetup>();
            continue;
        };

        let Some(ref anim_config) = definition.animations else {
            // No animations configured, just remove the marker
            commands.entity(enemy_entity).remove::<NeedsAnimationSetup>();
            continue;
        };

        // Build the animation graph
        let model_base = definition.model_path.replace("#Scene0", "");

        let (graph, node_indices) = build_animation_graph(
            &asset_server,
            &model_base,
            anim_config,
        );

        let graph_handle = graphs.add(graph);

        // Add the graph to the animation player
        commands.entity(player_entity).insert(AnimationGraphHandle(graph_handle.clone()));

        let idle_node = node_indices.0;

        // Add components to the enemy entity
        commands.entity(enemy_entity)
            .remove::<NeedsAnimationSetup>()
            .insert((
                AnimationLink(player_entity),
                EnemyAnimations {
                    graph: graph_handle,
                    idle: idle_node,
                    walk: node_indices.1,
                    combat_idle: node_indices.2,
                    attack: node_indices.3,
                    hurt: node_indices.4,
                    death: node_indices.5,
                },
                AnimationState::Idle,
                PreviousAnimationState::default(),
            ));

        // Immediately start playing the idle animation to avoid first-frame issues
        // with Changed<AnimationState> detection
        if let Ok((_, mut player)) = animation_player_query.get_mut(player_entity) {
            player.stop_all();
            player.start(idle_node).set_repeat(RepeatAnimation::Forever);
        }

        info!("Animation setup complete for enemy: {}", definition.name);
    }
}

/// Recursively search for AnimationPlayer entity in hierarchy.
fn find_animation_player_entity(
    children: &Children,
    children_query: &Query<&Children>,
    animation_player_query: &Query<(Entity, &mut AnimationPlayer)>,
) -> Option<Entity> {
    for &child in children.iter() {
        if animation_player_query.get(child).is_ok() {
            return Some(child);
        }

        if let Ok(grandchildren) = children_query.get(child) {
            if let Some(found) = find_animation_player_entity(grandchildren, children_query, animation_player_query) {
                return Some(found);
            }
        }
    }
    None
}

/// Build animation graph from config.
fn build_animation_graph(
    asset_server: &AssetServer,
    model_base: &str,
    config: &AnimationConfig,
) -> (AnimationGraph, (AnimationNodeIndex, AnimationNodeIndex, AnimationNodeIndex, AnimationNodeIndex, Option<AnimationNodeIndex>, AnimationNodeIndex)) {
    let mut graph = AnimationGraph::new();

    // Load animation clips
    let idle_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation{}", model_base, config.indices.idle));
    let walk_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation{}", model_base, config.indices.walk));
    let combat_idle_idx = config.indices.combat_idle.unwrap_or(config.indices.idle);
    let combat_idle_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation{}", model_base, combat_idle_idx));
    let attack_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation{}", model_base, config.indices.attack));
    let death_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation{}", model_base, config.indices.death));

    // Add nodes to graph
    let idle_node = graph.add_clip(idle_clip, 1.0, graph.root);
    let walk_node = graph.add_clip(walk_clip, 1.0, graph.root);
    let combat_idle_node = graph.add_clip(combat_idle_clip, 1.0, graph.root);
    let attack_node = graph.add_clip(attack_clip, 1.0, graph.root);
    let death_node = graph.add_clip(death_clip, 1.0, graph.root);

    // Hurt is optional
    let hurt_node = config.indices.hurt.map(|idx| {
        let hurt_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation{}", model_base, idx));
        graph.add_clip(hurt_clip, 1.0, graph.root)
    });

    (graph, (idle_node, walk_node, combat_idle_node, attack_node, hurt_node, death_node))
}

/// Maps AiState + context to AnimationState.
pub fn sync_animation_state(
    mut query: Query<
        (&AiState, &mut AnimationState, &EnemyStats, &Transform),
        (With<Enemy>, With<EnemyAnimations>, Without<OneShotTimer>),
    >,
    player_query: Query<&Transform, (With<crate::player::Player>, Without<Enemy>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for (ai_state, mut anim_state, stats, enemy_transform) in query.iter_mut() {
        // Don't change animation state if dying
        if *anim_state == AnimationState::Dying {
            continue;
        }

        let new_state = match ai_state {
            AiState::Idle => AnimationState::Idle,
            AiState::Chasing => AnimationState::Walking,
            AiState::Attacking => {
                // Check if in attack range for combat idle vs attacking
                let distance = enemy_transform.translation.distance(player_transform.translation);
                if distance <= stats.attack_range {
                    // The actual attack animation is triggered separately
                    AnimationState::CombatIdle
                } else {
                    AnimationState::Walking
                }
            }
            AiState::Dying => AnimationState::Dying,
        };

        if *anim_state != new_state && *anim_state != AnimationState::Attacking && *anim_state != AnimationState::Hurt {
            *anim_state = new_state;
        }
    }
}

/// Listens to DamageEvent and triggers hurt animation.
pub fn trigger_hurt_animation(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    mut query: Query<(Entity, &mut AnimationState, &EnemyType), With<Enemy>>,
    registry: Res<EnemyRegistry>,
) {
    for event in damage_events.read() {
        if let Ok((entity, mut anim_state, enemy_type)) = query.get_mut(event.target) {
            // Don't interrupt dying
            if *anim_state == AnimationState::Dying {
                continue;
            }

            // Get hurt duration from config
            let hurt_duration = registry.get(&enemy_type.0)
                .and_then(|def| def.animations.as_ref())
                .map(|cfg| cfg.hurt_duration)
                .unwrap_or(0.4);

            // Determine what state to return to after hurt
            let return_to = match *anim_state {
                AnimationState::Attacking => AnimationState::CombatIdle,
                AnimationState::Walking => AnimationState::Walking,
                _ => AnimationState::Idle,
            };

            *anim_state = AnimationState::Hurt;

            commands.entity(entity).insert(OneShotTimer {
                timer: Timer::from_seconds(hurt_duration, TimerMode::Once),
                return_to,
            });
        }
    }
}

/// Triggers attack animation when AI enters attack state with cooldown ready.
pub fn trigger_attack_animation(
    mut commands: Commands,
    mut query: Query<
        (Entity, &AiState, &mut AnimationState, &EnemyType),
        (With<Enemy>, With<EnemyAnimations>, With<AttackReady>, Without<OneShotTimer>),
    >,
    registry: Res<EnemyRegistry>,
) {
    for (entity, ai_state, mut anim_state, enemy_type) in query.iter_mut() {
        // Only trigger attack animation when:
        // 1. AI is in attacking state
        // 2. Not already attacking or hurt
        // 3. AttackReady marker is present (set by AI system)
        if *ai_state != AiState::Attacking {
            // Remove AttackReady if no longer attacking
            commands.entity(entity).remove::<AttackReady>();
            continue;
        }

        if *anim_state == AnimationState::Attacking || *anim_state == AnimationState::Hurt || *anim_state == AnimationState::Dying {
            commands.entity(entity).remove::<AttackReady>();
            continue;
        }

        // Get attack hit frame from config
        let hit_frame = registry.get(&enemy_type.0)
            .and_then(|def| def.animations.as_ref())
            .map(|cfg| cfg.attack_hit_frame)
            .unwrap_or(0.5);

        *anim_state = AnimationState::Attacking;

        // Use attack cooldown as animation duration estimate
        let attack_duration = registry.get(&enemy_type.0)
            .map(|def| def.attack_cooldown * 0.6) // Animation is shorter than full cooldown
            .unwrap_or(0.6);

        // Remove AttackReady marker and add animation components
        commands.entity(entity)
            .remove::<AttackReady>()
            .insert((
                OneShotTimer {
                    timer: Timer::from_seconds(attack_duration, TimerMode::Once),
                    return_to: AnimationState::CombatIdle,
                },
                AttackAnimationProgress {
                    hit_fired: false,
                    hit_frame,
                },
            ));
    }
}

/// Applies AnimationState changes to AnimationPlayer.
pub fn play_animations(
    mut query: Query<
        (&AnimationState, &PreviousAnimationState, &AnimationLink, &EnemyAnimations),
        Changed<AnimationState>,
    >,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (anim_state, prev_state, link, animations) in query.iter_mut() {
        // Only play if state changed
        if *anim_state == prev_state.0 {
            continue;
        }

        let Ok(mut player) = animation_players.get_mut(link.0) else {
            continue;
        };

        let (node, is_looping) = match anim_state {
            AnimationState::Idle => (animations.idle, true),
            AnimationState::Walking => (animations.walk, true),
            AnimationState::CombatIdle => (animations.combat_idle, true),
            AnimationState::Attacking => (animations.attack, false),
            AnimationState::Hurt => {
                if let Some(hurt_node) = animations.hurt {
                    (hurt_node, false)
                } else {
                    continue; // No hurt animation, skip
                }
            }
            AnimationState::Dying => (animations.death, false),
        };

        // Start the animation
        player.stop_all();
        let anim = player.start(node);

        if is_looping {
            anim.set_repeat(RepeatAnimation::Forever);
        } else {
            anim.set_repeat(RepeatAnimation::Never);
        }
    }
}

/// Updates PreviousAnimationState after animations are played.
pub fn update_previous_animation_state(
    mut query: Query<(&AnimationState, &mut PreviousAnimationState), Changed<AnimationState>>,
) {
    for (current, mut previous) in query.iter_mut() {
        previous.0 = *current;
    }
}

/// Handles one-shot animation completion and returns to looping state.
pub fn update_oneshot_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut OneShotTimer, &mut AnimationState)>,
) {
    for (entity, mut oneshot, mut anim_state) in query.iter_mut() {
        oneshot.timer.tick(time.delta());

        if oneshot.timer.finished() {
            *anim_state = oneshot.return_to;
            commands.entity(entity).remove::<OneShotTimer>();
            commands.entity(entity).remove::<AttackAnimationProgress>();
        }
    }
}

/// Fires AttackHitEvent when attack animation reaches configured hit frame.
pub fn detect_attack_hit(
    mut query: Query<(Entity, &mut AttackAnimationProgress, &OneShotTimer, &EnemyStats)>,
    mut attack_hit_events: EventWriter<AttackHitEvent>,
) {
    for (entity, mut progress, oneshot, stats) in query.iter_mut() {
        if progress.hit_fired {
            continue;
        }

        // Calculate animation progress (0.0 to 1.0)
        let anim_progress = oneshot.timer.elapsed_secs() / oneshot.timer.duration().as_secs_f32();

        if anim_progress >= progress.hit_frame {
            progress.hit_fired = true;
            attack_hit_events.send(AttackHitEvent {
                attacker: entity,
                damage: stats.damage,
            });
        }
    }
}

/// Triggers death animation when AI enters dying state.
pub fn trigger_death_animation(
    mut query: Query<
        (&AiState, &mut AnimationState),
        (With<Enemy>, With<EnemyAnimations>, Changed<AiState>),
    >,
) {
    for (ai_state, mut anim_state) in query.iter_mut() {
        if *ai_state == AiState::Dying && *anim_state != AnimationState::Dying {
            *anim_state = AnimationState::Dying;
        }
    }
}
