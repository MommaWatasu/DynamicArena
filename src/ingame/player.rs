use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::{character_def::CHARACTER_PROFILES, ingame::{InGame, Ground}, AppState, GameConfig};
use super::pose::*;

const LIMB_LENGTH: f32 = 30.0;
const LIMB_RADIUS: f32 = 15.0;

const HEAD_OFFSET: f32 = 100.0;
const BODY_OFFSET: f32 = 40.0;
const UPPER_ARM_OFFSET: f32 = 0.0;
const LOWER_ARM_OFFSET: f32 = -60.0;
const UPPER_LEG_OFFSET: f32 = -100.0;
const LOWER_LEG_OFFSET: f32 = -60.0;

const PIXELS_PER_METER: f32 = 100.0;
const GRAVITY_ACCEL: f32 = 9.80665;
const TERMINATE_VELOCITY: f32 = 5.0;

const FPS: f32 = 60.0;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct PlayerID(pub u8);

#[derive(Component)]
pub struct HealthBar(pub f32);

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum PlayerState {
    #[default]
    Idle,
    Running,
    Jumping,
}

#[derive(Resource)]
struct AnimationTimer {
    timer: Timer
}

struct PlayerAnimation {
    diff_pose: Pose,
    phase: u8,
    count: u8,
}

#[derive(Component)]
struct Player {
    pose: Pose,
    animation: PlayerAnimation,
    state: PlayerState,
    velocity: Vec2,
    health: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pose: IDLE_POSE1,
            animation: PlayerAnimation { diff_pose: default(), phase: 1, count: 10 },
            state: PlayerState::default(),
            velocity: Vec2::ZERO,
            health: 100,
        }
    }
}

#[derive(Component)]
struct BodyParts {
    flags: u8
}

#[allow(dead_code)]
impl BodyParts {
    const HEAD: Self = Self { flags: 0b10000 };
    const BODY: Self = Self { flags: 0b01000 };
    pub fn new(head: bool, body: bool, arm: bool, right: bool, upper: bool) -> Self {
        Self {
            flags: (head as u8) << 4 | (body as u8) << 3 | (arm as u8) << 2 | (right as u8) << 1 | (upper as u8)
        }
    }
    pub fn is_head(&self) -> bool {
        self.flags & 0b10000 != 0
    }
    pub fn is_body(&self) -> bool {
        self.flags & 0b01000 != 0
    }
    pub fn is_arm(&self) -> bool {
        self.flags & 0b00100 != 0
    }
    pub fn is_right(&self) -> bool {
        self.flags & 0b00010 != 0
    }
    pub fn is_upper(&self) -> bool {
        self.flags & 0b00001 != 0
    }
}

