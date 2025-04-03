use std::{fmt::Debug, ops::{BitAndAssign, BitOr, BitOrAssign, Not}};
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy_rapier2d::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::GameMode;
use crate::{character_def::CHARACTER_PROFILES, ingame::{InGame, GameState}, AppState, GameConfig};
use super::{pose::*, BackGround, Fighting};

#[cfg(target_arch = "wasm32")]
use crate::ingame::wasm::DoubleJumpCheck;

const SKIN_COLOR: Color = Color::srgb(0.9, 0.8, 0.7);

// definition for normal display
#[cfg(not(target_arch = "wasm32"))]
const LIMB_LENGTH: f32 = 30.0;
#[cfg(not(target_arch = "wasm32"))]
const LIMB_RADIUS: f32 = 15.0;
#[cfg(not(target_arch = "wasm32"))]
const BODY_THICKNESS: f32 = 10.0;
#[cfg(not(target_arch = "wasm32"))]
const HEAD_OFFSET: f32 = 100.0;
#[cfg(not(target_arch = "wasm32"))]
const BODY_OFFSET: f32 = 40.0;
#[cfg(not(target_arch = "wasm32"))]
const UPPER_ARM_OFFSET: f32 = 30.0;
#[cfg(not(target_arch = "wasm32"))]
const LOWER_ARM_OFFSET: f32 = -60.0;
#[cfg(not(target_arch = "wasm32"))]
const UPPER_LEG_OFFSET: f32 = -100.0;
#[cfg(not(target_arch = "wasm32"))]
const LOWER_LEG_OFFSET: f32 = -60.0;

