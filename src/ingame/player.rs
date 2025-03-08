use std::{fmt::Debug, ops::{BitAndAssign, BitOr, BitOrAssign, Not}};
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy_rapier2d::prelude::*;
use crate::{character_def::CHARACTER_PROFILES, ingame::{Ground, InGame}, AppState, GameConfig, GameMode};
use super::{pose::*, BackGround, Fighting};

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

const FPS: f32 = 60.0;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct PlayerID(pub u8);

#[derive(Component)]
pub struct HealthBar(pub f32, pub f32);

/// Represents the current state of a player using bit flags.
/// Multiple states can be active simultaneously by combining flags with bitwise OR.
/// 
/// | State          | Bit Pattern | Description                    |
/// |----------------|-------------|--------------------------------|
/// | IDLE          | 0b00000000  | Default state, no action      |
/// | RUNNING       | 0b00000001  | Player is moving horizontally |
/// | JUMPING       | 0b00000010  | Player is in first jump      |
/// | DOUBLE_JUMPING| 0b00000100  | Player is in second jump     |
/// | KICKING       | 0b00001000  | Player is performing kick    |
/// | PUNCHING      | 0b00010000  | Player is performing punch   |
/// | SPECIAL_ATTACK| 0b00100000  | Player is performing special attack |
/// | COOLDOWN      | 0b01000000  | Player is in cooldown state  |
#[derive(PartialEq, Eq, Copy, Clone)]
struct PlayerState(u8);

impl BitOr for PlayerState {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for PlayerState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for PlayerState {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for PlayerState {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self(0)
    }
}

impl Debug for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let states = [
            (PlayerState::RUNNING, "RUNNING"),
            (PlayerState::JUMPING, "JUMPING"),
            (PlayerState::DOUBLE_JUMPING, "DOUBLE_JUMPING"),
            (PlayerState::KICKING, "KICKING"),
            (PlayerState::PUNCHING, "PUNCHING"),
            (PlayerState::SPECIAL_ATTACK, "SPECIAL_ATTACK"),
            (PlayerState::COOLDOWN, "COOLDOWN"),
        ];
        
        if self.0 == 0 {
            write!(f, "IDLE")?;
            return Ok(());
        }
        
        let mut first = true;
        for (state, name) in states {
            if self.check(state) {
                if !first {
                    write!(f, "|")?;
                }
                write!(f, "{}", name)?;
                first = false;
            }
        }
        
        // Also include the raw binary representation
        write!(f, " ({:#010b})", self.0)
    }
}