pub fn spawn_player(
    id: u8,
    character_id: isize,
    builder: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    y_pos: f32,
) {
    let profile = &CHARACTER_PROFILES[character_id as usize];
    builder.spawn((
        Player::default(),
        PlayerID(id),
        InGame,
        // Player 0 is on top of the screen
        Transform::from_translation(Vec3::new(if id == 0 {-500.0} else {500.0}, y_pos, if id == 0 { 10.0 } else {1.0})),
        Visibility::Visible,
    ))
        // Body
        .with_children(|builder| {
            builder.spawn((
                Mesh2d(meshes.add(Capsule2d {
                    radius: LIMB_RADIUS,
                    half_length: 60.0,
                })),
                MeshMaterial2d(materials.add(profile.color)),
                Transform::default(),
                BodyParts::BODY,
                PlayerID(id),
                Collider::capsule_y(60.0, LIMB_RADIUS),
                RigidBody::KinematicPositionBased,
            ))
                // Head
                .with_children(|builder| {
                    builder.spawn((
                        Mesh2d(meshes.add(Circle::new(45.0))),
                        MeshMaterial2d(materials.add(profile.color)),
                        BodyParts::HEAD,
                        Transform::from_translation(Vec3::new(0.0, 100.0, 1.0)),
                        RigidBody::KinematicPositionBased,
                        Collider::ball(45.0),
                    ));
                    // Right Upper Arm
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(profile.color)),
                        BodyParts::new(false, false, true, true, true),
                        PlayerID(id),
                        // player 0 is right facing, and player 1 is left facing
                        // so we need to change which arm is on top
                        Transform::from_translation(Vec3::new(0.0, 0.0, if id == 0 { 3.0 } else { 1.0 })),
                        RigidBody::KinematicPositionBased,
                        Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                    ))
                        // Right Lower Arm
                        .with_child((
                            Mesh2d(meshes.add(Capsule2d {
                                radius: LIMB_RADIUS,
                                half_length: LIMB_LENGTH,
                            })),
                            MeshMaterial2d(materials.add(profile.color)),
                            BodyParts::new(false, false, true, true, false),
                            PlayerID(id),
                            Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
                            RigidBody::KinematicPositionBased,
                            Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                        ));
                    // Left Upper Arm
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(profile.color)),
                        BodyParts::new(false, false, true, false, true),
                        PlayerID(id),
                        // player 0 is right facing, and player 1 is left facing
                        // so we need to change which arm is on top
                        Transform::from_translation(Vec3::new(0.0, 0.0, if id == 0 { 1.0 } else { 3.0 })),
                        RigidBody::KinematicPositionBased,
                        Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                    ))
                        // Left Lower Arm
                        .with_child((
                            Mesh2d(meshes.add(Capsule2d {
                                radius: LIMB_RADIUS,
                                half_length: LIMB_LENGTH,
                            })),
                            MeshMaterial2d(materials.add(profile.color)),
                            BodyParts::new(false, false, true, false, false),
                            PlayerID(id),
                            Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
                            RigidBody::KinematicPositionBased,
                            Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                        ));
                    // Right Upper Leg
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(profile.color)),
                        // right upper leg
                        BodyParts::new(false, false, false, true, true),
                        PlayerID(id),
                        // player 0 is right facing, and player 1 is left facing
                        // so we need to change which leg is on top
                        Transform::from_translation(Vec3::new(0.0, -100.0, if id == 0 { 3.0 } else { 1.0 })),
                        RigidBody::KinematicPositionBased,
                        Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                    ))
                        // Right Lower Leg
                        .with_children(|builder| {
                            builder.spawn((
                                Mesh2d(meshes.add(Capsule2d {
                                    radius: LIMB_RADIUS,
                                    half_length: LIMB_LENGTH,
                                })),
                                MeshMaterial2d(materials.add(profile.color)),
                                // right lower leg
                                BodyParts::new(false, false, false, true, false),
                                PlayerID(id),
                                Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
                                RigidBody::KinematicPositionBased,
                                Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                                Sensor,
                                ActiveEvents::COLLISION_EVENTS,
                            ));
                        });
                    // Left Upper Leg
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(profile.color)),
                        BodyParts::new(false, false, false, false, true),
                        PlayerID(id),
                        Transform::from_translation(Vec3::new(0.0, -100.0, if id == 0 { 1.0 } else { 3.0 })),
                        RigidBody::KinematicPositionBased,
                        Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                    ))
                        // Left Lower Leg
                        .with_children(|builder| {
                            builder.spawn((
                                Mesh2d(meshes.add(Capsule2d {
                                    radius: LIMB_RADIUS,
                                    half_length: LIMB_LENGTH,
                                })),
                                MeshMaterial2d(materials.add(profile.color)),
                                BodyParts::new(false, false, false, false, false),
                                PlayerID(id),
                                Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
                                RigidBody::KinematicPositionBased,
                                Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                                Sensor,
                                ActiveEvents::COLLISION_EVENTS,
                            ));
                        });
                });
        });
}