// definition for web display
#[cfg(target_arch = "wasm32")]
const LIMB_LENGTH: f32 = 15.0;
#[cfg(target_arch = "wasm32")]
const LIMB_RADIUS: f32 = 7.5;
#[cfg(target_arch = "wasm32")]
const BODY_THICKNESS: f32 = 5.0;
#[cfg(target_arch = "wasm32")]
const HEAD_OFFSET: f32 = 50.0;
#[cfg(target_arch = "wasm32")]
const BODY_OFFSET: f32 = 20.0;
#[cfg(target_arch = "wasm32")]
const UPPER_ARM_OFFSET: f32 = 15.0;
#[cfg(target_arch = "wasm32")]
const LOWER_ARM_OFFSET: f32 = -30.0;
#[cfg(target_arch = "wasm32")]
const UPPER_LEG_OFFSET: f32 = -50.0;
#[cfg(target_arch = "wasm32")]
const LOWER_LEG_OFFSET: f32 = -30.0;


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
/// | State          | Bit Pattern         | Description                         |
/// |----------------|---------------------|-------------------------------------|
/// | IDLE           | 0b0000000000000000  | Default state, no action            |
/// | WALKING        | 0b0000000000000001  | Player is moving horizontally       |
/// | JUMP UP        | 0b0000000000000010  | Player is in first jump             |
/// | DOUBLE_JUMP    | 0b0000000000000100  | Player is in second jump            |
/// | KICKING        | 0b0000000000001000  | Player is performing kick           |
/// | PUNCHING       | 0b0000000000010000  | Player is performing punch          |
/// | SPECIAL_ATTACK | 0b0000000000100000  | Player is performing special attack |
/// | COOLDOWN       | 0b0000000001000000  | Player is in cooldown state         |
/// | DIRECTION      | 0b0000000010000000  | Player is moving right              |
/// | JUMP FORWARD   | 0b0000000100000000  | Player is jumping forward           |
/// | JUMP BACKWARD  | 0b0000001000000000  | Player is jumping backward          |
/// | BEND DOWN      | 0b0000010000000000  | Player is bending down              |
/// | ROLL BACK      | 0b0000100000000000  | Player is rolling back              |
/// | ROLL FORWARD   | 0b0001000000000000  | Player is rolling forward           |
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct PlayerState(u16);

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
            (PlayerState::WALKING, "WALKING"),
            (PlayerState::JUMP_UP, "JUMPING"),
            (PlayerState::DOUBLE_JUMP, "DOUBLE_JUMPING"),
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
    pub const IDLE: Self           = Self(0b0000000000000000);
    pub const WALKING: Self        = Self(0b0000000000000001);
    pub const JUMP_UP: Self        = Self(0b0000000000000010);
    pub const DOUBLE_JUMP: Self    = Self(0b0000000000000100);
    pub const KICKING: Self        = Self(0b0000000000001000);
    pub const PUNCHING: Self       = Self(0b0000000000010000);
    pub const SPECIAL_ATTACK: Self = Self(0b0000000000100000);
    pub const COOLDOWN: Self       = Self(0b0000000001000000);
    pub const DIRECTION: Self      = Self(0b0000000010000000);
    pub const JUMP_FORWARD: Self   = Self(0b0000000100000000);
    pub const JUMP_BACKWARD: Self  = Self(0b0000001000000000);
    pub const BEND_DOWN: Self      = Self(0b0000010000000000);
    pub const ROLL_BACK: Self      = Self(0b0000100000000000);
    pub const ROLL_FORWARD: Self   = Self(0b0001000000000000);

    // ignore cooldown state
    pub fn is_idle(&self) -> bool {
        self.0 & !Self::COOLDOWN.0 & !Self::DIRECTION.0 == 0
    }
    pub fn check(&self, state: Self) -> bool {
        self.0 & state.0 != 0
    }
    pub fn is_forward(&self) -> bool {
        self.0 & Self::DIRECTION.0 != 0
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
    pub character_id: isize,
    pub pose: Pose,
    animation: PlayerAnimation,
    pub state: PlayerState,
    pub velocity: Vec2,
    pub health: u32,
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
    pub fn set_animation(&mut self, pose: Pose, phase: u8, count: u8) {
        self.animation = PlayerAnimation { diff_pose: (pose - self.pose) / count as f32, phase, count };
    }
    pub fn update_animation(&mut self) {
        if self.animation.count == 0 {
            return;
        }
        self.pose += self.animation.diff_pose;
        self.animation.count -= 1;
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

// Represents a foot Entity
// true for right foot, false for left foot
#[derive(Component)]
struct Foot(bool);

#[derive(Resource)]
struct PlayerCollision(u8);


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
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(if id == 0 {-500.0} else {500.0}, y_pos, if id == 0 { 10.0 } else {1.0})),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(if id == 0 {-250.0} else {250.0}, y_pos, if id == 0 { 10.0 } else {1.0})),
        Visibility::Visible,
    ))
        // Body
        .with_children(|builder| {
            builder.spawn((
                #[cfg(not(target_arch = "wasm32"))]
                Mesh2d(meshes.add( Rectangle::new(BODY_THICKNESS * 4.0, 130.0))),
                #[cfg(target_arch = "wasm32")]
                Mesh2d(meshes.add( Rectangle::new(BODY_THICKNESS * 4.0, 65.0))),
                MeshMaterial2d(materials.add(profile.color)),
                Transform::default(),
                BodyParts::BODY,
                PlayerID(id),
                #[cfg(not(target_arch = "wasm32"))]
                Collider::cuboid(BODY_THICKNESS * 2.0, 65.0),
                #[cfg(target_arch = "wasm32")]
                Collider::cuboid(BODY_THICKNESS, 32.5),
                RigidBody::KinematicPositionBased,
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
            ))
                // Head
                .with_children(|builder| {
                    builder.spawn((
                        #[cfg(not(target_arch = "wasm32"))]
                        Mesh2d(meshes.add(Circle::new(45.0))),
                        #[cfg(target_arch = "wasm32")]
                        Mesh2d(meshes.add(Circle::new(22.5))),
                        MeshMaterial2d(materials.add(SKIN_COLOR)),
                        BodyParts::HEAD,
                        #[cfg(not(target_arch = "wasm32"))]
                        Transform::from_translation(Vec3::new(0.0, 100.0, 2.0)),
                        #[cfg(target_arch = "wasm32")]
                        Transform::from_translation(Vec3::new(0.0, 50.0, 2.0)),
                        RigidBody::KinematicPositionBased,
                        #[cfg(not(target_arch = "wasm32"))]
                        Collider::ball(45.0),
                        #[cfg(target_arch = "wasm32")]
                        Collider::ball(22.5),
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
                    ));
                    // Right Upper Arm
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(SKIN_COLOR)),
                        BodyParts::new(false, false, true, true, true),
                        PlayerID(id),
                        // player 0 is right facing, and player 1 is left facing
                        // so we need to change which arm is on top
                        Transform::from_translation(Vec3::new(0.0, UPPER_ARM_OFFSET, 2.0)),
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
                            MeshMaterial2d(materials.add(SKIN_COLOR)),
                            BodyParts::new(false, false, true, true, false),
                            PlayerID(id),
                            #[cfg(not(target_arch = "wasm32"))]
                            Transform::from_translation(Vec3::new(0.0, -60.0, 2.0)),
                            #[cfg(target_arch = "wasm32")]
                            Transform::from_translation(Vec3::new(0.0, -30.0, 2.0)),
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
                        MeshMaterial2d(materials.add(SKIN_COLOR)),
                        BodyParts::new(false, false, true, false, true),
                        PlayerID(id),
                        // player 0 is right facing, and player 1 is left facing
                        // so we need to change which arm is on top
                        Transform::from_translation(Vec3::new(0.0, UPPER_ARM_OFFSET, -1.0)),
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
                            MeshMaterial2d(materials.add(SKIN_COLOR)),
                            BodyParts::new(false, false, true, false, false),
                            PlayerID(id),
                            #[cfg(not(target_arch = "wasm32"))]
                            Transform::from_translation(Vec3::new(0.0, -60.0, 2.0)),
                            #[cfg(target_arch = "wasm32")]
                            Transform::from_translation(Vec3::new(0.0, -30.0, 2.0)),
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
                        #[cfg(not(target_arch = "wasm32"))]
                        Transform::from_translation(Vec3::new(20.0, -100.0, 3.0)),
                        #[cfg(target_arch = "wasm32")]
                        Transform::from_translation(Vec3::new(10.0, -50.0, 3.0)),
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
                                #[cfg(not(target_arch = "wasm32"))]
                                Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
                                #[cfg(target_arch = "wasm32")]
                                Transform::from_translation(Vec3::new(0.0, -30.0, 1.0)),
                                RigidBody::KinematicPositionBased,
                                Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                                ActiveEvents::COLLISION_EVENTS,
                                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC
                            )).with_child((
                                Foot(true),
                                PlayerID(id),
                                Mesh2d(meshes.add(Circle::new(LIMB_RADIUS))),
                                MeshMaterial2d(materials.add(SKIN_COLOR)),
                                #[cfg(not(target_arch = "wasm32"))]
                                Transform::from_translation(Vec3::new(0.0, -40.0, 1.0)),
                                #[cfg(target_arch = "wasm32")]
                                Transform::from_translation(Vec3::new(0.0, -20.0, 1.0)),
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
                        #[cfg(not(target_arch = "wasm32"))]
                        Transform::from_translation(Vec3::new(-20.0, -100.0, 1.0)),
                        #[cfg(target_arch = "wasm32")]
                        Transform::from_translation(Vec3::new(-10.0, -50.0, 1.0)),
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
                            )).with_child((
                                Foot(false),
                                PlayerID(id),
                                Mesh2d(meshes.add(Circle::new(LIMB_RADIUS))),
                                MeshMaterial2d(materials.add(SKIN_COLOR)),
                                #[cfg(not(target_arch = "wasm32"))]
                                Transform::from_translation(Vec3::new(0.0, -40.0, 1.0)),
                                #[cfg(target_arch = "wasm32")]
                                Transform::from_translation(Vec3::new(0.0, -20.0, 1.0)),
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
fn rotate_parts(transform: &mut Transform, x_offset: f32, y_offset: f32, degree: f32) {
    let rad = degree.to_radians();
    transform.rotation = Quat::from_rotation_z(rad);
    transform.translation.x = x_offset + LIMB_LENGTH * rad.sin();
    transform.translation.y = y_offset + LIMB_LENGTH * (1.0-rad.cos());
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
#[cfg(not(target_arch = "wasm32"))]
fn keyboard_input(
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
            if player.state.check(
                PlayerState::JUMP_UP
                | PlayerState::DOUBLE_JUMP
                | PlayerState::JUMP_BACKWARD
                | PlayerState::JUMP_FORWARD
                | PlayerState::BEND_DOWN
                | PlayerState::ROLL_BACK
                | PlayerState::ROLL_FORWARD
            ) {
                // player is jumping or bending or rolling
                // then just adding state
                player.state |= PlayerState::WALKING;
            } else if !player.state.check(PlayerState::WALKING) {
                // player is just walking
                player.state |= PlayerState::WALKING;
                player.set_animation(WALKING_POSE1, 0, 10);
            }
            // direction is right
            player.state |= PlayerState::DIRECTION;
        } else if keys.pressed(KeyCode::KeyA) {
            if player.state.check(
                PlayerState::JUMP_UP
                | PlayerState::DOUBLE_JUMP
                | PlayerState::JUMP_BACKWARD
                | PlayerState::JUMP_FORWARD
                | PlayerState::BEND_DOWN
                | PlayerState::ROLL_BACK
                | PlayerState::ROLL_FORWARD
            ) {
                // player is jumping or bending or rolling
                // then just adding state
                player.state |= PlayerState::WALKING;
            } else if !player.state.check(PlayerState::WALKING) {
                // player is just walking
                player.state |= PlayerState::WALKING;
                player.set_animation(WALKING_POSE1, 0, 10);
            }
            // direction is left
            player.state &= !PlayerState::DIRECTION;
        } else {
            // player is not walking
            if player.state.check(PlayerState::WALKING) {
                player.state &= !PlayerState::WALKING;
                player.set_animation(IDLE_POSE1, 0, 10);
            }
        }
        if keys.just_pressed(KeyCode::KeyS) {
            if player.state.is_idle() {
                // player is idle
                // then player will bend down
                player.state |= PlayerState::BEND_DOWN;
                player.set_animation(BEND_DOWN_POSE, 0, 10);
            } else if player.state.check(PlayerState::WALKING) {
                if player.state.check(PlayerState::DIRECTION) {
                    // player is walking right
                    // then player will roll forward
                    player.state |= PlayerState::ROLL_FORWARD;
                    player.set_animation(ROLL_FORWARD_POSE1, 0, 10);
                } else {
                    // player is walking left
                    // then player will roll back
                    player.state |= PlayerState::ROLL_BACK;
                    player.set_animation(ROLL_BACK_POSE1, 0, 10);
                }
                let x_vel = if player.state.is_forward() { 1.0 } else { -1.0 } * CHARACTER_PROFILES[player.character_id as usize].agility;
                player.velocity = Vec2::new(x_vel, 0.0);
            }
        }
        if keys.just_pressed(KeyCode::Space) {
            if player.character_id == 1 {
                // character 1 can double jump
                if player.state.is_idle() {
                    // player is idle
                    // then player will jump up
                    player.state |= PlayerState::JUMP_UP;
                    player.set_animation(JUMPING_POSE1, 0, 10);
                    player.velocity = Vec2::new(0.0, 12.0);                    
                } else if !player.state.check(
                    PlayerState::JUMP_UP
                        | PlayerState::DOUBLE_JUMP
                        | PlayerState::JUMP_FORWARD
                        | PlayerState::JUMP_BACKWARD
                ) && player.state.check(PlayerState::WALKING) {
                    if player.state.check(PlayerState::DIRECTION) {
                        // player is walking right
                        // then player will jump forward
                        player.state |= PlayerState::JUMP_FORWARD;
                        player.set_animation(JUMPING_POSE1, 0, 10);
                        let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                        player.velocity = Vec2::new(x_vel, 12.0);
                    } else {
                        // player is walking left
                        // then player will jump backward
                        player.state |= PlayerState::JUMP_BACKWARD;
                        player.set_animation(JUMPING_POSE1, 0, 10);
                        let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                        player.velocity = Vec2::new(-x_vel, 12.0);
                    }
                } else if player.state.check(
                    PlayerState::JUMP_UP
                    | PlayerState::JUMP_BACKWARD
                    | PlayerState::JUMP_FORWARD
                ) && !player.state.check(PlayerState::DOUBLE_JUMP) {
                    // player is jumping
                    // then player will double jump
                    player.state |= PlayerState::DOUBLE_JUMP;
                    player.set_animation(JUMPING_POSE1, 0, 10);
                    player.velocity.y = 7.5;
                }
            } else {
                // character 0 and 2 can only single jump
                if player.state.is_idle() {
                    // player is idle
                    // then player will jump up
                    player.state |= PlayerState::JUMP_UP;
                    player.set_animation(JUMPING_POSE1, 0, 10);
                    player.velocity = Vec2::new(0.0, 12.0);                    
                } else if !player.state.check(
                    PlayerState::JUMP_UP
                        | PlayerState::DOUBLE_JUMP
                        | PlayerState::JUMP_FORWARD
                        | PlayerState::JUMP_BACKWARD
                ) && player.state.check(PlayerState::WALKING) {
                    if player.state.check(PlayerState::DIRECTION) {
                        // player is walking right
                        // then player will jump forward
                        player.state |= PlayerState::JUMP_FORWARD;
                        player.set_animation(JUMPING_POSE1, 0, 10);
                        let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                        player.velocity = Vec2::new(x_vel, 12.0);
                    } else {
                        // player is walking left
                        // then player will jump backward
                        player.state |= PlayerState::JUMP_BACKWARD;
                        player.set_animation(JUMPING_POSE1, 0, 10);
                        let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                        player.velocity = Vec2::new(-x_vel, 12.0);
                    }
                }
            }
        }
        if keys.just_pressed(KeyCode::KeyK) {
            if player.state.is_idle() {
                // player is idle
                // then player will kick
                player.state |= PlayerState::KICKING;
                player.set_animation(KICK_POSE, 0, 10);
            } else if player.state.check(PlayerState::JUMP_UP | PlayerState::DOUBLE_JUMP | PlayerState::JUMP_FORWARD | PlayerState::JUMP_BACKWARD) {
                // player is jumping
                // then just adding state
                player.state |= PlayerState::KICKING;
            }
        }
        if keys.just_pressed(KeyCode::KeyL) {
            if player.state.is_idle() {
                // player is idle
                // then player will punch
                player.state |= PlayerState::PUNCHING;
                player.set_animation(PUNCH_POSE, 0, 5);
            }
        }
        if keys.just_pressed(KeyCode::KeyJ) {
            if player.state.is_idle() {
                // player is idle
                // then player will special kick
                player.state |= PlayerState::KICKING | PlayerState::SPECIAL_ATTACK;
                player.set_animation(HIGH_KICK_POSE, 0, 10);
            } else if player.state.check(
                PlayerState::JUMP_UP
                | PlayerState::DOUBLE_JUMP
                | PlayerState::JUMP_FORWARD
                | PlayerState::JUMP_BACKWARD
            ) {
                // player is jumping
                // then just adding state
                player.state |= PlayerState::KICKING;
            }
        }
        if keys.just_pressed(KeyCode::KeyH) {
            if player.state.is_idle() {
                // player is idle
                // then player will special punch
                player.state |= PlayerState::PUNCHING | PlayerState::SPECIAL_ATTACK;
                player.set_animation(UPPER_PUNCH_POSE1, 0, 5);
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
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    player_collision: Res<PlayerCollision>,
    mut gamestate: ResMut<GameState>,
    mut timer: ResMut<AnimationTimer>,
    mut player_query: Query<(&mut Player, &PlayerID, &mut Transform), Without<BackGround>>,
    mut ground_query: Query<&mut Transform, (With<BackGround>, Without<Player>)>,
) {
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        for (mut player, id, mut transform) in player_query.iter_mut() {
            // when game phase is 6(gameover), player will perform the loser and winner pose
            if gamestate.phase == 6 && player.animation.count != 0 {
                player.animation.count -= 1;
                let diff_pose = player.animation.diff_pose;
                player.pose += diff_pose;
                if gamestate.winners[gamestate.round as usize - 1] != id.0 + 1 {
                    transform.translation.y -= 15.0;
                }
                if player.animation.count == 0 {
                    player.animation.phase = 1;
                    commands.remove_resource::<Fighting>();
                    gamestate.phase = 7;
                    gamestate.count = 0;
                }
                continue
            }
            // player is idle
            if player.state.is_idle() {
                player.velocity = Vec2::ZERO;
                if player.animation.phase == 0 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        if player.state.check(PlayerState::COOLDOWN) {
                            player.state &= !PlayerState::COOLDOWN;
                        }
                        player.set_animation(IDLE_POSE2, 1, 15);
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.set_animation(IDLE_POSE1, 2, 15);
                    }
                } else if player.animation.phase == 2 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.set_animation(IDLE_POSE2, 1, 15);
                    }
                }
            }
            if player.state.check(PlayerState::JUMP_UP | PlayerState::DOUBLE_JUMP | PlayerState::JUMP_FORWARD | PlayerState::JUMP_BACKWARD) {
                // player is jumping
                player.velocity -= Vec2::new(0.0, GRAVITY_ACCEL * 2.0 / FPS);
                if player.state.check(PlayerState::KICKING) {
                    if player.animation.phase == 0 {
                        player.update_animation();
                        if player.animation.count == 0 {
                            player.animation.phase = 1;
                            player.animation.count = 0;
                        }
                    }
                } else {
                    if player.animation.phase == 0 {
                        player.update_animation();
                        if player.animation.count == 0 {
                            player.set_animation(JUMPING_POSE2, 1, 5);
                        }
                    } else if player.animation.phase == 1 {
                        player.update_animation();
                        if player.animation.count == 0 {
                            player.animation.phase = 2;
                            player.animation.count = 0;
                        }
                    } else if player.animation.phase == 2 {
                        player.update_animation();
                    }
                }
            } else if player.state.check(PlayerState::BEND_DOWN) {
                // player is bending down
                if player.animation.phase == 0 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                        player.set_animation(IDLE_POSE1, 0, 10);
                    }
                }
            } else if player.state.check(PlayerState::ROLL_FORWARD) {
                // player is rolling forward
                if player.animation.phase == 0 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_FORWARD_POSE2, 1, 10);
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_FORWARD_POSE3, 2, 10);
                    }
                } else if player.animation.phase == 2 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.pose.body = 0.0;
                        player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                        player.set_animation(IDLE_POSE1, 0, 10);
                    }
                }
            } else if player.state.check(PlayerState::ROLL_BACK) {
                // player is rolling back
                if player.animation.phase == 0 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_BACK_POSE2, 1, 10);
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_BACK_POSE3, 2, 10);
                    }
                } else if player.animation.phase == 2 {
                    player.update_animation();
                    if player.animation.count == 0 {
                        player.pose.body = 0.0;
                        player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                        player.set_animation(IDLE_POSE1, 0, 10);
                    }
                }
            } else {
                if player.state.check(PlayerState::KICKING) {
                    if player.state.check(PlayerState::SPECIAL_ATTACK) {
                        if player.animation.phase == 0 {
                            player.update_animation();
                            if player.animation.count == 0 {
                                player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                                player.set_animation(IDLE_POSE1, 0, 30);
                            }
                        }
                    } else {
                        if player.animation.phase == 0 {
                            player.update_animation();
                            if player.animation.count == 0 {
                                player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                                player.set_animation(IDLE_POSE1, 0, 30);
                            }
                        }
                    }
                }
                if player.state.check(PlayerState::PUNCHING) {
                    if player.state.check(PlayerState::SPECIAL_ATTACK) {
                        if player.animation.phase == 0 {
                            player.update_animation();
                            if player.animation.count == 0 {
                                player.set_animation(UPPER_PUNCH_POSE2, 1, 5);
                            }
                        } else if player.animation.phase == 1 {
                            player.update_animation();
                            if player.animation.count == 0 {
                                player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                                player.set_animation(IDLE_POSE1, 0, 30);
                            }
                        }
                    } else {
                        if player.animation.phase == 0 {
                            player.update_animation();
                            if player.animation.count == 0 {
                                player.state = PlayerState::IDLE | PlayerState::COOLDOWN;
                                player.set_animation(IDLE_POSE1, 0, 30);
                            }
                        }
                    }
                }
                if player.state.check(PlayerState::WALKING) {
                    if player.state.is_forward() && player.velocity.x < CHARACTER_PROFILES[player.character_id as usize].agility {
                        player.velocity += Vec2::new(1.0, 0.0) * PIXELS_PER_METER / FPS;
                    } else if !player.state.is_forward() && player.velocity.x > -CHARACTER_PROFILES[player.character_id as usize].agility {
                        player.velocity += Vec2::new(-1.0, 0.0) * PIXELS_PER_METER / FPS;
                    }
                    if player.velocity.x > CHARACTER_PROFILES[player.character_id as usize].agility {
                        player.velocity.x = CHARACTER_PROFILES[player.character_id as usize].agility;
                    } else if player.velocity.x < -CHARACTER_PROFILES[player.character_id as usize].agility {
                        player.velocity.x = -CHARACTER_PROFILES[player.character_id as usize].agility;
                    }
                    if player.animation.phase == 0 {
                        player.update_animation();
                        if player.animation.count == 0 {
                            player.set_animation(WALKING_POSE2, 1, 15);
                        }
                    } else if player.animation.phase == 1 {
                        player.update_animation();
                        if player.animation.count == 0 {
                            player.set_animation(WALKING_POSE1, 0, 15);
                        }
                    } else if player.animation.phase == 2 {
                        player.update_animation();
                        if player.animation.count == 0 {
                            player.set_animation(WALKING_POSE2, 1, 15);
                        }
                    }
                }
            }
            if player_collision.0 == 2 {
                // no collision, player moves freely
                transform.translation += Vec3::new(player.velocity.x, player.velocity.y, 0.0) * PIXELS_PER_METER / FPS;
            } else {
                // collision, player cannot move along x-axis
                transform.translation += Vec3::new(0.0, player.velocity.y, 0.0) * PIXELS_PER_METER / FPS;
            }
        }

        /*
        // move player and ground
        */
        let mut ground = ground_query.get_single_mut().unwrap();
        
        // Check if players are at opposite ends of the screen
        // 0 means player isn't at edge, 1 means player is at left edge, 2 means player is at right edge
        let mut at_edges: [u8;2] = [0;2];
        for (_, player_id, transform) in player_query.iter() {
            if transform.translation.x < -config.window_size.x / 2.0 + 100.0 {
                at_edges[player_id.0 as usize] = 1;
            } else if transform.translation.x > config.window_size.x / 2.0 - 100.0 {
                at_edges[player_id.0 as usize] = 2; 
            } else if transform.translation.x == -config.window_size.x / 2.0 + 100.0 {
                at_edges[player_id.0 as usize] = 3;
            } else if transform.translation.x == config.window_size.x / 2.0 - 100.0 {
                at_edges[player_id.0 as usize] = 4;
            }
        }

        if (at_edges[0] == 1 || at_edges[0] == 2) && at_edges[1] == 0 {
            let mut diff = 0.0;
            if let Some((_, _, mut transform)) = player_query.iter_mut().find(|(_, player_id, _)| player_id.0 == 0) {
                if at_edges[0] == 1 {
                    // If player 0 is at the left edge, move camera to the left and move players to the right
                    // transform.translation.x - (-config.window_size.x / 2.0 + 100.0): difference between player 0 and left edge
                    diff = -config.window_size.x / 2.0 + 100.0 - transform.translation.x;
                    ground.translation.x += diff;
                    transform.translation.x = -config.window_size.x / 2.0 + 100.0;
                } else {
                    // If player 0 is at the right edge, move camera to the right and move players to the left
                    // transform.translation.x - (config.window_size.x / 2.0 - 100.0): difference between player 0 and right edge
                    diff = config.window_size.x / 2.0 - 100.0 - transform.translation.x;
                    ground.translation.x += diff;
                    transform.translation.x = config.window_size.x / 2.0 - 100.0;
                }
            }
            if ground.translation.x < config.window_size.x / 2.0 - 2000.0 {
                ground.translation.x = config.window_size.x / 2.0 - 2000.0;
            } else if ground.translation.x > 2000.0 - config.window_size.x / 2.0 {
                ground.translation.x = 2000.0 - config.window_size.x / 2.0;
            } else if let Some((_, _, mut transform)) = player_query.iter_mut().find(|(_, player_id, _)| player_id.0 == 1) {
                transform.translation.x += diff;
            }
        } else if at_edges[0] == 0 && (at_edges[1] == 1 || at_edges[1] == 2) {
            let mut diff = 0.0;
            if let Some((_, _, mut transform)) = player_query.iter_mut().find(|(_, player_id, _)| player_id.0 == 1) {
                if at_edges[1] == 1 {
                    // If player 0 is at the left edge, move camera to the left and move players to the right
                    // transform.translation.x - (-config.window_size.x / 2.0 + 100.0): difference between player 0 and left edge
                    diff = -config.window_size.x / 2.0 + 100.0 - transform.translation.x;
                    ground.translation.x += diff;
                    transform.translation.x = -config.window_size.x / 2.0 + 100.0;
                } else {
                    // If player 0 is at the right edge, move camera to the right and move players to the left
                    // transform.translation.x - (config.window_size.x / 2.0 - 100.0): difference between player 0 and right edge
                    diff = config.window_size.x / 2.0 - 100.0 - transform.translation.x;
                    ground.translation.x += diff;
                    transform.translation.x = config.window_size.x / 2.0 - 100.0;
                }
            }
            if ground.translation.x < config.window_size.x / 2.0 - 2000.0 {
                ground.translation.x = config.window_size.x / 2.0 - 2000.0;
            } else if ground.translation.x > 2000.0 - config.window_size.x / 2.0 {
                ground.translation.x = 2000.0 - config.window_size.x / 2.0;
            } else if let Some((_, _, mut transform)) = player_query.iter_mut().find(|(_, player_id, _)| player_id.0 == 0) {
                transform.translation.x += diff;
            }
        } else if at_edges[0] != 0 && at_edges[1] != 0 {
            if (at_edges[0] == 1 && at_edges[1] == 3) || (at_edges[0] == 3 && at_edges[1] == 1) {
                // If both players are at the same edge, move the camera to the edge
                let mut diff = 0.0;
                if let Some((_, _, mut transform)) = player_query.iter_mut().find(|(_, player_id, _)| player_id.0 == if at_edges[0] == 1 { 0 } else { 1 }) {
                    diff = -config.window_size.x / 2.0 + 100.0 - transform.translation.x;
                    transform.translation.x = -config.window_size.x / 2.0 + 100.0;
                }
                ground.translation.x += diff;
                if let Some((_, _, mut transform)) = player_query.iter_mut().find(|(_, player_id, _)| player_id.0 == if at_edges[0] == 1 { 1 } else { 0 }) {
                    transform.translation.x += diff;
                }
            } else if (at_edges[0] == 2 && at_edges[1] == 4) || (at_edges[0] == 4 && at_edges[1] == 2) {
                // If both players are at the same edge, move the camera to the edge
                let mut diff = 0.0;
                if let Some((_, _, mut transform)) = player_query.iter_mut().find(|(_, player_id, _)| player_id.0 == if at_edges[0] == 2 { 0 } else { 1 }) {
                    diff = config.window_size.x / 2.0 - 100.0 - transform.translation.x;
                    transform.translation.x = config.window_size.x / 2.0 - 100.0;
                }
                ground.translation.x += diff;
                if let Some((_, _, mut transform)) = player_query.iter_mut().find(|(_, player_id, _)| player_id.0 == if at_edges[0] == 2 { 1 } else { 0 }) {
                    transform.translation.x += diff;
                }
            } else {
                for (_, player_id, mut transform) in player_query.iter_mut() {
                    // If players are at opposite edges, don't move
                    if at_edges[player_id.0 as usize] == 1 || at_edges[player_id.0 as usize] == 3 {
                        transform.translation.x = -config.window_size.x / 2.0 + 100.0;
                    } else {
                        transform.translation.x = config.window_size.x / 2.0 - 100.0;
                    }
                }
            }
        }
    }
}