impl PlayerState {
    pub const IDLE: Self = Self(0b00000000);
    pub const RUNNING: Self = Self(0b00000001);
    pub const JUMPING: Self = Self(0b00000010);
    pub const DOUBLE_JUMPING: Self = Self(0b00000100);
    pub const KICKING: Self = Self(0b00001000);
    pub const PUNCHING: Self = Self(0b00010000);
    pub const SPECIAL_ATTACK: Self = Self(0b00100000);
    pub const COOLDOWN: Self = Self(0b01000000);
    // ignore cooldown state
    pub fn is_idle(&self) -> bool {
        self.0 & !Self::COOLDOWN.0 == 0
    }
    pub fn check(&self, state: Self) -> bool {
        self.0 & state.0 != 0
    }
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
pub struct Player {
    character_id: isize,
    pose: Pose,
    animation: PlayerAnimation,
    state: PlayerState,
    velocity: Vec2,
    health: u32,
}

impl Player {
    pub fn new(character_id: isize) -> Self {
        Self {
            character_id,
            pose: IDLE_POSE1,
            animation: PlayerAnimation { diff_pose: default(), phase: 1, count: 10 },
            state: PlayerState::default(),
            velocity: Vec2::ZERO,
            health: CHARACTER_PROFILES[character_id as usize].health,
        }
    }
    pub fn new_opposite(character_id: isize) -> Self {
        Self {
            character_id,
            pose: OPPOSITE_DEFAULT_POSE,
            animation: PlayerAnimation { diff_pose: default(), phase: 1, count: 10 },
            state: PlayerState::default(),
            velocity: Vec2::ZERO,
            health: CHARACTER_PROFILES[character_id as usize].health,
        }
    }
    pub fn reset(&mut self, id: &PlayerID) {
        if id.0 == 0 {
            self.pose = IDLE_POSE1;
        } else {
            self.pose = OPPOSITE_DEFAULT_POSE;
        }
        self.animation = PlayerAnimation { diff_pose: default(), phase: 1, count: 10 };
        self.state = PlayerState::default();
        self.velocity = Vec2::ZERO;
        self.health = CHARACTER_PROFILES[self.character_id as usize].health;
    }
}

#[derive(Component)]
struct BodyParts {
    flags: u8
}

#[allow(dead_code)]
impl BodyParts {
    const NULL: Self = Self { flags: 0b00000 };
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


/// Spawns a player character with the specified ID and character profile.
/// 
/// # Arguments
/// 
/// * `id` - The player ID (0 for player 1, 1 for player 2)
/// * `character_id` - Index into CHARACTER_PROFILES for the character definition
/// * `builder` - The entity builder to spawn the player hierarchy
/// * `meshes` - Asset server for creating mesh components 
/// * `materials` - Asset server for creating material components
/// * `y_pos` - Initial Y position to spawn the player at
///
/// Creates a full player character hierarchy including:
/// - Main player entity with components for state, animation, etc
/// - Body parts (head, torso, arms, legs) with colliders and materials
/// - Configures physics properties and collision sensors
pub fn spawn_player(
    id: u8,
    character_id: isize,
    builder: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    y_pos: f32,
) {
    let profile = &CHARACTER_PROFILES[character_id as usize];
    builder.spawn((
        if id == 0 { Player::new(character_id) } else { Player::new_opposite(character_id) },
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
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                                ActiveEvents::COLLISION_EVENTS,
                                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
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
                                ActiveEvents::COLLISION_EVENTS,
                                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
                            ));
                        });
                });
        });
}

/// Rotates and positions a body part based on given parameters.
/// 
/// # Arguments
/// 
/// * `transform` - The Transform component to modify
/// * `offset` - Vertical offset from the parent joint
/// * `degree` - Rotation angle in degrees (positive is counterclockwise)
/// 
/// This function:
/// 1. Converts degree to radians and sets rotation
/// 2. Calculates X offset using sin of angle * limb length  
/// 3. Calculates Y position using cosine and adds vertical offset
fn rotate_parts(transform: &mut Transform, offset: f32, degree: f32) {
    let rad = degree.to_radians();
    transform.rotation = Quat::from_rotation_z(rad);
    transform.translation.x = LIMB_LENGTH * rad.sin();
    transform.translation.y = offset + LIMB_LENGTH * (1.0-rad.cos());
}