fn rotate_parts(transform: &mut Transform, offset: f32, degree: f32) {
    let rad = degree.to_radians();
    transform.rotation = Quat::from_rotation_z(rad);
    transform.translation.x = LIMB_LENGTH * rad.sin();
    transform.translation.y = offset + LIMB_LENGTH * (1.0-rad.cos());
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Player>
) {
    if keys.get_pressed().len() == 0 {
        for mut player in query.iter_mut() {
            if player.state != PlayerState::Idle && player.state != PlayerState::Jumping {
                player.state = PlayerState::Idle;
                player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                player.animation.phase = 0;
                player.animation.count = 30;
            }
        }
    }
    if keys.pressed(KeyCode::KeyD) {
        for mut player in query.iter_mut() {
            if player.state == PlayerState::Jumping {
                player.velocity.x = TERMINATE_VELOCITY;
            } else if player.state != PlayerState::Running {
                player.state = PlayerState::Running;
                player.animation.diff_pose = (RUNNING_POSE1 - player.pose) / 30.0;
                player.animation.phase = 0;
                player.animation.count = 30;
            }
            player.pose.facing = true;
        }

    } else if keys.pressed(KeyCode::KeyA) {
        for mut player in query.iter_mut() {
            if player.state == PlayerState::Jumping {
                player.velocity.x = -TERMINATE_VELOCITY;
            } else if player.state != PlayerState::Running {
                player.state = PlayerState::Running;
                player.animation.diff_pose = (RUNNING_POSE1 - player.pose) / 30.0;
                player.animation.phase = 0;
                player.animation.count = 30;
            }
            player.pose.facing = false;
        }
    }
    if keys.just_pressed(KeyCode::Space) {
        for mut player in query.iter_mut() {
            if player.state != PlayerState::Jumping {
                player.state = PlayerState::Jumping;
                player.animation.diff_pose = (JUMPING_POSE1 - player.pose) / 30.0;
                player.animation.phase = 0;
                player.animation.count = 30;
                player.velocity = Vec2::new(0.0, 6.0);
            }
        }
    }
}

fn player_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut timer: ResMut<AnimationTimer>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        for (mut player, mut transform) in player_query.iter_mut() {
            match player.state {
                PlayerState::Idle => {
                    player.velocity = Vec2::ZERO;
                    if player.animation.phase == 0 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.phase = 1;
                            player.animation.count = 10;
                        }
                    } else if player.animation.phase == 1 {
                        player.animation.count -= 1;
                        if player.animation.count == 0 {
                            player.animation.diff_pose = (IDLE_POSE2 - IDLE_POSE1) / 10.0;
                            player.animation.phase = 2;
                            player.animation.count = 10;
                        }
                    } else if player.animation.phase == 2 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.diff_pose = (IDLE_POSE1 - IDLE_POSE2) / 10.0;
                            player.animation.phase = 3;
                            player.animation.count = 10;
                        }
                    } else if player.animation.phase == 3 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.phase = 1;
                            player.animation.count = 10;
                        }
                    }
                }
                PlayerState::Running => {
                    if player.pose.facing && player.velocity.x < TERMINATE_VELOCITY {
                        player.velocity += Vec2::new(1.0, 0.0) * PIXELS_PER_METER / FPS;
                    } else if !player.pose.facing && player.velocity.x > -TERMINATE_VELOCITY {
                        player.velocity += Vec2::new(-1.0, 0.0) * PIXELS_PER_METER / FPS;
                    }
                    if player.velocity.x > TERMINATE_VELOCITY {
                        player.velocity.x = TERMINATE_VELOCITY;
                    } else if player.velocity.x < -TERMINATE_VELOCITY {
                        player.velocity.x = -TERMINATE_VELOCITY;
                    }
                    if player.animation.phase == 0 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.diff_pose = (RUNNING_POSE2 - RUNNING_POSE1) / 15.0;
                            player.animation.phase = 1;
                            player.animation.count = 15;
                        }
                    } else if player.animation.phase == 1 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.diff_pose = (RUNNING_POSE1 - RUNNING_POSE2) / 15.0;
                            player.animation.phase = 2;
                            player.animation.count = 15;
                        }
                    } else if player.animation.phase == 2 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.diff_pose = (RUNNING_POSE2 - RUNNING_POSE1) / 15.0;
                            player.animation.phase = 1;
                            player.animation.count = 15;
                        }
                    }
                }
                PlayerState::Jumping => {
                    player.velocity -= Vec2::new(0.0, GRAVITY_ACCEL * 1.5 / FPS);
                    if player.animation.phase == 0 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.diff_pose = (JUMPING_POSE2 - JUMPING_POSE1) / 30.0;
                            player.animation.phase = 1;
                            player.animation.count = 30;
                        }
                    } else if player.animation.phase == 1 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.phase = 2;
                            player.animation.count = 0;
                        }
                    }
                }
            }
            transform.translation += Vec3::new(player.velocity.x, player.velocity.y, 0.0) * PIXELS_PER_METER / FPS;
            if transform.translation.x < -config.window_size.x / 2.0 {
                transform.translation.x = -config.window_size.x / 2.0;
            } else if transform.translation.x > config.window_size.x / 2.0 {
                transform.translation.x = config.window_size.x / 2.0;
            }
        }
    }
}