// check if the player is grounding
#[cfg(not(target_arch = "wasm32"))]
fn check_ground(
    config: Res<GameConfig>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, mut transform) in player_query.iter_mut() {
        if player.state.check(
            PlayerState::JUMP_UP
            | PlayerState::DOUBLE_JUMP
            | PlayerState::JUMP_BACKWARD
            | PlayerState::JUMP_FORWARD
        ) && transform.translation.y - player.pose.offset[1] < 270.0-config.window_size.y/2.0 {
            player.state &= !(PlayerState::JUMP_UP | PlayerState::DOUBLE_JUMP | PlayerState::JUMP_BACKWARD | PlayerState::JUMP_FORWARD);
            player.set_animation(IDLE_POSE1, 0, 10);
            transform.translation.y = 270.0 - config.window_size.y/2.0 + player.pose.offset[1];
            player.velocity = Vec2::ZERO;
        }
    }
}
// check if the player is grounding
#[cfg(target_arch = "wasm32")]
fn check_ground(
    config: Res<GameConfig>,
    mut double_jump_check: ResMut<DoubleJumpCheck>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, mut transform) in player_query.iter_mut() {
        if player.state.check(
            PlayerState::JUMP_UP
            | PlayerState::DOUBLE_JUMP
            | PlayerState::JUMP_BACKWARD
            | PlayerState::JUMP_FORWARD
        ) && transform.translation.y - player.pose.offset[1] < 135.0-config.window_size.y/2.0 {
            player.state &= !(PlayerState::JUMP_UP | PlayerState::DOUBLE_JUMP | PlayerState::JUMP_BACKWARD | PlayerState::JUMP_FORWARD);
            player.set_animation(IDLE_POSE1, 0, 10);
            transform.translation.y = 135.0 - config.window_size.y/2.0 + player.pose.offset[1];
            player.velocity = Vec2::ZERO;

            double_jump_check.reset();
        }
    }
}

