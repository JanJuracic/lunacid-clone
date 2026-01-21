//! World plugin - level loading, environment, and interactables.

use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

use crate::core::GameState;
use crate::enemies::SpawnZone;
use crate::player::spawn_player;

/// World plugin - handles level loading and world setup.
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_dungeon)
            .add_systems(OnExit(GameState::InGame), cleanup_level);
    }
}

/// Marker for all level geometry that should be cleaned up.
#[derive(Component)]
struct LevelGeometry;

/// Materials used throughout the dungeon.
struct DungeonMaterials {
    floor: Handle<StandardMaterial>,
    wall: Handle<StandardMaterial>,
    ceiling: Handle<StandardMaterial>,
    pillar: Handle<StandardMaterial>,
    floor_alt: Handle<StandardMaterial>,
}

/// Constants for dungeon construction.
const WALL_HEIGHT: f32 = 4.0;
const WALL_THICKNESS: f32 = 0.5;
const DOOR_WIDTH: f32 = 2.5;

/// Set up the dungeon level with multiple rooms.
pub fn setup_dungeon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn player at the center of the starting room
    spawn_player(&mut commands, Vec3::new(0.0, 1.0, 0.0));

    // Create materials
    let mats = DungeonMaterials {
        floor: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35),
            perceptual_roughness: 0.9,
            ..default()
        }),
        wall: materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.35, 0.3),
            perceptual_roughness: 0.8,
            ..default()
        }),
        ceiling: materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.3),
            perceptual_roughness: 0.9,
            ..default()
        }),
        pillar: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.45, 0.4),
            perceptual_roughness: 0.7,
            ..default()
        }),
        floor_alt: materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.3, 0.3),
            perceptual_roughness: 0.9,
            ..default()
        }),
    };

    // Ambient light (dim dungeon atmosphere)
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.4, 0.4, 0.5),
        brightness: 30.0,
    });

    // =========================================================================
    // ROOM 1: Central Hall (Player Spawn) - 10x10
    // =========================================================================
    let center = Vec3::ZERO;
    spawn_floor(&mut commands, &mut meshes, &mats, center, 10.0, 10.0);
    spawn_ceiling(&mut commands, &mut meshes, &mats, center, 10.0, 10.0);

    // North wall with doorway
    spawn_wall_with_doorway(
        &mut commands, &mut meshes, &mats,
        center + Vec3::new(0.0, 0.0, -5.0),
        10.0, Direction::North,
    );
    // South wall (solid)
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        center + Vec3::new(0.0, WALL_HEIGHT / 2.0, 5.0),
        Vec3::new(10.0, WALL_HEIGHT, WALL_THICKNESS));
    // East wall with doorway
    spawn_wall_with_doorway(
        &mut commands, &mut meshes, &mats,
        center + Vec3::new(5.0, 0.0, 0.0),
        10.0, Direction::East,
    );
    // West wall with doorway
    spawn_wall_with_doorway(
        &mut commands, &mut meshes, &mats,
        center + Vec3::new(-5.0, 0.0, 0.0),
        10.0, Direction::West,
    );

    // Pillars in corners
    for pos in [
        center + Vec3::new(-3.0, WALL_HEIGHT / 2.0, -3.0),
        center + Vec3::new(3.0, WALL_HEIGHT / 2.0, -3.0),
        center + Vec3::new(-3.0, WALL_HEIGHT / 2.0, 3.0),
        center + Vec3::new(3.0, WALL_HEIGHT / 2.0, 3.0),
    ] {
        spawn_pillar(&mut commands, &mut meshes, &mats, pos);
    }

    // Central chandelier light
    spawn_light(&mut commands, center + Vec3::new(0.0, 3.0, 0.0), 120000.0, true);

    // =========================================================================
    // ROOM 2: North Corridor - 4x8 leading to Great Hall
    // =========================================================================
    let north_corridor = center + Vec3::new(0.0, 0.0, -9.0);
    spawn_floor(&mut commands, &mut meshes, &mats, north_corridor, 4.0, 8.0);
    spawn_ceiling(&mut commands, &mut meshes, &mats, north_corridor, 4.0, 8.0);

    // Corridor walls (east and west)
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        north_corridor + Vec3::new(2.0, WALL_HEIGHT / 2.0, 0.0),
        Vec3::new(WALL_THICKNESS, WALL_HEIGHT, 8.0));
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        north_corridor + Vec3::new(-2.0, WALL_HEIGHT / 2.0, 0.0),
        Vec3::new(WALL_THICKNESS, WALL_HEIGHT, 8.0));

    spawn_light(&mut commands, north_corridor + Vec3::new(0.0, 3.0, 0.0), 40000.0, false);

    // =========================================================================
    // ROOM 3: Great Hall (North) - 14x12
    // =========================================================================
    let great_hall = center + Vec3::new(0.0, 0.0, -19.0);
    spawn_floor(&mut commands, &mut meshes, &mats, great_hall, 14.0, 12.0);
    spawn_ceiling(&mut commands, &mut meshes, &mats, great_hall, 14.0, 12.0);

    // North wall (back of great hall)
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        great_hall + Vec3::new(0.0, WALL_HEIGHT / 2.0, -6.0),
        Vec3::new(14.0, WALL_HEIGHT, WALL_THICKNESS));
    // South wall with doorway (connects to corridor)
    spawn_wall_with_doorway(
        &mut commands, &mut meshes, &mats,
        great_hall + Vec3::new(0.0, 0.0, 6.0),
        14.0, Direction::South,
    );
    // East wall
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        great_hall + Vec3::new(7.0, WALL_HEIGHT / 2.0, 0.0),
        Vec3::new(WALL_THICKNESS, WALL_HEIGHT, 12.0));
    // West wall
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        great_hall + Vec3::new(-7.0, WALL_HEIGHT / 2.0, 0.0),
        Vec3::new(WALL_THICKNESS, WALL_HEIGHT, 12.0));

    // Pillars in great hall
    for x in [-4.0, 4.0] {
        for z in [-3.0, 3.0] {
            spawn_pillar(&mut commands, &mut meshes, &mats,
                great_hall + Vec3::new(x, WALL_HEIGHT / 2.0, z));
        }
    }

    // Lights in great hall
    spawn_light(&mut commands, great_hall + Vec3::new(0.0, 3.0, 0.0), 100000.0, true);
    spawn_light(&mut commands, great_hall + Vec3::new(-4.0, 2.5, -3.0), 30000.0, false);
    spawn_light(&mut commands, great_hall + Vec3::new(4.0, 2.5, -3.0), 30000.0, false);

    // Spawn zone for orcs in great hall
    commands.spawn((
        SpawnZone {
            enemy_type: "orc".to_string(),
            half_extents: Vec3::new(4.0, 0.0, 4.0),
            max_enemies: 2,
            spawn_delay: 0.0,
        },
        Transform::from_translation(great_hall),
        LevelGeometry,
    ));

    // =========================================================================
    // ROOM 4: East Chamber - 8x8
    // =========================================================================
    let east_room = center + Vec3::new(9.0, 0.0, 0.0);
    spawn_floor(&mut commands, &mut meshes, &mats, east_room, 8.0, 8.0);
    spawn_ceiling(&mut commands, &mut meshes, &mats, east_room, 8.0, 8.0);

    // North wall
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        east_room + Vec3::new(0.0, WALL_HEIGHT / 2.0, -4.0),
        Vec3::new(8.0, WALL_HEIGHT, WALL_THICKNESS));
    // South wall
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        east_room + Vec3::new(0.0, WALL_HEIGHT / 2.0, 4.0),
        Vec3::new(8.0, WALL_HEIGHT, WALL_THICKNESS));
    // East wall
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        east_room + Vec3::new(4.0, WALL_HEIGHT / 2.0, 0.0),
        Vec3::new(WALL_THICKNESS, WALL_HEIGHT, 8.0));
    // West wall with doorway (connects to center)
    spawn_wall_with_doorway(
        &mut commands, &mut meshes, &mats,
        east_room + Vec3::new(-4.0, 0.0, 0.0),
        8.0, Direction::West,
    );

    spawn_light(&mut commands, east_room + Vec3::new(0.0, 3.0, 0.0), 60000.0, true);

    // Spawn zone for orcs in east chamber
    commands.spawn((
        SpawnZone {
            enemy_type: "orc".to_string(),
            half_extents: Vec3::new(2.5, 0.0, 2.5),
            max_enemies: 1,
            spawn_delay: 0.0,
        },
        Transform::from_translation(east_room),
        LevelGeometry,
    ));

    // =========================================================================
    // ROOM 5: West Chamber - 8x10
    // =========================================================================
    let west_room = center + Vec3::new(-9.0, 0.0, 0.0);
    spawn_floor(&mut commands, &mut meshes, &mats, west_room, 8.0, 10.0);
    spawn_ceiling(&mut commands, &mut meshes, &mats, west_room, 8.0, 10.0);

    // North wall
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        west_room + Vec3::new(0.0, WALL_HEIGHT / 2.0, -5.0),
        Vec3::new(8.0, WALL_HEIGHT, WALL_THICKNESS));
    // South wall
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        west_room + Vec3::new(0.0, WALL_HEIGHT / 2.0, 5.0),
        Vec3::new(8.0, WALL_HEIGHT, WALL_THICKNESS));
    // West wall
    spawn_wall(&mut commands, &mut meshes, mats.wall.clone(),
        west_room + Vec3::new(-4.0, WALL_HEIGHT / 2.0, 0.0),
        Vec3::new(WALL_THICKNESS, WALL_HEIGHT, 10.0));
    // East wall with doorway (connects to center)
    spawn_wall_with_doorway(
        &mut commands, &mut meshes, &mats,
        west_room + Vec3::new(4.0, 0.0, 0.0),
        10.0, Direction::East,
    );

    // Pillars in west room
    spawn_pillar(&mut commands, &mut meshes, &mats,
        west_room + Vec3::new(0.0, WALL_HEIGHT / 2.0, -2.5));
    spawn_pillar(&mut commands, &mut meshes, &mats,
        west_room + Vec3::new(0.0, WALL_HEIGHT / 2.0, 2.5));

    spawn_light(&mut commands, west_room + Vec3::new(0.0, 3.0, 0.0), 60000.0, true);
    spawn_light(&mut commands, west_room + Vec3::new(0.0, 2.5, -2.5), 20000.0, false);
    spawn_light(&mut commands, west_room + Vec3::new(0.0, 2.5, 2.5), 20000.0, false);

    // Spawn zone for orcs in west chamber
    commands.spawn((
        SpawnZone {
            enemy_type: "orc".to_string(),
            half_extents: Vec3::new(2.5, 0.0, 3.0),
            max_enemies: 1,
            spawn_delay: 0.0,
        },
        Transform::from_translation(west_room),
        LevelGeometry,
    ));
}