fn check_ground(
    mut collision_events: EventReader<CollisionEvent>,
    parts_query: Query<(&BodyParts, &PlayerID)>,
    ground_query: Query<Entity, With<Ground>>,
    mut player_query: Query<(&mut Player, &PlayerID)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _) => {
                if *entity1 == ground_query.single(){
                    let (parts, id) = parts_query.get(*entity2).unwrap();
                    if !parts.is_arm() && !parts.is_upper() {
                        for (mut player, player_id) in player_query.iter_mut() {
                            if id == player_id {
                                player.velocity = Vec2::ZERO;
                                player.state = PlayerState::Idle;
                                player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                                player.animation.phase = 0;
                                player.animation.count = 30;
                            }
                        }
                    }
                } else if *entity2 == ground_query.single() {
                    let (parts, id) = parts_query.get(*entity1).unwrap();
                    if !parts.is_arm() && !parts.is_upper() {
                        for (mut player, player_id) in player_query.iter_mut() {
                            if id == player_id {
                                player.velocity = Vec2::ZERO;
                                player.state = PlayerState::Idle;
                                player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                                player.animation.phase = 0;
                                player.animation.count = 30;
                            }
                        }
                    }
                }
            }
            CollisionEvent::Stopped(_, _, _) => {
            }
        }
    }
}

fn update_pose(
    player_query: Query<(&Player, &PlayerID)>,
    mut parts_query: Query<(&BodyParts, &PlayerID, &mut Transform)>,
) {
    for (player, player_id) in player_query.iter() {
        let flip = if player.pose.facing { 1.0 } else { -1.0 };
        for (parts, parts_id, mut transform) in parts_query.iter_mut() {
            if player_id.0 == parts_id.0 {
                match parts.flags {
                    // Head
                    0b10000 => rotate_parts(&mut transform, HEAD_OFFSET, flip * player.pose.head),
                    // Body
                    0b01000 => rotate_parts(&mut transform, BODY_OFFSET, flip * player.pose.body),
                    // Right Upper Arm
                    0b00111 => rotate_parts(&mut transform, UPPER_ARM_OFFSET, flip * player.pose.right_upper_arm),
                    // Right Lower Arm
                    0b00110 => rotate_parts(&mut transform, LOWER_ARM_OFFSET, flip * player.pose.right_lower_arm),
                    // Right Upper Leg
                    0b00011 => rotate_parts(&mut transform, UPPER_LEG_OFFSET, flip * player.pose.right_upper_leg),
                    // Right Lower Leg
                    0b00010 => rotate_parts(&mut transform, LOWER_LEG_OFFSET, flip * player.pose.right_lower_leg),
                    // Left Upper Arm
                    0b00101 => rotate_parts(&mut transform, UPPER_ARM_OFFSET, flip * player.pose.left_upper_arm),
                    // Left Lower Arm
                    0b00100 => rotate_parts(&mut transform, LOWER_ARM_OFFSET, flip * player.pose.left_lower_arm),
                    // Left Upper Leg
                    0b00001 => rotate_parts(&mut transform, UPPER_LEG_OFFSET, flip * player.pose.left_upper_leg),
                    // Left Lower Leg
                    0b00000 => rotate_parts(&mut transform, LOWER_LEG_OFFSET, flip * player.pose.left_lower_leg),
                    _ => {}
                }
            }
        }
    }
}

fn update_health_bar(
    player_query: Query<(&Player, &PlayerID)>,
    mut health_query: Query<(&mut HealthBar, &PlayerID)>,
) {
    for (player, player_id) in player_query.iter() {
        let profile = &CHARACTER_PROFILES[player_id.0 as usize];
        for (mut health, health_id) in health_query.iter_mut() {
            if player_id == health_id {
                health.0 = (player.health / profile.health * 100) as f32;
            }
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AnimationTimer {
                timer: Timer::from_seconds(1.0 / FPS, TimerMode::Repeating),
            })
            .add_systems(Update, player_input.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, player_movement.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, check_ground.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, update_pose.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, update_health_bar.run_if(in_state(AppState::Ingame)));
    }
}