/// Updates the pose of the player character based on their current state.
fn update_pose(
    mut player_query: Query<(&mut Player, &mut Transform, &PlayerID), (Without<BodyParts>, Without<Foot>)>,
    mut parts_query: Query<(&BodyParts, &PlayerID, &mut Transform), (Without<Player>, Without<Foot>)>,
    mut foot_query: Query<(&Foot, &PlayerID, &mut Transform), (Without<Player>, Without<BodyParts>)>,
) {
    for (mut player, mut player_transform, player_id) in player_query.iter_mut() {
        let flip = if player.pose.facing { 1.0 } else { -1.0 };
        for (parts, parts_id, mut transform) in parts_query.iter_mut() {
            if player_id.0 == parts_id.0 {
                match parts.flags {
                    // Head
                    0b10000 => rotate_parts(&mut transform, 0.0, HEAD_OFFSET, flip * player.pose.head),
                    // Body
                    0b01000 => rotate_parts(&mut transform, 0.0, BODY_OFFSET, flip * player.pose.body),
                    // Right Upper Arm
                    0b00111 => rotate_parts(&mut transform, -2.0 * flip * BODY_THICKNESS, UPPER_ARM_OFFSET, flip * player.pose.right_upper_arm),
                    // Right Lower Arm
                    0b00110 => rotate_parts(&mut transform, 0.0, LOWER_ARM_OFFSET, flip * player.pose.right_lower_arm),
                    // Right Upper Leg
                    0b00011 => rotate_parts(&mut transform, -flip * BODY_THICKNESS, UPPER_LEG_OFFSET, flip * player.pose.right_upper_leg),
                    // Right Lower Leg
                    0b00010 => rotate_parts(&mut transform, 0.0, LOWER_LEG_OFFSET, flip * player.pose.right_lower_leg),
                    // Left Upper Arm
                    0b00101 => rotate_parts(&mut transform, 2.0 * flip * BODY_THICKNESS, UPPER_ARM_OFFSET, flip * player.pose.left_upper_arm),
                    // Left Lower Arm
                    0b00100 => rotate_parts(&mut transform, 0.0, LOWER_ARM_OFFSET, flip * player.pose.left_lower_arm),
                    // Left Upper Leg
                    0b00001 => rotate_parts(&mut transform, flip * BODY_THICKNESS, UPPER_LEG_OFFSET, flip * player.pose.left_upper_leg),
                    // Left Lower Leg
                    0b00000 => rotate_parts(&mut transform, 0.0, LOWER_LEG_OFFSET, flip * player.pose.left_lower_leg),
                    _ => {}
                }
            }
        }
        // update player position offset
        player_transform.translation.x += player.pose.offset[0] - player.pose.old_offset[0];
        player_transform.translation.y += player.pose.offset[1] - player.pose.old_offset[1];
        player.pose.old_offset = player.pose.offset;

        // update foot position
        for (foot, foot_id, mut transform) in foot_query.iter_mut() {
            if player_id.0 == foot_id.0 {
                if foot.0 {
                    transform.translation.x += player.pose.foot_offset[0] - player.pose.old_foot_offset[0];
                    transform.translation.y += player.pose.foot_offset[1] - player.pose.old_foot_offset[1];
                } else {
                    transform.translation.x += player.pose.foot_offset[2] - player.pose.old_foot_offset[2];
                    transform.translation.y += player.pose.foot_offset[3] - player.pose.old_foot_offset[3];
                }
            }
        }
        player.pose.old_foot_offset = player.pose.foot_offset;
    }
}