/// Direction for doorways.
#[derive(Clone, Copy)]
enum Direction {
    North,
    South,
    East,
    West,
}

/// Spawn a floor plane.
fn spawn_floor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &DungeonMaterials,
    center: Vec3,
    width: f32,
    depth: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(width, depth))),
        MeshMaterial3d(mats.floor.clone()),
        Transform::from_translation(center),
        Collider::cuboid(width / 2.0, 0.1, depth / 2.0),
        LevelGeometry,
    ));
}

/// Spawn a ceiling plane.
fn spawn_ceiling(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &DungeonMaterials,
    center: Vec3,
    width: f32,
    depth: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(width, depth))),
        MeshMaterial3d(mats.ceiling.clone()),
        Transform::from_xyz(center.x, WALL_HEIGHT, center.z)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
        LevelGeometry,
    ));
}

/// Spawn a solid wall.
fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
        LevelGeometry,
    ));
}

/// Spawn a wall with a doorway in the center.
fn spawn_wall_with_doorway(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &DungeonMaterials,
    base_position: Vec3,
    wall_length: f32,
    direction: Direction,
) {
    let segment_length = (wall_length - DOOR_WIDTH) / 2.0;
    let half_segment = segment_length / 2.0;
    let offset = DOOR_WIDTH / 2.0 + half_segment;

    match direction {
        Direction::North | Direction::South => {
            // Horizontal wall (along X axis) with doorway
            let y = base_position.y + WALL_HEIGHT / 2.0;
            let z = base_position.z;

            // Left segment
            spawn_wall(commands, meshes, mats.wall.clone(),
                Vec3::new(base_position.x - offset, y, z),
                Vec3::new(segment_length, WALL_HEIGHT, WALL_THICKNESS));
            // Right segment
            spawn_wall(commands, meshes, mats.wall.clone(),
                Vec3::new(base_position.x + offset, y, z),
                Vec3::new(segment_length, WALL_HEIGHT, WALL_THICKNESS));
        }
        Direction::East | Direction::West => {
            // Vertical wall (along Z axis) with doorway
            let y = base_position.y + WALL_HEIGHT / 2.0;
            let x = base_position.x;

            // Front segment
            spawn_wall(commands, meshes, mats.wall.clone(),
                Vec3::new(x, y, base_position.z - offset),
                Vec3::new(WALL_THICKNESS, WALL_HEIGHT, segment_length));
            // Back segment
            spawn_wall(commands, meshes, mats.wall.clone(),
                Vec3::new(x, y, base_position.z + offset),
                Vec3::new(WALL_THICKNESS, WALL_HEIGHT, segment_length));
        }
    }
}

/// Spawn a pillar.
fn spawn_pillar(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &DungeonMaterials,
    position: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.6, WALL_HEIGHT, 0.6))),
        MeshMaterial3d(mats.pillar.clone()),
        Transform::from_translation(position),
        Collider::cuboid(0.3, WALL_HEIGHT / 2.0, 0.3),
        LevelGeometry,
    ));
}

/// Spawn a point light.
fn spawn_light(
    commands: &mut Commands,
    position: Vec3,
    intensity: f32,
    shadows: bool,
) {
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.8, 0.6),
            intensity,
            range: 15.0,
            shadows_enabled: shadows,
            ..default()
        },
        Transform::from_translation(position),
        LevelGeometry,
    ));
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