/// Handles player input for character controls.
///
/// # Arguments
///
/// * `keys` - Resource providing keyboard input state
/// * `config` - Resource containing game configuration
/// * `query` - Query to access player components
///
/// This function processes keyboard input to control player characters:
/// - Movement (A/D keys for running left/right)
/// - Jumping (Space key for single/double jumps)
/// - Combat moves:
///   - K key for kicks
///   - L key for punches
///   - J key for special kick attack
///   - H key for special punch attack
///
/// The function updates player state and animations based on input,
/// handling state transitions and preventing invalid combinations
/// of moves. For multiplayer, it processes input for both players
/// unless in single player mode.
fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Player, &PlayerID)>
) {
    for (mut player, player_id) in query.iter_mut() {
        if player_id.0 == 1 && config.mode == GameMode::SinglePlayer {
            continue;
        }
        if player.state.check(PlayerState::COOLDOWN) {
            continue;
        }
        if keys.pressed(KeyCode::KeyD) {
            if player.state.check(PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) {
                player.velocity.x = CHARACTER_PROFILES[player.character_id as usize].agility;
            } else if !player.state.check(PlayerState::RUNNING) {
                player.state |= PlayerState::RUNNING;
                player.animation.diff_pose = (RUNNING_POSE1 - player.pose) / 10.0;
                player.animation.phase = 0;
                player.animation.count = 10;
            }
            player.pose.facing = true;
        } else if keys.pressed(KeyCode::KeyA) {
            if player.state.check(PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) {
                player.velocity.x = -CHARACTER_PROFILES[player.character_id as usize].agility;
            } else if !player.state.check(PlayerState::RUNNING) {
                player.state |= PlayerState::RUNNING;
                player.animation.diff_pose = (RUNNING_POSE1 - player.pose) / 10.0;
                player.animation.phase = 0;
                player.animation.count = 10;
            }
            player.pose.facing = false;
        } else {
            if player.state.check(PlayerState::RUNNING) {
                player.state &= !PlayerState::RUNNING;
                player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                player.animation.phase = 0;
                player.animation.count = 30;
            }
        }
        if keys.just_pressed(KeyCode::Space) {
            if player.character_id == 1 {
                if !player.state.check(PlayerState::JUMPING | PlayerState::KICKING | PlayerState::PUNCHING) {
                    player.state |= PlayerState::JUMPING;
                    player.animation.diff_pose = (JUMPING_POSE1 - player.pose) / 30.0;
                    player.animation.phase = 0;
                    player.animation.count = 30;
                    player.velocity = Vec2::new(0.0, 7.0);
                } else if !player.state.check(PlayerState::DOUBLE_JUMPING) {
                    player.state |= PlayerState::DOUBLE_JUMPING;
                    player.animation.diff_pose = (JUMPING_POSE1 - player.pose) / 30.0;
                    player.animation.phase = 0;
                    player.animation.count = 30;
                    player.velocity = Vec2::new(0.0, 7.5);
                }
            } else {
                if !player.state.check(PlayerState::JUMPING) {
                    player.state |= PlayerState::JUMPING;
                    player.animation.diff_pose = (JUMPING_POSE1 - player.pose) / 30.0;
                    player.animation.phase = 0;
                    player.animation.count = 30;
                    player.velocity = Vec2::new(0.0, 7.0);
                }
            }
        }
        if keys.just_pressed(KeyCode::KeyK) {
            if !player.state.check(PlayerState::KICKING | PlayerState::PUNCHING) {
                if player.state.check(PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) {
                    player.state |= PlayerState::KICKING;
                    player.animation.diff_pose = (JUMPING_KICK_POSE - player.pose) / 10.0;
                    player.animation.phase = 0;
                    player.animation.count = 10;
                } else if !player.state.check(PlayerState::RUNNING) {
                    player.state |= PlayerState::KICKING;
                    player.animation.diff_pose = (KICK_POSE - player.pose) / 10.0;
                    player.animation.phase = 0;
                    player.animation.count = 10;
                }
            }
        }
        if keys.just_pressed(KeyCode::KeyL) {
            if !player.state.check(PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::RUNNING | PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) {
                if !player.state.check(PlayerState::RUNNING) {
                    player.state |= PlayerState::PUNCHING;
                    player.animation.diff_pose = (PUNCH_POSE - player.pose) / 10.0;
                    player.animation.phase = 0;
                    player.animation.count = 10;
                }
            }
        }
        if keys.just_pressed(KeyCode::KeyJ) {
            if player.state.check(PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) & !player.state.check(PlayerState::KICKING) {
                player.state |= PlayerState::KICKING;
                player.animation.diff_pose = (JUMPING_KICK_POSE - player.pose) / 10.0;
                player.animation.phase = 0;
                player.animation.count = 10;
            } else if !player.state.check(PlayerState::SPECIAL_ATTACK | PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::RUNNING) {
                player.state |= PlayerState::KICKING | PlayerState::SPECIAL_ATTACK;
                player.animation.diff_pose = (HIGH_KICK_POSE - player.pose) / 10.0;
                player.animation.phase = 0;
                player.animation.count = 10;
            }
        }
        if keys.just_pressed(KeyCode::KeyH) {
            if !player.state.check(PlayerState::SPECIAL_ATTACK | PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::RUNNING | PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) {
                player.state |= PlayerState::PUNCHING | PlayerState::SPECIAL_ATTACK;
                player.animation.diff_pose = (UPPER_PUNCH_POSE1 - player.pose) / 5.0;
                player.animation.phase = 0;
                player.animation.count = 5;
            }
        }
    }
}