fn check_attack(
    mut player_collision: ResMut<PlayerCollision>,
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
                if attacker_id == PlayerID(2) {
                    if player_collision.0 != 2 { continue; }
                    // No attacker found
                    // If the collision is between one body and another body part, move player to avoid collision
                    if parts1.is_body() && !parts2.is_body() {
                        for (player, player_id) in player_query.iter() {
                            if player_id == id2 {
                                if player.state.is_idle() {
                                    player_collision.0 = id1.0;
                                } else {
                                    player_collision.0 = id2.0;
                                }
                            }
                        }
                    } else if parts2.is_body() &&  !parts1.is_body() {
                        for (player, player_id) in player_query.iter() {
                            if player_id == id1 {
                                if player.state.is_idle() {
                                    player_collision.0 = id2.0;
                                } else {
                                    player_collision.0 = id1.0;
                                }
                            }
                        }                   
                    }
                    continue;
                }

                let mut damage: u32 = 0;
                if let Some((mut player, _)) = player_query.iter_mut().find(|(_, id)| id.0 == attacker_id.0) {
                    damage = calculate_damage(
                        player_info[attacker_id.0 as usize],
                        player_info[opponent_id.0 as usize],
                        opponent_parts,
                    );
                    println!("Player {} hit: {} damage", attacker_id.0, damage);
                    player.state &= !(PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::SPECIAL_ATTACK);
                    player.set_animation(IDLE_POSE1, 0, 30);
                }
                if let Some((mut player, _)) = player_query.iter_mut().find(|(_, id)| id.0 == opponent_id.0) {
                    player.health = player.health.saturating_sub(damage);
                }
            }
            CollisionEvent::Stopped(entity1, entity2, _) => {
                let Ok((parts1, id1)) = parts_query.get(*entity1) else {
                    continue;
                };
                let Ok((parts2, id2)) = parts_query.get(*entity2) else {
                    continue;
                };

                // Check if the collision is between two player characters
                if id1 == id2 { continue }

                // Check if the collision is between one body and another body part
                if parts1.is_body() != parts2.is_body() {
                    player_collision.0 = 2;
                }
            }
        }
    }
}

fn avoid_collision(
    player_collision: Res<PlayerCollision>,
    mut player_query: Query<(&Player, &PlayerID, &mut Transform)>,
) {
    // no collision
    if player_collision.0 == 2 {
        return;
    }
    // move player to avoid collision
    if let Some((player, _, mut transform)) = player_query.iter_mut().find(|(_, id, _)| id.0 == player_collision.0) {
        transform.translation.x += if player.pose.facing { -1.0 } else { 1.0 };
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
    if attacker_info.1.check(PlayerState::JUMP_UP | PlayerState::DOUBLE_JUMP | PlayerState::JUMP_FORWARD | PlayerState::JUMP_BACKWARD) {
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
                        if cfg!(target_arch = "wasm32") {
                            positions[2][0] = health_bar.1 * health_bar.0 + if player_id.0 == 0 { 25.0 } else { -25.0 };
                        } else {
                            positions[2][0] = health_bar.1 * health_bar.0 + if player_id.0 == 0 { 50.0 } else { -50.0 };
                        }
                    }
                }
            }
        }
    }
}

fn update_facing(
    mut player_query: Query<(&mut Player, &PlayerID, &Transform)>,
) {
    let mut positions = [0.0;2];
    for (_, player_id, transform) in player_query.iter_mut() {
        positions[player_id.0 as usize] = transform.translation.x;
    }
    for (mut player, player_id, _) in player_query.iter_mut() {
        if !player.state.check(!(PlayerState::COOLDOWN | PlayerState::DIRECTION | PlayerState::WALKING)) {
            if player_id.0 == 0 {
                if positions[0] < positions[1] {
                    player.pose.facing = true;
                } else {
                    player.pose.facing = false;
                }
            } else {
                if positions[1] < positions[0] {
                    player.pose.facing = true;
                } else {
                    player.pose.facing = false;
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
            .insert_resource(PlayerCollision(2))
            .add_systems(Update, player_movement.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, check_ground.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, update_pose.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, check_attack.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, avoid_collision.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, update_health_bar.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, update_facing.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)));

        #[cfg(not(target_arch = "wasm32"))]
        app
            .add_systems(Update, keyboard_input.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)));
    }
}