/// Handles the movement and animation of the player character.
///
/// # Arguments
///
/// * `time` - Resource providing access to the elapsed time
/// * `config` - Resource containing game configuration settings
/// * `timer` - Resource managing the animation timer
/// * `player_query` - Query to access and modify player components
///
/// This function:
/// 1. Updates the animation timer and checks if it has finished.
/// 2. Iterates through each player and updates their state based on their current state and animation phase.
/// 3. Adjusts the player's velocity and position based on their state and input.
/// 4. Ensures the player stays within the game window boundaries.
fn player_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut timer: ResMut<AnimationTimer>,
    mut player_query: Query<(&mut Player, &mut Transform), Without<BackGround>>,
    mut ground_query: Query<&mut Transform, (With<BackGround>, Without<Player>)>,
) {
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        for (mut player, mut transform) in player_query.iter_mut() {
            if player.state.is_idle() {
                player.velocity = Vec2::ZERO;
                if player.animation.phase == 0 {
                    player.animation.count -= 1;
                    let diff_pose = player.animation.diff_pose;
                    player.pose += diff_pose;
                    if player.animation.count == 0 {
                        if player.state.check(PlayerState::COOLDOWN) {
                            player.state &= !PlayerState::COOLDOWN;
                        }
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
            if player.state.check(PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) {
                player.velocity -= Vec2::new(0.0, GRAVITY_ACCEL * 1.5 / FPS);
                if player.state.check(PlayerState::KICKING) {
                    if player.animation.phase == 0 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                        if player.animation.count == 0 {
                            player.animation.phase = 1;
                            player.animation.count = 0;
                        }
                    }
                } else {
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
                    } else if player.animation.phase == 2 && player.animation.count != 0 {
                        player.animation.count -= 1;
                        let diff_pose = player.animation.diff_pose;
                        player.pose += diff_pose;
                    }
                }
            } else {
                if player.state.check(PlayerState::KICKING) {
                    if player.state.check(PlayerState::SPECIAL_ATTACK) {
                        if player.animation.phase == 0 {
                            player.animation.count -= 1;
                            let diff_pose = player.animation.diff_pose;
                            player.pose += diff_pose;
                            if player.animation.count == 0 {
                                player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                                player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                                player.animation.phase = 0;
                                player.animation.count = 30;
                            }
                        }
                    } else {
                        if player.animation.phase == 0 {
                            player.animation.count -= 1;
                            let diff_pose = player.animation.diff_pose;
                            player.pose += diff_pose;
                            if player.animation.count == 0 {
                                player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                                player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                                player.animation.phase = 0;
                                player.animation.count = 30;
                            }
                        }
                    }
                }
                if player.state.check(PlayerState::PUNCHING) {
                    if player.state.check(PlayerState::SPECIAL_ATTACK) {
                        if player.animation.phase == 0 {
                            player.animation.count -= 1;
                            let diff_pose = player.animation.diff_pose;
                            player.pose += diff_pose;
                            if player.animation.count == 0 {
                                player.animation.diff_pose = (UPPER_PUNCH_POSE2 - UPPER_PUNCH_POSE1) / 5.0;
                                player.animation.phase = 1;
                                player.animation.count = 5;
                            }
                        } else if player.animation.phase == 1 {
                            player.animation.count -= 1;
                            let diff_pose = player.animation.diff_pose;
                            player.pose += diff_pose;
                            if player.animation.count == 0 {
                                player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                                player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                                player.animation.phase = 0;
                                player.animation.count = 30;
                            }
                        }
                    } else {
                        if player.animation.phase == 0 {
                            player.animation.count -= 1;
                            let diff_pose = player.animation.diff_pose;
                            player.pose += diff_pose;
                            if player.animation.count == 0 {
                                player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                                player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                                player.animation.phase = 0;
                                player.animation.count = 30;
                            }
                        }
                    }
                }
                if player.state.check(PlayerState::RUNNING) {
                    if player.pose.facing && player.velocity.x < CHARACTER_PROFILES[player.character_id as usize].agility {
                        player.velocity += Vec2::new(1.0, 0.0) * PIXELS_PER_METER / FPS;
                    } else if !player.pose.facing && player.velocity.x > -CHARACTER_PROFILES[player.character_id as usize].agility {
                        player.velocity += Vec2::new(-1.0, 0.0) * PIXELS_PER_METER / FPS;
                    }
                    if player.velocity.x > CHARACTER_PROFILES[player.character_id as usize].agility {
                        player.velocity.x = CHARACTER_PROFILES[player.character_id as usize].agility;
                    } else if player.velocity.x < -CHARACTER_PROFILES[player.character_id as usize].agility {
                        player.velocity.x = -CHARACTER_PROFILES[player.character_id as usize].agility;
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
            }
            transform.translation += Vec3::new(player.velocity.x, player.velocity.y, 0.0) * PIXELS_PER_METER / FPS;
        }

        // move player and ground
        let mut ground = ground_query.get_single_mut().unwrap();
        let mut avg_x = 0.0;
        for (_, transform) in player_query.iter_mut() {
            avg_x += transform.translation.x;
        }
        avg_x /= 2.0;

        if avg_x > 0.0 && ground.translation.x == config.window_size.x / 2.0 - 2000.0 {
            return;
        } else if avg_x < 0.0 && ground.translation.x == 2000.0 - config.window_size.x / 2.0 {
            return;
        }
        
        // Check if players are at opposite ends of the screen
        let mut at_edges = true;
        let mut x_positions = Vec::new();
        for (_, transform) in player_query.iter() {
            x_positions.push(transform.translation.x);
            if transform.translation.x > -config.window_size.x / 2.0 + 100.0 && 
               transform.translation.x < config.window_size.x / 2.0 - 100.0 {
                at_edges = false;
                break;
            }
        }
        
        // If both players are at edges and on opposite sides, don't move camera
        if at_edges && x_positions.len() > 1 && 
           ((x_positions[0] < 0.0 && x_positions[1] > 0.0) || 
            (x_positions[0] > 0.0 && x_positions[1] < 0.0))
        {
            avg_x = 0.0;
        }
        let mut diff = avg_x;

        // Otherwise, move camera to center players
        if ground.translation.x + avg_x < config.window_size.x / 2.0 - 2000.0 {
            avg_x = config.window_size.x / 2.0 - 2000.0 - ground.translation.x;
        } else if ground.translation.x + avg_x > 2000.0 - config.window_size.x / 2.0 {
            avg_x = 2000.0 - config.window_size.x / 2.0 - ground.translation.x;
        }
        ground.translation.x += avg_x;

        // Move players to center of screen
        if ground.translation.x < config.window_size.x / 2.0 - 2000.0 {
            diff = config.window_size.x / 2.0 - 2000.0 - ground.translation.x;
            ground.translation.x = config.window_size.x / 2.0 - 2000.0;
        } else if ground.translation.x > 2000.0 - config.window_size.x / 2.0 {
            diff = 2000.0 - config.window_size.x / 2.0 - ground.translation.x;
            ground.translation.x = 2000.0 - config.window_size.x / 2.0;
        }

        for (_, mut transform) in player_query.iter_mut() {
            transform.translation.x -= diff;
            if transform.translation.x < -config.window_size.x / 2.0 + 100.0 {
                transform.translation.x = -config.window_size.x / 2.0 + 100.0;
            } else if transform.translation.x > config.window_size.x / 2.0 - 100.0 {
                transform.translation.x = config.window_size.x / 2.0 - 100.0;
            }
        }
    }
}

/// Checks for collisions between player characters and the ground.
fn check_ground(
    mut collision_events: EventReader<CollisionEvent>,
    parts_query: Query<(&BodyParts, &PlayerID)>,
    ground_query: Query<Entity, With<Ground>>,
    mut player_query: Query<(&mut Player, &PlayerID)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _) => {
                // Determine which entity is the ground and which is the potential player part
                let (part_entity, _) = if *entity1 == ground_query.single() {
                    (*entity2, *entity1)
                } else if *entity2 == ground_query.single() {
                    (*entity1, *entity2)
                } else {
                    continue // Neither entity is ground, so exit early
                };

                // Check if we found a collision with the ground
                if let Ok((parts, id)) = parts_query.get(part_entity) {
                    // Only process for non-arm and non-upper body parts
                    if !parts.is_arm() && !parts.is_upper() {
                        for (mut player, player_id) in player_query.iter_mut() {
                            if id == player_id && player.state.check(PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) {
                                player.velocity = Vec2::ZERO;
                                player.state &= !(PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING | 
                                                PlayerState::KICKING | PlayerState::SPECIAL_ATTACK);
                                
                                // Set animation based on running state
                                if player.state.check(PlayerState::RUNNING) {
                                    player.animation.diff_pose = (RUNNING_POSE1 - player.pose) / 10.0;
                                    player.animation.phase = 0;
                                    player.animation.count = 10;
                                } else {
                                    player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                                    player.animation.phase = 0;
                                    player.animation.count = 30;
                                }
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

/// Updates the pose of the player character based on their current state.
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

fn check_attack(
    mut collision_events: EventReader<CollisionEvent>,
    parts_query: Query<(&BodyParts, &PlayerID)>,
    mut player_query: Query<(&mut Player, &PlayerID)>,
) {
    let mut player_info: [(isize, PlayerState); 2] = [(0, PlayerState::IDLE); 2];
    for (player, player_id) in player_query.iter() {
        player_info[player_id.0 as usize] = (player.character_id, player.state);
    }
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _) => {
                let Ok((parts1, id1)) = parts_query.get(*entity1) else {
                    continue;
                };
                let Ok((parts2, id2)) = parts_query.get(*entity2) else {
                    continue;
                };

                // Check if the collision is between two player characters
                if id1 == id2 { continue }

                let mut attacker_id: PlayerID = PlayerID(2);
                let mut opponent_id: PlayerID = PlayerID(2);
                let mut attacker_parts: &BodyParts = &BodyParts::NULL;
                let mut opponent_parts: &BodyParts = &BodyParts::NULL;
                let mut attacker_power: f32 = 0.0;
                for (player, player_id) in player_query.iter() {
                    if player.state.check(PlayerState::KICKING | PlayerState::PUNCHING) {
                        // Check if the attacker is already set
                        if id1 == player_id {
                            attacker_parts = parts1;
                            opponent_parts = parts2;
                        } else if id2 == player_id {
                            attacker_parts = parts2;
                            opponent_parts = parts1;
                        }

                        // Check if the attacker is in a valid state
                        if player.state.check(PlayerState::KICKING) && attacker_parts.is_arm(){
                            continue;
                        } else if player.state.check(PlayerState::PUNCHING) &&  !attacker_parts.is_arm(){
                            continue;
                        }

                        // Check if the attacker is already set
                        if attacker_id != PlayerID(2) && attacker_power > CHARACTER_PROFILES[player.character_id as usize].power {
                            continue;
                        }
                        attacker_id = *player_id;
                        opponent_id = if PlayerID(0) == attacker_id { PlayerID(1) } else { PlayerID(0) };
                        attacker_power = CHARACTER_PROFILES[player.character_id as usize].power;
                    }
                }
                if attacker_id == PlayerID(2) { continue; }

                let mut damage: u32 = 0;
                if let Some((mut player, _)) = player_query.iter_mut().find(|(_, id)| id.0 == attacker_id.0) {
                    damage = calculate_damage(
                        player_info[attacker_id.0 as usize],
                        player_info[opponent_id.0 as usize],
                        opponent_parts,
                    );
                    println!("Player {} hit: {} damage", attacker_id.0, damage);
                    player.state &= !(PlayerState::KICKING | PlayerState::PUNCHING);
                    player.animation.diff_pose = (IDLE_POSE1 - player.pose) / 30.0;
                    player.animation.phase = 0;
                    player.animation.count = 30;
                }
                if let Some((mut player, _)) = player_query.iter_mut().find(|(_, id)| id.0 == opponent_id.0) {
                    player.health = player.health.saturating_sub(damage);
                }
            }
            CollisionEvent::Stopped(_, _, _) => {
            }
        }
    }
}

fn calculate_damage(
    attacker_info: (isize, PlayerState),
    opponent_info: (isize, PlayerState),
    opponent_parts: &BodyParts,
) -> u32 {
    let attacker_profile = &CHARACTER_PROFILES[attacker_info.0 as usize];
    let opponent_profile = &CHARACTER_PROFILES[opponent_info.0 as usize];
    let mut damage = attacker_profile.power;
    
    // Apply damage multipliers based on player states
    if attacker_info.1.check(PlayerState::KICKING) {
        damage *= 1.3;
    } else if attacker_info.1.check(PlayerState::PUNCHING) {
        damage *= 1.0;
    }

    // If attacker is performes a jumping kick or double jump kick, double the damage
    if attacker_info.1.check(PlayerState::JUMPING | PlayerState::DOUBLE_JUMPING) {
        damage *= 2.0;
    }

    // If attacker is performing a special attack, double the damage
    if attacker_info.1.check(PlayerState::SPECIAL_ATTACK) {
        damage *= 2.0;
    }

    // Apply damage multipliers based on opponent body parts
    if opponent_parts.is_head() {
        damage *= 2.0;
    } else if opponent_parts.is_body() {
        damage *= 1.5;
    } else if opponent_parts.is_upper() {
        damage *= 1.3;
    } else if opponent_parts.is_arm() {
        damage *= 0.8;
    }

    // Apply damage reduction based on opponent defense
    return (damage / opponent_profile.defense).floor() as u32;
}

/// Updates the health bar of the player character based on their current health.
fn update_health_bar(
    mut meshes: ResMut<Assets<Mesh>>,
    player_query: Query<(&Player, &PlayerID)>,
    mut health_query: Query<(&mut HealthBar, &mut Mesh2d, &PlayerID)>,
) {
    for (player, player_id) in player_query.iter() {
        let profile = &CHARACTER_PROFILES[player.character_id as usize];
        for (mut health_bar, mesh_handler, health_id) in health_query.iter_mut() {
            if player_id == health_id {
                let old_health = health_bar.0;
                health_bar.0 = player.health as f32 / profile.health as f32;
                if old_health == health_bar.0 { continue };
                if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                    if let Some(VertexAttributeValues::Float32x3(ref mut positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
                        positions[3][0] = health_bar.1 * health_bar.0;
                        positions[2][0] = health_bar.1 * health_bar.0 + if player_id.0 == 0 { 50.0 } else { -50.0 };
                    }
                }
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
            .add_systems(Update, player_input.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, player_movement.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, check_ground.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, update_pose.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, check_attack.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, update_health_bar.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)));
    }
}