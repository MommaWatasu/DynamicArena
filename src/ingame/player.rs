use super::{pose::*, rand, BackGround, Fighting, SkillEntity, SkillName};
#[cfg(not(target_arch = "wasm32"))]
use crate::GameMode;
use crate::{
    character_def::*,
    CharacterTextures,
    ingame::{GameState, InGame, DamageDisplay},
    AppState, GameConfig, SoundEffect, PATH_SOUND_PREFIX, PATH_IMAGE_PREFIX
};
use bevy::{prelude::*, asset::RenderAssetUsages, render::mesh::{VertexAttributeValues, PrimitiveTopology, Indices}};
use bevy_rapier2d::prelude::*;
use std::{
    fmt::Debug,
    ops::{BitAndAssign, BitOr, BitOrAssign, Not},
};

// definition for normal display
#[cfg(not(target_arch = "wasm32"))]
const UPPER_ARM_LENGTH: f32 = 20.0;
#[cfg(not(target_arch = "wasm32"))]
const UPPER_LEG_LENGTH: f32 = 40.0;
#[cfg(not(target_arch = "wasm32"))]
const LIMB_LENGTH: f32 = 30.0;
#[cfg(not(target_arch = "wasm32"))]
const NECK_LENGTH: f32 = 40.0;
#[cfg(not(target_arch = "wasm32"))]
const LIMB_RADIUS: f32 = 10.0;
#[cfg(not(target_arch = "wasm32"))]
const BODY_THICKNESS: f32 = 10.0;
#[cfg(not(target_arch = "wasm32"))]
const BODY_LENGTH: f32 = 65.0;
#[cfg(not(target_arch = "wasm32"))]
const HEAD_OFFSET: f32 = 80.0;
#[cfg(not(target_arch = "wasm32"))]
const BODY_OFFSET: f32 = -20.0;
#[cfg(not(target_arch = "wasm32"))]
const UPPER_ARM_OFFSET: f32 = 0.0;
#[cfg(not(target_arch = "wasm32"))]
const LOWER_ARM_OFFSET: f32 = -50.0;
#[cfg(not(target_arch = "wasm32"))]
const UPPER_LEG_OFFSET: f32 = -90.0;
#[cfg(not(target_arch = "wasm32"))]
const LOWER_LEG_OFFSET: f32 = -70.0;

// definition for web display
#[cfg(target_arch = "wasm32")]
const UPPER_ARM_LENGTH: f32 = 10.0;
#[cfg(target_arch = "wasm32")]
const LOWER_ARM_LENGTH: f32 = 20.0;
#[cfg(target_arch = "wasm32")]
const LIMB_LENGTH: f32 = 15.0;
#[cfg(target_arch = "wasm32")]
const NECK_LENGTH: f32 = 20.0;
#[cfg(target_arch = "wasm32")]
const LIMB_RADIUS: f32 = 7.5;
#[cfg(target_arch = "wasm32")]
const BODY_THICKNESS: f32 = 5.0;
#[cfg(target_arch = "wasm32")]
const HEAD_OFFSET: f32 = 40.0;
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

#[derive(Component)]
pub struct EnergyBar(pub f32, pub f32);

#[derive(Component)]
pub struct FireBar(pub f32, pub f32);

#[derive(Component)]
struct FireAnimation {
    facing: bool,
}

#[derive(Resource)]
pub struct SoulAbsorb;

/// Represents the current state of a player using bit flags.
/// Multiple states can be active simultaneously by combining flags with bitwise OR.
///
/// | State           | Bit Pattern        | Description                         |
/// |-----------------|--------------------|-------------------------------------|
/// | IDLE            | 0b0000000000000000 | Default state, no action            |
/// | WALKING         | 0b0000000000000001 | Player is moving horizontally       |
/// | JUMP_UP         | 0b0000000000000010 | Player is in first jump             |
/// | SKILL           | 0b0000000000000100 | Player is performing skill attack   |
/// | KICKING         | 0b0000000000001000 | Player is performing kick           |
/// | PUNCHING        | 0b0000000000010000 | Player is performing punch          |
/// | FIRE_EMISSION   | 0b0000000000100000 | Player is performing ranged attack  |
/// | BACK_KICKING    | 0b0000000001000000 | Player is performing back kick      |
/// | COOLDOWN        | 0b0000000010000000 | Player is in cooldown state         |
/// | DIRECTION       | 0b0000000100000000 | Player is moving right              |
/// | JUMP_FORWARD    | 0b0000001000000000 | Player is jumping forward           |
/// | JUMP_BACKWARD   | 0b0000010000000000 | Player is jumping backward          |
/// | BEND_DOWN       | 0b0000100000000000 | Player is bending down              |
/// | ROLL_BACK       | 0b0001000000000000 | Player is rolling back              |
/// | ROLL_FORWARD    | 0b0010000000000000 | Player is rolling forward           |
/// | ATTACK_DISABLED | 0b0100000000000000 | Player is in attack cooldown state  |
/// | STUN            | 0b1000000000000000 | Player is stunned                   |
#[derive(PartialEq, Eq, Copy, Clone, Default)]
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

impl Debug for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let states = [
            (0x0000, "IDLE"),
            (0x0001, "WALKING"),
            (0x0002, "JUMP_UP"),
            (0x0004, "SKILL"),
            (0x0008, "KICKING"),
            (0x0010, "PUNCHING"),
            (0x0020, "RANGED_ATTACK"),
            (0x0040, "BACK_KICKING"),
            (0x0080, "COOLDOWN"),
            (0x0100, "DIRECTION"),
            (0x0200, "JUMP_FORWARD"),
            (0x0400, "JUMP_BACKWARD"),
            (0x0800, "BEND_DOWN"),
            (0x1000, "ROLL_BACK"),
            (0x2000, "ROLL_FORWARD"),
            (0x4000, "ATTACK_DISABLED"),
            (0x8000, "STUN"),
        ];

        let active_states: Vec<&str> = states.iter()
            .filter(|(flag, _)| flag > &0 && (self.0 & *flag as u16) != 0)
            .map(|(_, name)| *name)
            .collect();

        if f.alternate() {
            // detailed format (using #?)
            write!(f, "{}", active_states.join(" | "))?;
            write!(f, " (0b{:016b})", self.0)
        } else {
            // normal format
            write!(f, "{}", active_states.join("|"))
        }
    }
}

impl PlayerState {
    pub const IDLE: Self = Self(0b0000000000000000);
    pub const WALKING: Self = Self(0b0000000000000001);
    pub const JUMP_UP: Self = Self(0b0000000000000010);
    pub const SKILL: Self = Self(0b0000000000000100);
    pub const KICKING: Self = Self(0b0000000000001000);
    pub const PUNCHING: Self = Self(0b0000000000010000);
    pub const RANGED_ATTACK: Self = Self(0b0000000000100000);
    pub const BACK_KICKING: Self = Self(0b0000000001000000);
    pub const COOLDOWN: Self = Self(0b0000000010000000);
    pub const DIRECTION: Self = Self(0b0000000100000000);
    pub const JUMP_FORWARD: Self = Self(0b0000001000000000);
    pub const JUMP_BACKWARD: Self = Self(0b0000010000000000);
    pub const BEND_DOWN: Self = Self(0b0000100000000000);
    pub const ROLL_BACK: Self = Self(0b0001000000000000);
    pub const ROLL_FORWARD: Self = Self(0b0010000000000000);
    pub const ATTACK_DISABLED: Self = Self(0b0100000000000000);
    pub const STUN:Self = Self(0b1000000000000000);

    // ignore cooldown state
    pub fn is_idle(&self) -> bool {
        self.0 & !(Self::COOLDOWN.0 | Self::DIRECTION.0 | Self::ATTACK_DISABLED.0) == 0
    }
    pub fn is_just_walk(&self) -> bool {
        self.0 & !(Self::COOLDOWN.0 | Self::DIRECTION.0 | Self::ATTACK_DISABLED.0 | Self::WALKING.0) == 0
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
    timer: Timer,
}

pub struct PlayerColliderAnimation {
    diff_pose: Pose,
    diff_y: f32,
    pub phase: u8,
    pub count: u8,
}

#[derive(Component)]
pub struct Player {
    pub character_id: isize,
    pub pose: Pose,
    pub animation: PlayerColliderAnimation,
    pub animation_frame_max: usize,
    pub state: PlayerState,
    pub velocity: Vec2,
    pub health: u32,
    pub energy: u8,
    pub fire_charge: u16,
    pub stun_count: u16
}

impl Player {
    pub fn new(character_id: isize) -> Self {
        Self {
            character_id,
            pose: IDLE_POSE1,
            animation: PlayerColliderAnimation {
                diff_pose: default(),
                diff_y: 0.0,
                phase: 1,
                count: 10,
            },
            animation_frame_max: FRAMES_IDLE,
            state: PlayerState::default(),
            velocity: Vec2::ZERO,
            health: CHARACTER_PROFILES[character_id as usize].health,
            energy: 0,
            fire_charge: FIRE_CHARGE_MAX,
            stun_count: 3,
        }
    }
    pub fn new_opposite(character_id: isize) -> Self {
        Self {
            character_id,
            pose: OPPOSITE_DEFAULT_POSE,
            animation: PlayerColliderAnimation {
                diff_pose: default(),
                diff_y: 0.0,
                phase: 1,
                count: 10,
            },
            animation_frame_max: FRAMES_IDLE,
            state: PlayerState::default(),
            velocity: Vec2::ZERO,
            health: CHARACTER_PROFILES[character_id as usize].health,
            energy: 0,
            fire_charge: FIRE_CHARGE_MAX,
            stun_count: 3,
        }
    }
    pub fn reset(&mut self, id: &PlayerID) {
        if id.0 == 0 {
            self.pose = IDLE_POSE1;
        } else {
            self.pose = OPPOSITE_DEFAULT_POSE;
        }
        self.animation = PlayerColliderAnimation {
            diff_pose: default(),
            diff_y: 0.0,
            phase: 1,
            count: 10,
        };
        self.state = PlayerState::default();
        self.velocity = Vec2::ZERO;
        self.health = CHARACTER_PROFILES[self.character_id as usize].health;
        self.fire_charge = FIRE_CHARGE_MAX;
    }
    pub fn set_animation(&mut self, pose: Pose, phase: u8, count: u8) {
        let real_count =
            (count as f32 / CHARACTER_PROFILES[self.character_id as usize].dexterity).round();
        self.animation = PlayerColliderAnimation {
            diff_pose: (pose - self.pose) / real_count,
            diff_y: 0.0,
            phase,
            count: real_count as u8,
        };
    }
    pub fn update_animation(&mut self, sprite: &mut Sprite) {
        if self.animation.count == 0 {
            return;
        }
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            if self.state.check(PlayerState::ROLL_BACK)
                || (self.state.check(PlayerState::WALKING | PlayerState::DIRECTION) && !self.pose.facing)
                || (self.state.check(PlayerState::WALKING) && !self.state.check(PlayerState::DIRECTION) && self.pose.facing) {
                atlas.index -= 1;
                if atlas.index == 0 {
                    atlas.index = self.animation_frame_max - 1;
                }
            } else {
                atlas.index += 1;
                if atlas.index == self.animation_frame_max - 1 {
                    atlas.index = 0;
                }
            }
        }
        self.pose += self.animation.diff_pose;
        self.animation.count -= 1;
    }
    pub fn update_animation_idle(&mut self, transform: &mut Transform, sprite: &mut Sprite) {
        if self.animation.count == 0 {
            return;
        }
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index += 1;
        }
        self.pose += self.animation.diff_pose;
        self.animation.count -= 1;
        transform.translation.y += self.animation.diff_y;
    }
}

#[derive(Component)]
struct BodyParts {
    flags: u8,
}

#[allow(dead_code)]
impl BodyParts {
    const NULL: Self = Self { flags: 0b00000 };
    const HEAD: Self = Self { flags: 0b10000 };
    const BODY: Self = Self { flags: 0b01000 };
    pub fn new(head: bool, body: bool, arm: bool, right: bool, upper: bool) -> Self {
        Self {
            flags: (head as u8) << 4
                | (body as u8) << 3
                | (arm as u8) << 2
                | (right as u8) << 1
                | (upper as u8),
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

impl Debug for BodyParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.flags {
            0b10000 => write!(f, "BodyParts: Head"),
            0b01000 => write!(f, "BodyParts: Body"),
            0b00111 => write!(f, "BodyParts: Right Upper Arm"),
            0b00110 => write!(f, "BodyParts: Right Lower Arm"),
            0b00011 => write!(f, "BodyParts: Right Upper Leg"),
            0b00010 => write!(f, "BodyParts: Right Lower Leg"),
            0b00101 => write!(f, "BodyParts: Left Upper Arm"),
            0b00100 => write!(f, "BodyParts: Left Lower Arm"),
            0b00001 => write!(f, "BodyParts: Left Upper Leg"),
            0b00000 => write!(f, "BodyParts: Left Lower Leg"),
            _ => write!(f, "BodyParts: Unkown"),
        }
    }
}

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
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    asset_server: &Res<AssetServer>,
    y_pos: f32,
) {
    // let profile = &CHARACTER_PROFILES[character_id as usize];

    // Load the sprite sheet using the `AssetServer`
    let texture = asset_server.load(format!("{}character{}/idle.png", PATH_IMAGE_PREFIX, character_id+1));

    // The sprite sheet has 30 sprites arranged in a row, and they are all 512px x 512px
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(512), 30, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    builder
        .spawn((
            if id == 0 {
                Player::new(character_id)
            } else {
                Player::new_opposite(character_id)
            },
            PlayerID(id),
            InGame,
            Sprite {
                image: texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                }),
                flip_x: if id == 0 {
                    false
                } else {
                    true
                },
                ..Default::default()
            },
            // Player 0 is on top of the screen
            #[cfg(not(target_arch = "wasm32"))]
            Transform::from_translation(Vec3::new(
                if id == 0 { -500.0 } else { 500.0 },
                y_pos,
                if id == 0 { 10.0 } else { 1.0 },
            )),
            #[cfg(target_arch = "wasm32")]
            Transform::from_translation(Vec3::new(
                if id == 0 { -250.0 } else { 250.0 },
                y_pos,
                if id == 0 { 10.0 } else { 1.0 },
            )),
            Visibility::Visible,
        ))
        // Body
        .with_children(|builder| {
            builder
                .spawn((
                    Transform::from_translation(Vec3::new(10.0, BODY_OFFSET, 0.0)),
                    BodyParts::BODY,
                    PlayerID(id),
                    #[cfg(not(target_arch = "wasm32"))]
                    Collider::cuboid(BODY_THICKNESS * 2.0, BODY_LENGTH),
                    #[cfg(target_arch = "wasm32")]
                    Collider::cuboid(BODY_THICKNESS, 32.5),
                    RigidBody::KinematicPositionBased,
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                ))
                // Head and Neck
                .with_children(|builder| {
                    // Neck
                    // Neck is invisible(completely transparent)
                    builder
                        .spawn((
                            BodyParts::HEAD,
                            PlayerID(id),
                            Transform::from_translation(Vec3::new(0.0, HEAD_OFFSET, 2.0)),
                        ))
                        // Head
                        .with_child((
                            //BodyParts::HEAD,
                            #[cfg(not(target_arch = "wasm32"))]
                            Transform::from_translation(Vec3::new(0.0, 20.0, -1.0)),
                            #[cfg(target_arch = "wasm32")]
                            Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
                            RigidBody::KinematicPositionBased,
                            #[cfg(not(target_arch = "wasm32"))]
                            Collider::ball(40.0),
                            #[cfg(target_arch = "wasm32")]
                            Collider::ball(20.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default()
                                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                        ));
                    // Right Upper Arm
                    builder
                        .spawn((
                            BodyParts::new(false, false, true, true, true),
                            PlayerID(id),
                            // player 0 is right facing, and player 1 is left facing
                            // so we need to change which arm is on top
                            Transform::from_translation(Vec3::new(0.0, UPPER_ARM_OFFSET, 2.0)),
                            RigidBody::KinematicPositionBased,
                            Collider::capsule_y(UPPER_ARM_LENGTH, LIMB_RADIUS),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default()
                                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                        ))
                        // Right Lower Arm
                        .with_child((
                            BodyParts::new(false, false, true, true, false),
                            PlayerID(id),
                            Transform::from_translation(Vec3::new(0.0, LOWER_ARM_OFFSET, 2.0)),
                            RigidBody::KinematicPositionBased,
                            Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default()
                                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                        ));
                    // Left Upper Arm
                    builder
                        .spawn((
                            BodyParts::new(false, false, true, false, true),
                            PlayerID(id),
                            // player 0 is right facing, and player 1 is left facing
                            // so we need to change which arm is on top
                            Transform::from_translation(Vec3::new(0.0, UPPER_ARM_OFFSET, -1.0)),
                            RigidBody::KinematicPositionBased,
                            Collider::capsule_y(UPPER_ARM_LENGTH, LIMB_RADIUS),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default()
                                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                        ))
                        // Left Lower Arm
                        .with_child((
                            BodyParts::new(false, false, true, false, false),
                            PlayerID(id),
                            Transform::from_translation(Vec3::new(0.0, LOWER_ARM_OFFSET, 2.0)),
                            RigidBody::KinematicPositionBased,
                            Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default()
                                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                        ));
                    // Right Upper Leg
                    builder
                        .spawn((
                            // right upper leg
                            BodyParts::new(false, false, false, true, true),
                            PlayerID(id),
                            // player 0 is right facing, and player 1 is left facing
                            // so we need to change which leg is on top
                            Transform::from_translation(Vec3::new(10.0, UPPER_LEG_OFFSET, 3.0)),
                            RigidBody::KinematicPositionBased,
                            Collider::capsule_y(UPPER_LEG_LENGTH, LIMB_RADIUS),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default()
                                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                        ))
                        // Right Lower Leg
                        .with_children(|builder| {
                            builder
                                .spawn((
                                    // right lower leg
                                    BodyParts::new(false, false, false, true, false),
                                    PlayerID(id),
                                    Transform::from_translation(Vec3::new(
                                        0.0,
                                        LOWER_LEG_OFFSET,
                                        1.0,
                                    )),
                                    RigidBody::KinematicPositionBased,
                                    Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                                    ActiveEvents::COLLISION_EVENTS,
                                    ActiveCollisionTypes::default()
                                        | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                                ));
                        });
                    // Left Upper Leg
                    builder
                        .spawn((
                            BodyParts::new(false, false, false, false, true),
                            PlayerID(id),
                            Transform::from_translation(Vec3::new(-10.0, UPPER_LEG_OFFSET, 1.0)),
                            RigidBody::KinematicPositionBased,
                            Collider::capsule_y(UPPER_LEG_LENGTH, LIMB_RADIUS),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::default()
                                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                        ))
                        // Left Lower Leg
                        .with_children(|builder| {
                            builder
                                .spawn((
                                    BodyParts::new(false, false, false, false, false),
                                    PlayerID(id),
                                    Transform::from_translation(Vec3::new(
                                        0.0,
                                        LOWER_LEG_OFFSET,
                                        1.0,
                                    )),
                                    RigidBody::KinematicPositionBased,
                                    Collider::capsule_y(LIMB_LENGTH, LIMB_RADIUS),
                                    ActiveEvents::COLLISION_EVENTS,
                                    ActiveCollisionTypes::default()
                                        | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                                ));
                        });
                });
        });
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
///   - J key for front kicks
///   - H key for back kicks
///
/// The function updates player state and animations based on input,
/// handling state transitions and preventing invalid combinations
/// of moves. For multiplayer, it processes input for both players
/// unless in single player mode.
#[cfg(not(target_arch = "wasm32"))]
fn keyboard_input(
    mut commands: Commands,
    mut fighting: ResMut<Fighting>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    character_textures: Res<CharacterTextures>,
    mut player_query: Query<(&mut Player, &PlayerID, &mut Sprite)>,
) {
    if config.gamepads[0] != Entity::from_raw(0) {
        // if gamepad is enabled, we don't handle keyboard input
        return;
    }

    for (mut player, player_id, mut sprite) in player_query.iter_mut() {
        // skip player 1(opponent) in order to control player 0
        // this is for debugging purpose
        #[cfg(debug_assertions)]
        if player_id.0 == 1 {
            continue;
        }

        if player_id.0 == 1 && config.mode == GameMode::SinglePlayer {
            continue;
        }
        if player.state.check(PlayerState::COOLDOWN) {
            continue;
        }
        if keys.pressed(KeyCode::KeyD) {
            if player.state.is_idle() {
                // player is just walking
                sprite.image = character_textures.textures[player.character_id as usize].walk.clone();
                if player.pose.facing {
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                } else {
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = FRAMES_WALK - 1);
                }
                player.animation_frame_max = FRAMES_WALK;
                player.state |= PlayerState::WALKING;
                player.set_animation(WALKING_POSE1, 0, 15);
            }
            // direction is right
            player.state |= PlayerState::DIRECTION;
        } else if keys.pressed(KeyCode::KeyA) {
            if player.state.is_idle() {
                // player is just walking
                sprite.image = character_textures.textures[player.character_id as usize].walk.clone();
                if !player.pose.facing {
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                } else {
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = FRAMES_WALK - 1);
                }
                player.animation_frame_max = FRAMES_WALK;
                player.state |= PlayerState::WALKING;
                player.set_animation(WALKING_POSE1, 0, 15);
            }
            // direction is left
            player.state &= !PlayerState::DIRECTION;
        } else {
            // player is not walking
            if player.state.check(PlayerState::WALKING) {
                player.state &= !PlayerState::WALKING;
                if player.state.is_idle() {
                    sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_IDLE;
                    player.set_animation(IDLE_POSE1, 0, 10);
                }
            }
        }
        if keys.pressed(KeyCode::KeyS) {
            if player.state.is_idle() {
                // player is idle
                // then player will bend down
                sprite.image = character_textures.textures[player.character_id as usize].bend_down.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_BEND_DOWN;
                player.state |= PlayerState::BEND_DOWN;
                player.set_animation(BEND_DOWN_POSE1, 0, 5);
            } else if player.state.is_just_walk() && player.state.check(PlayerState::WALKING) {
                if player.pose.facing {
                    if player.state.check(PlayerState::DIRECTION) {
                        // player is walking right
                        // then player will roll forward
                        sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_ROLL;
                        player.state |= PlayerState::ROLL_FORWARD;
                        player.set_animation(ROLL_FORWARD_POSE1, 0, 10);
                    } else {
                        // player is walking left
                        // then player will roll back
                        sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = FRAMES_ROLL - 1);
                        player.animation_frame_max = FRAMES_ROLL;
                        player.state |= PlayerState::ROLL_BACK;
                        player.set_animation(ROLL_BACK_POSE1, 0, 10);
                    }
                } else {
                    if !player.state.check(PlayerState::DIRECTION) {
                        // player is walking right
                        // then player will roll forward
                        sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_ROLL;
                        player.state |= PlayerState::ROLL_FORWARD;
                        player.set_animation(ROLL_FORWARD_POSE1, 0, 10);
                    } else {
                        // player is walking left
                        // then player will roll back
                        player.state |= PlayerState::ROLL_BACK;
                        player.set_animation(ROLL_BACK_POSE1, 0, 10);
                    }
                }
                let x_vel = if player.state.is_forward() { 1.0 } else { -1.0 }
                    * CHARACTER_PROFILES[player.character_id as usize].agility * 2.0;
                player.velocity = Vec2::new(x_vel, 0.0);
            }
        } else if player.state.check(PlayerState::BEND_DOWN) {
            // player is bending down
            // then stop bending down
            sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
            player.animation_frame_max = FRAMES_IDLE;
            player.state &= !PlayerState::BEND_DOWN;
            player.set_animation(IDLE_POSE1, 0, 10);
        }
        // NOTE: this code block contains a lot of duplicate code
        //       I should refactor it later
        if keys.just_pressed(KeyCode::Space) {
            if player.state.is_idle() {
                // player is idle
                // then player will jump up
                sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_JUMP;
                player.state |= PlayerState::JUMP_UP;
                player.set_animation(JUMP_UP_POSE1, 0, 10);
                player.energy += 1;
            } else if player.state.is_just_walk() && player.state.check(PlayerState::WALKING)
            {
                if player.pose.facing {
                    if player.state.check(PlayerState::DIRECTION) {
                        // player is walking right
                        // then player will jump forward
                        sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_JUMP;
                        player.state |= PlayerState::JUMP_FORWARD;
                        player.set_animation(JUMP_FORWARD_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
                        player.energy += 1;
                    } else {
                        // player is walking left
                        // then player will jump backward
                        sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_JUMP;
                        player.state |= PlayerState::JUMP_BACKWARD;
                        player.set_animation(JUMP_UP_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
                        player.energy += 1;
                    }
                } else {
                    if !player.state.check(PlayerState::DIRECTION) {
                        // player is walking right
                        // then player will jump forward
                        sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_JUMP;
                        player.state |= PlayerState::JUMP_FORWARD;
                        player.set_animation(JUMP_FORWARD_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
                        player.energy += 1;
                    } else {
                        // player is walking left
                        // then player will jump backward
                        sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_JUMP;
                        player.state |= PlayerState::JUMP_BACKWARD;
                        player.set_animation(JUMP_UP_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
                        player.energy += 1;
                    }
                }
            }
        }
        if keys.just_pressed(KeyCode::KeyK) {
            if player.state.is_idle() {
                // player is idle
                // then player will kick
                sprite.image = character_textures.textures[player.character_id as usize].kick.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_KICK;
                player.state |= PlayerState::KICKING;
                player.set_animation(KICK_POSE2, 0, 21);
                player.energy += 2;
            } else if player
                .state
                .check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD)
                && !player.state.check(PlayerState::KICKING)
            {
                // player is jumping
                // then just adding state
                player.state |= PlayerState::KICKING;
                player.energy += 2;
            }
        }
        if keys.just_pressed(KeyCode::KeyL) {
            if player.state.is_idle() {
                // player is idle
                // then player will punch
                sprite.image = character_textures.textures[player.character_id as usize].punch.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_PUNCH;
                player.state |= PlayerState::PUNCHING;
                player.set_animation(PUNCH_POSE, 0, 32);
                player.energy += 2;
            }
        }
        if keys.just_pressed(KeyCode::KeyJ) {
            if player.state.is_idle() && player.fire_charge == FIRE_CHARGE_MAX {
                // player is idle
                // player will do ranged attack
                sprite.image = character_textures.textures[player.character_id as usize].punch.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_PUNCH;
                player.fire_charge = 0;
                player.state |= PlayerState::RANGED_ATTACK;
                player.set_animation(PUNCH_POSE, 0, 32);
                player.energy += 2;
            }
        }
        if keys.just_pressed(KeyCode::KeyH) {
            if player.state.is_idle() {
                // player is idle
                // then player will back kick
                sprite.image = character_textures.textures[player.character_id as usize].back_kick.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_BACK_KICK;
                player.state |= PlayerState::BACK_KICKING;
                player.set_animation(BACK_KICK_POSE1, 0, 5);
                player.energy += 2;
            }
        }
        if keys.just_pressed(KeyCode::KeyG) && player.energy == 100 {
            if player.state.is_idle() {
                // player is idle
                // then player will use skill
                player.energy = 0;
                player.state |= PlayerState::SKILL;
                fighting.0 = player_id.0 + 1;
                player.animation.phase = 0;
                player.animation.count = 0;
                if player.character_id == 1 {
                    commands.insert_resource(SoulAbsorb);
                }
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
    fighting: ResMut<Fighting>,
    time: Res<Time>,
    config: Res<GameConfig>,
    player_collision: Res<PlayerCollision>,
    asset_server: Res<AssetServer>,
    mut gamestate: ResMut<GameState>,
    mut timer: ResMut<AnimationTimer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    character_textures: Res<CharacterTextures>,
    mut player_query: Query<
        (&mut Player, &PlayerID, &mut Sprite, &mut Transform, &mut Visibility),
        Without<BackGround>,
    >,
    mut ground_query: Query<
        &mut Transform,
        (
            With<BackGround>,
            Without<Player>
        ),
    >,
) {
    // skill animation
    if fighting.0 != 0 {
        return;
    }
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        for (mut player, player_id, mut sprite, mut transform, _) in player_query.iter_mut() {
            if player.stun_count > 3 {
                player.stun_count -= 1;
            }

            // when game phase is 6(gameover), player will perform the loser and winner pose
            if gamestate.phase == 6 && player.animation.count != 0 {
                transform.translation.y += (270.0 - config.window_size.y / 2.0 - transform.translation.y) / player.animation.count as f32;
                player.update_animation(&mut sprite);
                if player.animation.count == 0 {
                    player.animation.phase = 1;
                    commands.remove_resource::<Fighting>();
                    gamestate.phase = 7;
                    gamestate.count = 0;
                }
                continue;
            }

            if !player.state.check(PlayerState::RANGED_ATTACK) && player.fire_charge < FIRE_CHARGE_MAX {
                player.fire_charge += 1;
            }

            // player is stunning
            if player.state.check(PlayerState::STUN) {
                player.update_animation(&mut sprite);
                if !player.state.check(PlayerState::BEND_DOWN) {
                    // if the player is not bend down
                    // the player will slip a little
                    if player.pose.facing {
                        transform.translation.x -= 10.0;
                    } else {
                        transform.translation.x += 10.0;
                    }
                }
                if player.animation.count == 0 {
                    player.state = PlayerState::COOLDOWN;
                    player.set_animation(IDLE_POSE1, 0, 10);
                    player.animation.diff_y = (270.0 - config.window_size.y / 2.0 - transform.translation.y) / player.animation.count as f32;
                }
            }

            // player is idle
            if player.state.is_idle() | player.state.check(PlayerState::COOLDOWN) {
                player.velocity = Vec2::ZERO;
                if player.animation.phase == 0 {
                    player.update_animation_idle(&mut transform, &mut sprite);
                    if player.animation.count == 0 {
                        if player.state.check(PlayerState::COOLDOWN) {
                            player.state &= !PlayerState::COOLDOWN;
                        }
                        if player.state.check(PlayerState::ATTACK_DISABLED) {
                            player.state &= !PlayerState::ATTACK_DISABLED;
                        }
                        player.set_animation(IDLE_POSE2, 1, 15);
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(IDLE_POSE1, 2, 15);
                    }
                } else if player.animation.phase == 2 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(IDLE_POSE2, 1, 15);
                    }
                }
            }
            if player.state.check(PlayerState::JUMP_UP) {
                // player is jumping

                // prepare for jump
                if player.animation.phase != 0 {
                    player.velocity -= Vec2::new(0.0, GRAVITY_ACCEL * 2.0 / FPS);
                }

                if player.animation.phase == 0 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        if cfg!(not(target_arch = "wasm32")) {
                            player.velocity = Vec2::new(0.0, 12.0);
                        } else {
                            player.velocity = Vec2::new(0.0, 8.0);
                        }
                        player.set_animation(JUMP_UP_POSE2, 1, 48);
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        if player.state.check(PlayerState::KICKING) {
                            let mut jumping_kick_pose = JUMPING_KICK_POSE;
                            jumping_kick_pose.body = 0.0;
                            player.set_animation(jumping_kick_pose, 2, 5);
                        } else {
                            player.animation.phase = 2;
                            player.animation.count = 0;
                        }
                    }
                } else if player.animation.phase == 2 {
                    if player.state.check(PlayerState::KICKING) {
                        let mut jumping_kick_pose = JUMPING_KICK_POSE;
                        jumping_kick_pose.body = 0.0;
                        player.set_animation(jumping_kick_pose, 2, 5);
                    }
                    player.update_animation(&mut sprite);
                }
            } else if player.state.check(PlayerState::JUMP_FORWARD) {
                // player is jumping forward

                // prepare for jump
                if player.animation.phase != 0 {
                    player.velocity -= Vec2::new(0.0, GRAVITY_ACCEL * 2.0 / FPS);
                }

                if player.animation.phase == 0 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        let x_vel = if player.state.check(PlayerState::DIRECTION) {
                            CHARACTER_PROFILES[player.character_id as usize].agility * 2.0
                        } else {
                            -CHARACTER_PROFILES[player.character_id as usize].agility * 2.0
                        };
                        if cfg!(not(target_arch = "wasm32")) {
                            player.velocity = Vec2::new(x_vel, 12.0);
                            if player.state.check(PlayerState::KICKING) {
                                player.set_animation(JUMP_FORWARD_POSE2, 1, 10);
                            } else {
                                player.set_animation(JUMP_FORWARD_POSE2, 1, 15);
                            }
                        } else {
                            player.velocity = Vec2::new(x_vel, 8.0);
                            if player.state.check(PlayerState::KICKING) {
                                player.set_animation(JUMP_FORWARD_POSE2, 1, 6);
                            } else {
                                player.set_animation(JUMP_FORWARD_POSE2, 1, 10);
                            }
                        }
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        #[cfg(not(target_arch = "wasm32"))]
                        if player.state.check(PlayerState::KICKING) {
                            player.set_animation(JUMP_FORWARD_POSE3, 2, 10);
                        } else {
                            player.set_animation(JUMP_FORWARD_POSE3, 2, 15);
                        }
                        #[cfg(target_arch = "wasm32")]
                        if player.state.check(PlayerState::KICKING) {
                            player.set_animation(JUMP_FORWARD_POSE3, 2, 6);
                        } else {
                            player.set_animation(JUMP_FORWARD_POSE3, 2, 10);
                        }
                    }
                } else if player.animation.phase == 2 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        #[cfg(not(target_arch = "wasm32"))]
                        if player.state.check(PlayerState::KICKING) {
                            player.set_animation(JUMP_FORWARD_POSE4, 3, 20);
                        } else {
                            player.set_animation(JUMP_FORWARD_POSE4, 3, 30);
                        }
                        #[cfg(target_arch = "wasm32")]
                        if player.state.check(PlayerState::KICKING) {
                            player.set_animation(JUMP_FORWARD_POSE4, 3, 13);
                        } else {
                            player.set_animation(JUMP_FORWARD_POSE4, 3, 20);
                        }
                    }
                } else if player.animation.phase == 3 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        #[cfg(not(target_arch = "wasm32"))]
                        if player.state.check(PlayerState::KICKING) {
                            player.set_animation(JUMPING_KICK_POSE, 4, 10);
                        } else {
                            player.set_animation(JUMP_FORWARD_POSE5, 4, 15);
                        }
                        #[cfg(target_arch = "wasm32")]
                        if player.state.check(PlayerState::KICKING) {
                            player.set_animation(JUMPING_KICK_POSE, 4, 6);
                        } else {
                            player.set_animation(JUMP_FORWARD_POSE5, 4, 10);
                        }
                    }
                } else if player.animation.phase == 4 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.pose.body = 0.0;
                    }
                }
            } else if player.state.check(PlayerState::JUMP_BACKWARD) {
                // player is jumping backward

                // prepare for jump
                if player.animation.phase != 0 {
                    player.velocity -= Vec2::new(0.0, GRAVITY_ACCEL * 2.0 / FPS);
                }

                if player.animation.phase == 0 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        let x_vel = if player.state.check(PlayerState::DIRECTION) {
                            CHARACTER_PROFILES[player.character_id as usize].agility * 2.0
                        } else {
                            -CHARACTER_PROFILES[player.character_id as usize].agility * 2.0
                        };
                        if cfg!(not(target_arch = "wasm32")) {
                            player.velocity = Vec2::new(x_vel, 12.0);
                            player.set_animation(JUMP_BACKWARD_POSE2, 1, 15);
                        } else {
                            player.velocity = Vec2::new(x_vel, 8.0);
                            player.set_animation(JUMP_BACKWARD_POSE2, 1, 10);
                        }
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        #[cfg(not(target_arch = "wasm32"))]
                        player.set_animation(JUMP_BACKWARD_POSE3, 2, 15);
                        #[cfg(target_arch = "wasm32")]
                        player.set_animation(JUMP_BACKWARD_POSE3, 2, 10);
                    }
                } else if player.animation.phase == 2 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        #[cfg(not(target_arch = "wasm32"))]
                        player.set_animation(JUMP_BACKWARD_POSE4, 3, 30);
                        #[cfg(target_arch = "wasm32")]
                        player.set_animation(JUMP_BACKWARD_POSE4, 3, 20);
                    }
                } else if player.animation.phase == 3 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        #[cfg(not(target_arch = "wasm32"))]
                        player.set_animation(JUMP_BACKWARD_POSE5, 4, 15);
                        #[cfg(target_arch = "wasm32")]
                        player.set_animation(JUMP_BACKWARD_POSE5, 4, 10);
                    }
                } else if player.animation.phase == 4 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.pose.body = 0.0;
                    }
                }
            } else if player.state.check(PlayerState::BEND_DOWN) {
                // player is bending down
                if player.animation.phase == 0 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(BEND_DOWN_POSE2, 1, 10);
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        // Bend Down Pose lasts until the player stands up
                        player.animation.phase = 2;
                    }
                }
            } else if player.state.check(PlayerState::ROLL_FORWARD) {
                // player is rolling forward
                if player.animation.phase == 0 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_FORWARD_POSE2, 1, 5);
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_FORWARD_POSE3, 2, 5);
                    }
                } else if player.animation.phase == 2 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_FORWARD_POSE4, 3, 5);
                    }
                } else if player.animation.phase == 3 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_FORWARD_POSE5, 4, 5);
                    }
                } else if player.animation.phase == 4 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_FORWARD_POSE6, 5, 5);
                    }
                } else if player.animation.phase == 5 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_FORWARD_POSE7, 6, 5);
                    }
                } else if player.animation.phase == 6 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_IDLE;
                        player.pose.body = -20.0;
                        player.state = PlayerState::IDLE | PlayerState::COOLDOWN | PlayerState::ATTACK_DISABLED;
                        player.set_animation(IDLE_POSE1, 0, 10);
                    }
                }
            } else if player.state.check(PlayerState::ROLL_BACK) {
                // player is rolling back
                if player.animation.phase == 0 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.pose.body = -380.0;
                        player.set_animation(ROLL_BACK_POSE2, 1, 5);
                    }
                } else if player.animation.phase == 1 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_BACK_POSE3, 2, 5);
                    }
                } else if player.animation.phase == 2 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_BACK_POSE4, 3, 5);
                    }
                } else if player.animation.phase == 3 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_BACK_POSE5, 4, 5);
                    }
                } else if player.animation.phase == 4 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_BACK_POSE6, 5, 5);
                    }
                } else if player.animation.phase == 5 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        player.set_animation(ROLL_BACK_POSE7, 6, 5);
                    }
                } else if player.animation.phase == 6 {
                    player.update_animation(&mut sprite);
                    if player.animation.count == 0 {
                        sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_IDLE;
                        player.state = PlayerState::IDLE | PlayerState::COOLDOWN | PlayerState::ATTACK_DISABLED;
                        player.set_animation(IDLE_POSE1, 0, 10);
                    }
                }
            } else {
                if player.state.check(PlayerState::KICKING) {
                    if player.animation.phase == 0 {
                        player.update_animation(&mut sprite);
                        if player.animation.count == 0 {
                            player.state |= PlayerState::COOLDOWN | PlayerState::ATTACK_DISABLED;
                            player.set_animation(IDLE_POSE1, 0, 25);
                        }
                    } else if player.animation.phase == 1 {
                        player.update_animation(&mut sprite);
                        if player.animation.count == 0 {
                            sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                            player.animation_frame_max = FRAMES_IDLE;
                            player.state = PlayerState::IDLE;
                            player.set_animation(IDLE_POSE2, 1, 25);
                        }
                    }
                } else if player.state.check(PlayerState::RANGED_ATTACK) {
                    if player.animation.phase == 0 {
                        player.update_animation(&mut sprite);
                        if player.animation.count == 0 {
                            player.state = PlayerState::IDLE | PlayerState::COOLDOWN | PlayerState::ATTACK_DISABLED;
                            sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                            player.animation_frame_max = FRAMES_IDLE;
                            player.set_animation(IDLE_POSE1, 0, 30);
                            commands.spawn((
                                Sprite {
                                    image: asset_server.load(format!("{}fire_arrow_atlas.png", PATH_IMAGE_PREFIX)),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: texture_atlas_layouts.add(
                                            TextureAtlasLayout::from_grid(UVec2::new(216, 112), 1, 10, None, None)
                                        ),
                                        index: 0
                                    }),
                                    flip_x: if player.pose.facing {
                                        true
                                    } else {
                                        false
                                    },
                                    ..Default::default()
                                },
                                PlayerID(player_id.0),
                                FireAnimation {
                                    facing: player.pose.facing,
                                },
                                Transform::from_translation(Vec3::new(
                                    transform.translation.x,
                                    transform.translation.y,
                                    20.0,
                                )),
                            ));
                            
                        }
                    }
                } else if player.state.check(PlayerState::BACK_KICKING) {
                    if player.animation.phase == 0 {
                        player.update_animation(&mut sprite);
                        if player.animation.count == 0 {
                            player.set_animation(BACK_KICK_POSE2, 1, 40);
                        }
                    } else if player.animation.phase == 1 {
                        player.update_animation(&mut sprite);
                        if player.animation.count == 0 {
                            sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                            player.animation_frame_max = FRAMES_IDLE;
                            player.state = PlayerState::IDLE | PlayerState::COOLDOWN | PlayerState::ATTACK_DISABLED;
                            player.set_animation(IDLE_POSE1, 0, 30);
                        }
                    }
                } else if player.state.check(PlayerState::PUNCHING) {
                    if player.animation.phase == 0 {
                        player.update_animation(&mut sprite);
                        if player.animation.count == 0 {
                            sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                            player.animation_frame_max = FRAMES_IDLE;
                            player.state = PlayerState::IDLE | PlayerState::COOLDOWN | PlayerState::ATTACK_DISABLED;
                            player.set_animation(IDLE_POSE1, 0, 30);
                        }
                    }
                }
                if player.state.check(PlayerState::WALKING) {
                    if player.state.is_forward()
                        && player.velocity.x
                            < CHARACTER_PROFILES[player.character_id as usize].agility
                    {
                        player.velocity += Vec2::new(1.0, 0.0) * PIXELS_PER_METER / FPS;
                    } else if !player.state.is_forward()
                        && player.velocity.x
                            > -CHARACTER_PROFILES[player.character_id as usize].agility
                    {
                        player.velocity += Vec2::new(-1.0, 0.0) * PIXELS_PER_METER / FPS;
                    }
                    if player.velocity.x > CHARACTER_PROFILES[player.character_id as usize].agility
                    {
                        player.velocity.x =
                            CHARACTER_PROFILES[player.character_id as usize].agility;
                    } else if player.velocity.x
                        < -CHARACTER_PROFILES[player.character_id as usize].agility
                    {
                        player.velocity.x =
                            -CHARACTER_PROFILES[player.character_id as usize].agility;
                    }
                    if player.animation.phase == 0 {
                        player.update_animation(&mut sprite);
                        if player.animation.count == 0 {
                            player.set_animation(WALKING_POSE2, 1, 15);
                        }
                    } else if player.animation.phase == 1 {
                        player.update_animation(&mut sprite);
                        if player.animation.count == 0 {
                            player.set_animation(WALKING_POSE1, 0, 15);
                        }
                    }
                }
            }
            if player_collision.0 == 2 {
                // no collision, player moves freely
                transform.translation +=
                    Vec3::new(player.velocity.x, player.velocity.y, 0.0) * PIXELS_PER_METER / FPS;
            } else {
                // collision, player cannot move along x-axis
                transform.translation +=
                    Vec3::new(0.0, player.velocity.y, 0.0) * PIXELS_PER_METER / FPS;
            }
        }

        /*
        // move player and ground
         */
        let mut ground = ground_query.single_mut();

        // Check if players are at opposite ends of the screen
        // 0 means player isn't at edge, 1 means player is at left edge, 2 means player is at right edge
        let mut at_edges: [u8; 2] = [0; 2];
        for (_, player_id, _, transform, _) in player_query.iter() {
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
            if let Some((_, _, _, mut transform, _)) = player_query
                .iter_mut()
                .find(|(_, player_id, _, _, _)| player_id.0 == 0)
            {
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
            } else if let Some((_, _, _, mut transform, _)) = player_query
                .iter_mut()
                .find(|(_, player_id, _, _, _)| player_id.0 == 1)
            {
                transform.translation.x += diff;
            }
        } else if at_edges[0] == 0 && (at_edges[1] == 1 || at_edges[1] == 2) {
            let mut diff = 0.0;
            if let Some((_, _, _, mut transform, _)) = player_query
                .iter_mut()
                .find(|(_, player_id, _, _, _)| player_id.0 == 1)
            {
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
            } else if let Some((_, _, _, mut transform, _)) = player_query
                .iter_mut()
                .find(|(_, player_id, _, _, _)| player_id.0 == 0)
            {
                transform.translation.x += diff;
            }
        } else if at_edges[0] != 0 && at_edges[1] != 0 {
            if (at_edges[0] == 1 && at_edges[1] == 3) || (at_edges[0] == 3 && at_edges[1] == 1) {
                // If both players are at the same edge, move the camera to the edge
                let mut diff = 0.0;
                if let Some((_, _, _, mut transform, _)) =
                    player_query.iter_mut().find(|(_, player_id, _, _, _)| {
                        player_id.0 == if at_edges[0] == 1 { 0 } else { 1 }
                    })
                {
                    diff = -config.window_size.x / 2.0 + 100.0 - transform.translation.x;
                    transform.translation.x = -config.window_size.x / 2.0 + 100.0;
                }
                ground.translation.x += diff;
                if let Some((_, _, _, mut transform, _)) =
                    player_query.iter_mut().find(|(_, player_id, _, _, _)| {
                        player_id.0 == if at_edges[0] == 1 { 1 } else { 0 }
                    })
                {
                    transform.translation.x += diff;
                }
            } else if (at_edges[0] == 2 && at_edges[1] == 4)
                || (at_edges[0] == 4 && at_edges[1] == 2)
            {
                // If both players are at the same edge, move the camera to the edge
                let mut diff = 0.0;
                if let Some((_, _, _, mut transform, _)) =
                    player_query.iter_mut().find(|(_, player_id, _, _, _)| {
                        player_id.0 == if at_edges[0] == 2 { 0 } else { 1 }
                    })
                {
                    diff = config.window_size.x / 2.0 - 100.0 - transform.translation.x;
                    transform.translation.x = config.window_size.x / 2.0 - 100.0;
                }
                ground.translation.x += diff;
                if let Some((_, _, _, mut transform, _)) =
                    player_query.iter_mut().find(|(_, player_id, _, _, _)| {
                        player_id.0 == if at_edges[0] == 2 { 1 } else { 0 }
                    })
                {
                    transform.translation.x += diff;
                }
            } else {
                for (_, player_id, _, mut transform, _) in player_query.iter_mut() {
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

// This function handles the skill animation
fn skill_animation(
    mut commands: Commands,
    mut fighting: ResMut<Fighting>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut timer: ResMut<AnimationTimer>,
    mut player_query: Query<
        (&mut Player, &PlayerID, &mut Sprite, &mut Transform, &mut Visibility),
        (Without<Camera2d>, Without<SkillEntity>),
    >,
    energy_query: Query<(&mut EnergyBar, &mut Mesh2d, &PlayerID), Without<SkillEntity>>,
    mut skill_name_query: Query<
        (&SkillName, &mut Visibility),
        (Without<SkillEntity>, Without<Player>),
    >,
    mut thunder_query: Query<
        (&SkillEntity, &mut Visibility, &mut Transform),
        (Without<SkillName>, Without<Player>, Without<Mesh2d>),
    >,
    curtain_query: Query<(Entity, &SkillEntity, &Mesh2d), Without<EnergyBar>>,
    mut hammer_query: Query<
        (Entity, &SkillEntity, &Mesh2d, &mut Transform),
        (Without<SkillName>, Without<Player>),
    >,
    mut damage_display_query: Query<(&PlayerID, &mut Text, &mut TextColor, &mut DamageDisplay)>,
    mut camera_query: Query<
        &mut Transform,
        (With<Camera2d>, Without<SkillEntity>, Without<Player>, Without<Mesh2d>)
    >,
) {
    // normal animation
    if fighting.0 == 0 {
        return;
    }
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        // perform skill animation
        if fighting.0 != 0 {
            let mut opponent_position = Vec2::ZERO;
            if let Some((_, _, _, transform, _)) = player_query
                .iter_mut()
                .find(|(_, id, _, _, _)| id.0 != fighting.0 - 1)
            {
                opponent_position = Vec2::new(transform.translation.x, transform.translation.y);
            }
            let mut damage: u32 = 0;
            if let Some((mut player, player_id, mut sprite, mut transform, mut player_visibility)) =
                player_query
                    .iter_mut()
                    .find(|(_, id, _, _, _)| id.0 == fighting.0 - 1)
            {
                if player.animation.phase == 0 {
                    player.animation.count += 1;
                    // change curtain color to draken the screen
                    if let Some((_, _, mesh_handler)) = curtain_query.iter().find(|x| x.1.id == 1) {
                        let mesh = meshes.get_mut(mesh_handler.id()).unwrap();
                        if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                        {
                            for i in 0..4 {
                                colors[i][3] = player.animation.count as f32 / 60.0;
                            }
                        }
                    }
                    if player.animation.count == 20 {
                        // show skill name
                        for (skill_name, mut skill_name_visibility) in skill_name_query.iter_mut() {
                            if skill_name.0 == player.character_id as u8 {
                                *skill_name_visibility = Visibility::Visible;
                            }
                        }

                        player.animation.phase = 1;
                        player.animation.count = 0;
                    }
                } else if player.animation.phase == 1 {
                    player.animation.count += 1;
                    if player.animation.count == 60 {
                        for (skill_name, mut skill_name_visibility) in skill_name_query.iter_mut() {
                            if skill_name.0 == player.character_id as u8 {
                                *skill_name_visibility = Visibility::Hidden;
                            }
                        }
                        player.animation.phase = 2;
                        player.animation.count = 0;
                    }
                } else if player.animation.phase == 2 {
                    if player.character_id == 0 {
                        // character 0 skill
                        player.animation.count += 1;
                        // change curtain color to draken the screen
                        if let Some((_, _, mesh_handler)) =
                            curtain_query.iter().find(|x| x.1.id == 1)
                        {
                            let mesh = meshes.get_mut(mesh_handler.id()).unwrap();
                            if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                            {
                                for i in 0..4 {
                                    colors[i][3] = (player.animation.count + 20) as f32 / 60.0;
                                }
                            }
                        }
                        if player.animation.count == 35 {
                            player.animation.phase = 3;
                            player.animation.count = 0;
                        }
                    } else if player.character_id == 1 {
                        // character 1 skill
                        // decide soul color using random number
                        player.animation.count += 1;
                        let color_rand = rand();
                        let color = if color_rand >= 2.0 / 3.0 {
                            SOUL_COLOR[0]
                        } else if color_rand >= 1.0 / 3.0 {
                            SOUL_COLOR[1]
                        } else {
                            SOUL_COLOR[2]
                        };
                        // create a soul entity
                        commands.spawn((
                            Mesh2d(meshes.add(Circle::new(rand() * 10.0))),
                            MeshMaterial2d(materials.add(color)),
                            Transform::from_translation(Vec3::new(
                                opponent_position.x,
                                opponent_position.y + 50.0,
                                20.0,
                            )),
                            // soul entity's id is 2
                            SkillEntity { id: 2 },
                        ));
                        if player.animation.count == 100 {
                            player.animation.phase = 3;
                            player.animation.count = 0;
                        }
                    } else if player.character_id == 2 {
                        // character 2 skill
                        player.set_animation(HAMMER_POSE1, 3, 30);
                        commands.spawn((
                            Mesh2d(
                                meshes.add(
                                    Mesh::new(
                                        PrimitiveTopology::TriangleList,
                                        RenderAssetUsages::default(),
                                    )
                                    .with_inserted_attribute(
                                        Mesh::ATTRIBUTE_POSITION,
                                        if player.pose.facing {
                                            vec![
                                                [100.0, 100.0, 0.0],
                                                [100.0, 500.0, 0.0],
                                                [160.0, 500.0, 0.0],
                                                [160.0, 100.0, 0.0],
                                            ]
                                        } else {
                                            vec![
                                                [-100.0, 100.0, 0.0],
                                                [-100.0, 500.0, 0.0],
                                                [-160.0, 500.0, 0.0],
                                                [-160.0, 100.0, 0.0],
                                            ]
                                        }
                                    )
                                    .with_inserted_attribute(
                                        Mesh::ATTRIBUTE_COLOR,
                                        vec![
                                            [5.0, 0.0, 0.0, 0.0],
                                            [5.0, 0.0, 0.0, 0.0],
                                            [5.0, 0.0, 0.0, 0.0],
                                            [5.0, 0.0, 0.0, 0.0],
                                        ],
                                    )
                                    .with_inserted_indices(Indices::U32(vec![
                                        0, 1, 2,
                                        0, 2, 3,
                                    ])),
                                ),
                            ),
                            MeshMaterial2d(materials.add(ColorMaterial::default())),
                            SkillEntity { id: 3 },
                            Transform::from_translation(Vec3::new(
                                transform.translation.x,
                                transform.translation.y,
                                20.0,
                            )),
                        ))
                            .with_child((
                                Mesh2d(
                                    meshes.add(
                                        Mesh::new(
                                            PrimitiveTopology::TriangleList,
                                            RenderAssetUsages::default(),
                                        )
                                        .with_inserted_attribute(
                                            Mesh::ATTRIBUTE_POSITION,
                                            vec![
                                                [-100.0, -50.0, 0.0],
                                                [-100.0, 50.0, 0.0],
                                                [100.0, 50.0, 0.0],
                                                [100.0, -50.0, 0.0],
                                            ],
                                        )
                                        .with_inserted_attribute(
                                            Mesh::ATTRIBUTE_COLOR,
                                            vec![
                                                [0.0, 0.0, 0.0, 0.0],
                                                [0.0, 0.0, 0.0, 0.0],
                                                [0.0, 0.0, 0.0, 0.0],
                                                [0.0, 0.0, 0.0, 0.0],
                                            ],
                                        )
                                        .with_inserted_indices(Indices::U32(vec![
                                            0, 1, 2,
                                            0, 2, 3,
                                        ])),
                                    ),
                                ),
                                MeshMaterial2d(materials.add(ColorMaterial::default())),
                                SkillEntity { id: 4 },
                                if player.pose.facing {
                                    Transform::from_translation(Vec3::new(
                                        130.0,
                                        400.0,
                                        1.0,
                                    ))
                                } else {
                                    Transform::from_translation(Vec3::new(
                                        -130.0,
                                        400.0,
                                        1.0,
                                    ))
                                },
                            ));
                    }
                } else if player.animation.phase == 3 {
                    if player.character_id == 0 {
                        player.animation.count += 1;
                        if player.animation.count == 30 {
                            if let Some((_, mut thunder_visibility, mut thunder_transform)) =
                                thunder_query.iter_mut().find(|x| x.0.id == 0)
                            {
                                *thunder_visibility = Visibility::Visible;
                                thunder_transform.translation.x = transform.translation.x;
                            }
                            *player_visibility = Visibility::Hidden;
                            player.pose = THUNDER_PUNCH_POSE;
                            if player.pose.facing {
                                transform.translation.x = opponent_position.x - 100.0;
                                transform.translation.y = opponent_position.y + 50.0;
                            } else {
                                transform.translation.x = opponent_position.x + 100.0;
                                transform.translation.y = opponent_position.y + 50.0;
                            }
                            commands.spawn((
                                AudioPlayer::new(
                                    asset_server.load(format!("{}/thunder.ogg", PATH_SOUND_PREFIX)),
                                ),
                                SoundEffect,
                            ));
                            player.animation.phase = 4;
                            player.animation.count = 0;
                        }
                    } else if player.character_id == 2 {
                        player.update_animation(&mut sprite);
                        for (_, skill_entity, mesh_handler, _) in hammer_query.iter() {
                            if skill_entity.id == 3 || skill_entity.id == 4 {
                                if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                                    if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                                    {
                                        for i in 0..4 {
                                            colors[i][3] = (30 - player.animation.count) as f32 / 30.0;
                                        }
                                    }
                                }
                            }
                        }
                        if player.animation.count == 0 {
                            player.animation.phase = 4;
                            player.animation.count = 60;
                        }
                    }
                } else if player.animation.phase == 4 {
                    if player.character_id == 0 {
                        player.animation.count += 1;
                        // change curtain color to draken the screen
                        if let Some((_, _, mesh_handler)) =
                            curtain_query.iter().find(|x| x.1.id == 1)
                        {
                            let mesh = meshes.get_mut(mesh_handler.id()).unwrap();
                            if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                            {
                                for i in 0..4 {
                                    if player.animation.count <= 5 {
                                        colors[i][0] = player.animation.count as f32 / 5.0;
                                        colors[i][1] = player.animation.count as f32 / 5.0;
                                        colors[i][2] = player.animation.count as f32 / 5.0;
                                    } else {
                                        colors[i][3] = 2.0 - player.animation.count as f32 / 5.0;
                                    }
                                }
                            }
                            // earthquake effect
                            let mut transform = camera_query.single_mut();
                            transform.translation.x = rand() * 100.0;
                            transform.translation.y = rand() * 100.0;
                        }
                        if player.animation.count == 10 {
                            damage = 150;
                            *player_visibility = Visibility::Visible;
                            if let Some((_, _, mesh_handler)) =
                                curtain_query.iter().find(|x| x.1.id == 1)
                            {
                                let mesh = meshes.get_mut(mesh_handler.id()).unwrap();
                                if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                    mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                                {
                                    for i in 0..4 {
                                        colors[i][0] = 0.0;
                                        colors[i][1] = 0.0;
                                        colors[i][2] = 0.0;
                                    }
                                }
                            }
                            if let Some((_, mut thunder_visibility, _)) =
                                thunder_query.iter_mut().find(|x| x.0.id == 0)
                            {
                                *thunder_visibility = Visibility::Hidden;
                            }
                            player.set_animation(IDLE_POSE1, 5, 30);
                            player.velocity = Vec2::ZERO;

                            let mut transform = camera_query.single_mut();
                            transform.translation.x = 0.0;
                            transform.translation.y = 0.0;
                        }
                    } else if player.character_id == 1 {
                        player.animation.count += 1;
                        if let Some((_, _, mesh_handler)) =
                            curtain_query.iter().find(|x| x.1.id == 1)
                        {
                            let mesh = meshes.get_mut(mesh_handler.id()).unwrap();
                            if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                            {
                                for i in 0..4 {
                                    colors[i][3] = (20 - player.animation.count) as f32 / 60.0;
                                }
                            }
                        }
                        if player.animation.count == 20 {
                            player.animation.phase = 7;
                        }
                    } else if player.character_id == 2 {
                        player.animation.count -= 1;
                        if player.animation.count == 0 {
                            player.set_animation(HAMMER_POSE2, 5, 5);
                        }
                    }
                } else if player.animation.phase == 5 {
                    if player.character_id == 0 {
                        player.update_animation(&mut sprite);
                        if transform.translation.y > 270.0 - config.window_size.y / 2.0 {
                            player.velocity.y -= GRAVITY_ACCEL * 4.0 / FPS;
                            transform.translation.y += player.velocity.y;
                            if transform.translation.y < 270.0 - config.window_size.y / 2.0 {
                                transform.translation.y = 270.0 - config.window_size.y / 2.0;
                            }
                        }
                        if player.animation.count == 0 {
                            if transform.translation.y == 270.0 - config.window_size.y / 2.0 {
                                player.animation.phase = 7;
                            } else {
                                // player position have to be reset
                                player.animation.phase = 6;
                            }
                        }
                    } else if player.character_id == 2 {
                        player.update_animation(&mut sprite);
                        for (_, skill_entity, _, mut hammer_transform) in hammer_query.iter_mut() {
                            if skill_entity.id == 3 {
                                if player.pose.facing {
                                    let rad = (18.0 * player.animation.count as f32 - 90.0).to_radians();
                                    hammer_transform.rotation = Quat::from_rotation_z(rad);
                                } else {
                                    let rad = (90.0 - 18.0 * player.animation.count as f32).to_radians();
                                    hammer_transform.rotation = Quat::from_rotation_z(rad);
                                }
                            }
                        }
                        if player.animation.count == 0 {
                            // if the opponent is at the ground, the hammer will hit the opponent
                            if opponent_position.y < 300.0 - config.window_size.y / 2.0 {
                                damage = 300;
                            }
                            player.animation.phase = 6;
                            player.animation.count = 0;
                        }
                    }
                } else if player.animation.phase == 6 {
                    if player.character_id == 0 {
                        if transform.translation.y > 270.0 - config.window_size.y / 2.0 {
                            player.velocity.y -= GRAVITY_ACCEL * 4.0 / FPS;
                            transform.translation.y += player.velocity.y;
                            if transform.translation.y < 270.0 - config.window_size.y / 2.0 {
                                transform.translation.y = 270.0 - config.window_size.y / 2.0;
                                player.animation.phase = 7;
                            }
                        }
                    } else if player.character_id == 2 {
                        player.animation.count += 1;
                        if let Some((_, _, mesh_handler)) =
                            curtain_query.iter().find(|x| x.1.id == 1)
                        {
                            let mesh = meshes.get_mut(mesh_handler.id()).unwrap();
                            if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                            {
                                for i in 0..4 {
                                    colors[i][3] = (20 - player.animation.count) as f32 / 60.0;
                                }
                            }
                            for (_, skill_entity, mesh_handler, _) in hammer_query.iter() {
                                if skill_entity.id == 3 || skill_entity.id == 4 {
                                    if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                                        if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                                        {
                                            for i in 0..4 {
                                                colors[i][3] = (20 - player.animation.count) as f32 / 20.0;
                                            }
                                        }
                                    }
                                }
                            }
                            // earthquake effect
                            let mut transform = camera_query.single_mut();
                            transform.translation.x = rand() * 50.0;
                            transform.translation.y = rand() * 50.0;
                        }
                        if player.animation.count == 20 {
                            if let Some((entity, _, _)) = curtain_query.iter().find(|x| x.1.id == 3) {
                                commands.entity(entity).despawn_recursive();
                            }
                            player.animation.phase = 7;
                            player.animation.count = 0;

                            let mut transform = camera_query.single_mut();
                            transform.translation.x = 0.0;
                            transform.translation.y = 0.0;
                        }
                    }
                } else {
                    // finish skill
                    player.animation.phase = 0;
                    player.animation.count = 0;
                    player.state &= !PlayerState::SKILL;
                    fighting.0 = 0;
                    if let Some((_, mesh_handler, _)) =
                        energy_query.iter().find(|x| x.2 == player_id)
                    {
                        if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                            if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                            {
                                for i in 2..4 {
                                    colors[i][0] = 0.0;
                                    colors[i][2] = 10.0;
                                }
                            }
                        }
                    }
                }
                if damage != 0 {
                    if let Some((mut player, player_id, _, _, _)) = player_query
                        .iter_mut()
                        .find(|(_, id, _, _, _)| id.0 != fighting.0 - 1)
                    {
                        for (player_id_text, mut text, mut color, mut damage_display) in
                            damage_display_query.iter_mut()
                        {
                            if player_id.0 == player_id_text.0 {
                                text.0 = format!("{}", damage);
                                if damage > 100 {
                                    color.0 = Color::srgba(5.0, 0.0, 0.0, 1.0);   
                                    damage_display.is_red = true;              
                                } else {
                                    color.0 = Color::srgba(0.0, 0.0, 5.0, 1.0);
                                    damage_display.is_red = false;
                                }
                                damage_display.alpha = 1.0;
                            }
                        }
                        player.health = player.health.saturating_sub(damage);
                    }
                }
            }
        }
    }
}

fn update_soul_absorb_animation(
    mut commands: Commands,
    mut soul_query: Query<(Entity, &SkillEntity, &mut Transform), Without<Player>>,
    mut player_query: Query<(&mut Player, &Transform), Without<SkillEntity>>,
) {
    let mut destination = Vec2::ZERO;
    let mut vibe = false;
    for (mut player, transform) in player_query.iter_mut() {
        if player.character_id == 1 {
            if player.animation.phase == 3 && soul_query.iter().count() == 2 {
                player.animation.phase = 4;
                commands.remove_resource::<SoulAbsorb>();
            }
            destination.x = transform.translation.x;
            destination.y = transform.translation.y + 50.0;
            if player.animation.phase == 2 {
                vibe = true;
            }
        }
    }
    if vibe {
        if let Some((mut player, _)) = player_query.iter_mut().find(|x| x.0.character_id != 1) {
            // opponent shakes
            player.pose.offset[0] = rand() * 10.0;
            player.pose.offset[1] = rand() * 10.0;
        }
    }
    for (entity, skill_entity, mut transform) in soul_query.iter_mut() {
        if skill_entity.id != 2 {
            continue;
        }
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        // if distance between soul and character is lower than threshold, despawn soul and player gains HP
        if destination.distance(pos) < 10.0 {
            commands.entity(entity).despawn();
            for (mut player, _) in player_query.iter_mut() {
                if player.character_id == 1 {
                    player.health += 1;
                } else {
                    player.health -= 1;
                }
            }
        }
        let diff = {
            let x = destination.x - transform.translation.x;
            let y = destination.y - transform.translation.y;
            let norm = (x * x + y * y).sqrt();
            Vec2::new(x / norm, y / norm)
        };
        transform.translation.x += diff.x * 10.0 + rand() * 4.0 - 2.0;
        transform.translation.y += diff.y * 10.0 + rand() * 4.0 - 2.0;
    }
}

// check if the player is grounding
#[cfg(not(target_arch = "wasm32"))]
fn check_ground(config: Res<GameConfig>, character_textures: Res<CharacterTextures>, mut player_query: Query<(&mut Player, &mut Sprite, &mut Transform)>) {
    for (mut player, mut sprite, mut transform) in player_query.iter_mut() {
        // phase 0 is the preliminary motion
        if player.animation.phase == 0 {
            continue;
        }
        // change offset based on the type of jump
        if player
            .state
            .check(PlayerState::JUMP_BACKWARD | PlayerState::JUMP_FORWARD)
            && transform.translation.y + 50.0 < 270.0 - config.window_size.y / 2.0
            && player.animation.phase == 4
        {
            player.state &= !(PlayerState::JUMP_UP
                | PlayerState::JUMP_BACKWARD
                | PlayerState::JUMP_FORWARD
                | PlayerState::KICKING
                | PlayerState::WALKING);
            sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
            player.state |= PlayerState::COOLDOWN;
            player.set_animation(IDLE_POSE1, 0, 10);
            player.animation.diff_y = 50.0 / player.animation.count as f32;
            transform.translation.y = 220.0 - config.window_size.y / 2.0;
            player.velocity = Vec2::ZERO;
        } else if player.state.check(PlayerState::JUMP_UP)
            && transform.translation.y + 70.0 < 270.0 - config.window_size.y / 2.0
            && player.animation.phase == 2
        {
            player.state &= !(PlayerState::JUMP_UP
                | PlayerState::JUMP_BACKWARD
                | PlayerState::JUMP_FORWARD
                | PlayerState::KICKING
                | PlayerState::WALKING);
            sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
            player.state |= PlayerState::COOLDOWN;
            player.set_animation(IDLE_POSE1, 0, 10);
            player.animation.diff_y = 70.0 / player.animation.count as f32;
            transform.translation.y = 200.0 - config.window_size.y / 2.0;
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
        // phase 0 is the preliminary motion
        if player.animation.phase == 0 {
            continue;
        }
        // change offset based on the type of jump
        if player
            .state
            .check(PlayerState::JUMP_BACKWARD | PlayerState::JUMP_FORWARD)
            && transform.translation.y + 25.0 < 135.0 - config.window_size.y / 2.0
            && player.animation.phase == 4
        {
            player.state &=
                !(PlayerState::JUMP_UP | PlayerState::JUMP_BACKWARD | PlayerState::JUMP_FORWARD);
            player.set_animation(IDLE_POSE1, 0, 10);
            player.animation.diff_y = 25.0 / player.animation.count as f32;
            transform.translation.y = 110.0 - config.window_size.y / 2.0;
            player.velocity = Vec2::ZERO;
        } else if player.state.check(PlayerState::JUMP_UP)
            && transform.translation.y + 35.0 < 135.0 - config.window_size.y / 2.0
            && player.animation.phase == 2
        {
            player.state &=
                !(PlayerState::JUMP_UP | PlayerState::JUMP_BACKWARD | PlayerState::JUMP_FORWARD);
            player.set_animation(IDLE_POSE1, 0, 10);
            player.animation.diff_y = 35.0 / player.animation.count as f32;
            transform.translation.y = 100.0 - config.window_size.y / 2.0;
            player.velocity = Vec2::ZERO;
        }
    }
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
fn rotate_parts(transform: &mut Transform, x_offset: f32, y_offset: f32, degree: f32, length: f32) {
    let rad = degree.to_radians();
    transform.rotation = Quat::from_rotation_z(rad);
    transform.translation.x = x_offset + length * rad.sin();
    transform.translation.y = y_offset + length * (1.0 - rad.cos());
}

/// Rotates and positions the neck based on given parameters.
///
/// # Arguments
/// * `transform` - The Transform component to modify
/// * `degree` - Rotation angle in degrees (positive is counterclockwise)
///
/// This function:
/// 1. Converts degree to radians and sets rotation
/// 2. Calculates X offset using sin of angle * neck length
/// 3. Calculates Y position using cosine and adds vertical offset
///
/// # Note
/// The neck length is divided by 2.0 to position the neck correctly.
/// The head offset is subtracted from the Y position to align the head correctly.
fn rotate_neck(transform: &mut Transform, degree: f32) {
    let rad = degree.to_radians();
    transform.rotation = Quat::from_rotation_z(-rad);
    transform.translation.x = NECK_LENGTH / 2.0 * rad.sin();
    transform.translation.y = HEAD_OFFSET - NECK_LENGTH / 2.0 * (1.0 - rad.cos());
}

/// Updates the pose of the player character based on their current state.
fn update_pose(
    mut player_query: Query<
        (&mut Player, &PlayerID),
        Without<BodyParts>,
    >,
    mut parts_query: Query<
        (&BodyParts, &PlayerID, &mut Transform),
        Without<Player>,
    >,
) {
    for (player, player_id) in player_query.iter_mut() {
        let flip = if player.pose.facing { 1.0 } else { -1.0 };
        for (parts, parts_id, mut transform) in parts_query.iter_mut() {
            if player_id.0 == parts_id.0 {
                match parts.flags {
                    // Head(Neck)
                    0b10000 => rotate_neck(&mut transform, flip * player.pose.head),
                    // Body
                    0b01000 => {
                        rotate_parts(&mut transform, 0.0, BODY_OFFSET, flip * player.pose.body, BODY_LENGTH);
                        if cfg!(not(target_arch = "wasm32")) {
                            transform.translation.x += player.pose.offset[0] - player.pose.old_offset[0];
                            transform.translation.y += player.pose.offset[1] - player.pose.old_offset[1];
                        } else {
                            transform.translation.x += (player.pose.offset[0] - player.pose.old_offset[0]) / 2.0;
                            transform.translation.y += (player.pose.offset[1] - player.pose.old_offset[1]) / 2.0;
                        }
                    }
                    // Right Upper Arm
                    0b00111 => rotate_parts(
                        &mut transform,
                        -flip * BODY_THICKNESS,
                        UPPER_ARM_OFFSET,
                        flip * player.pose.right_upper_arm,
                        UPPER_ARM_LENGTH
                    ),
                    // Right Lower Arm
                    0b00110 => rotate_parts(
                        &mut transform,
                        0.0,
                        LOWER_ARM_OFFSET,
                        flip * player.pose.right_lower_arm,
                        LIMB_LENGTH
                    ),
                    // Right Upper Leg
                    0b00011 => rotate_parts(
                        &mut transform,
                        -flip * BODY_THICKNESS,
                        UPPER_LEG_OFFSET,
                        flip * player.pose.right_upper_leg,
                        UPPER_LEG_LENGTH
                    ),
                    // Right Lower Leg
                    0b00010 => rotate_parts(
                        &mut transform,
                        0.0,
                        LOWER_LEG_OFFSET,
                        flip * player.pose.right_lower_leg,
                        LIMB_LENGTH
                    ),
                    // Left Upper Arm
                    0b00101 => rotate_parts(
                        &mut transform,
                        2.0 * flip * BODY_THICKNESS,
                        UPPER_ARM_OFFSET,
                        flip * player.pose.left_upper_arm,
                        UPPER_ARM_LENGTH
                    ),
                    // Left Lower Arm
                    0b00100 => rotate_parts(
                        &mut transform,
                        0.0,
                        LOWER_ARM_OFFSET,
                        flip * player.pose.left_lower_arm,
                        LIMB_LENGTH
                    ),
                    // Left Upper Leg
                    0b00001 => rotate_parts(
                        &mut transform,
                        flip * BODY_THICKNESS,
                        UPPER_LEG_OFFSET,
                        flip * player.pose.left_upper_leg,
                        UPPER_LEG_LENGTH
                    ),
                    // Left Lower Leg
                    0b00000 => rotate_parts(
                        &mut transform,
                        0.0,
                        LOWER_LEG_OFFSET,
                        flip * player.pose.left_lower_leg,
                        LIMB_LENGTH
                    ),
                    _ => {}
                }
            }
        }
    }
}

// TODO: sometimes attacker detection is going wrong
// This is because of the way to determine the attacker
// Now, attacker is determined based on their animation phase and count and PlayerState
/// Checks for collisions between players and updates their states accordingly.
fn check_attack(
    mut player_collision: ResMut<PlayerCollision>,
    mut collision_events: EventReader<CollisionEvent>,
    parts_query: Query<(&BodyParts, &PlayerID)>,
    mut player_query: Query<(&mut Player, &PlayerID)>,
    mut damage_display_query: Query<(&PlayerID, &mut Text, &mut TextColor, &mut DamageDisplay)>,
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
                if id1 == id2 {
                    continue;
                }

                let mut attacker_id: PlayerID = PlayerID(2);
                let mut opponent_id: PlayerID = PlayerID(2);
                let mut attacker_parts: &BodyParts = &BodyParts::NULL;
                let mut opponent_parts: &BodyParts = &BodyParts::NULL;
                let mut attacker_phase: u8 = 0;
                let mut attacker_count: u8 = 0;
                for (mut player, player_id) in player_query.iter_mut() {
                    if player.state.check(
                        PlayerState::KICKING
                            | PlayerState::BACK_KICKING
                            | PlayerState::PUNCHING,
                    ) && !player.state.check(PlayerState::ATTACK_DISABLED) {
                        // Check if the attacker is already set
                        if id1 == player_id {
                            attacker_parts = parts1;
                            opponent_parts = parts2;
                        } else if id2 == player_id {
                            attacker_parts = parts2;
                            opponent_parts = parts1;
                        }

                        // Check if the attacker is in a valid state
                        if player.state.check(
                            PlayerState::KICKING
                                | PlayerState::BACK_KICKING
                        ) && attacker_parts.is_arm()
                        {
                            continue;
                        } else if player.state.check(PlayerState::PUNCHING)
                            && !attacker_parts.is_arm()
                        {
                            continue;
                        }

                        // Check if the attacker is already set
                        if attacker_id != PlayerID(2)
                        && attacker_phase > player.animation.phase
                        || (attacker_phase == player.animation.phase && attacker_count > player.animation.count)
                        {
                            continue;
                        }
                        attacker_id = *player_id;
                        attacker_phase = player.animation.phase;
                        attacker_count = player.animation.count;
                        player.state |= PlayerState::ATTACK_DISABLED;
                        opponent_id = if PlayerID(0) == attacker_id {
                            PlayerID(1)
                        } else {
                            PlayerID(0)
                        };
                    }
                }
                if attacker_id == PlayerID(2) {
                    if player_collision.0 != 2 {
                        continue;
                    }
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
                    } else if parts2.is_body() && !parts1.is_body() {
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

                let damage = calculate_damage(
                    player_info[attacker_id.0 as usize],
                    player_info[opponent_id.0 as usize],
                    opponent_parts,
                );
                if let Some((mut player, _)) = player_query
                    .iter_mut()
                    .find(|(_, id)| id.0 == opponent_id.0)
                {
                    for (player_id, mut text, mut color, mut damage_display) in
                        damage_display_query.iter_mut()
                    {
                        if player_id.0 == opponent_id.0 {
                            text.0 = format!("{}", damage);
                            if damage > 100 {
                                color.0 = Color::srgba(5.0, 0.0, 0.0, 1.0);   
                                damage_display.is_red = true;              
                            } else {
                                color.0 = Color::srgba(0.0, 0.0, 5.0, 1.0);
                                damage_display.is_red = false;
                            }
                            damage_display.alpha = 1.0;
                        }
                    }
                    player.health = player.health.saturating_sub(damage);
                    if !player.state.check(PlayerState::BEND_DOWN) && player.stun_count <= 3 {
                        player.stun_count -= 1;
                        if player.stun_count == 0 {
                            // after 3 hits, player will be invicible for 240 frames(4 seconds)
                            player.stun_count = 243;
                        }
                        if player.state.is_idle() {
                            player.state = PlayerState::STUN | PlayerState::BEND_DOWN;
                            player.set_animation(STUN_POSE, 0, 5);
                        } else {
                            player.state = PlayerState::STUN;
                            player.set_animation(STUN_POSE, 0, 5);
                        }
                    }
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
                if id1 == id2 {
                    continue;
                }

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
    if let Some((player, _, mut transform)) = player_query
        .iter_mut()
        .find(|(_, id, _)| id.0 == player_collision.0)
    {
        transform.translation.x += if player.pose.facing { -1.0 } else { 1.0 };
    }
}

// coefficiency for each attack
const SKILL_COEFFICIENT: [f32; 3] = [
    1.0, // punch
    1.2, // kick
    1.5, // back kick
];
// coefficiency for each body part
const PARTS_COEFFICIENT: [f32; 4] = [
    1.5, // head
    1.0, // body
    0.9, // arm
    0.8, // leg
];
const DEFENCE_COEFICIENCY: f32 = 20.0;
const DEFENCE_OFFSET: f32 = 50.0;

fn calculate_damage(
    attacker_info: (isize, PlayerState),
    opponent_info: (isize, PlayerState),
    opponent_parts: &BodyParts,
) -> u32 {
    let attacker_profile = &CHARACTER_PROFILES[attacker_info.0 as usize];
    let opponent_profile = &CHARACTER_PROFILES[opponent_info.0 as usize];
    let mut damage = attacker_profile.power;

    // Apply damage multipliers based on player states
    if attacker_info.1.check(PlayerState::PUNCHING) {
        damage *= SKILL_COEFFICIENT[0];
    } else if attacker_info.1.check(PlayerState::KICKING) {
        damage *= SKILL_COEFFICIENT[1];
    } else if attacker_info.1.check(PlayerState::BACK_KICKING) {
        damage *= SKILL_COEFFICIENT[2];
    }

    // If attacker is performes a jumping kick, increase the damage
    if attacker_info
        .1
        .check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD)
    {
        damage *= 1.5;
    }

    // Apply damage multipliers based on opponent body parts
    if opponent_parts.is_head() {
        damage *= PARTS_COEFFICIENT[0];
    } else if opponent_parts.is_body() {
        damage *= PARTS_COEFFICIENT[1];
    } else if opponent_parts.is_arm() {
        damage *= PARTS_COEFFICIENT[2];
    } else {
        damage *= PARTS_COEFFICIENT[3];
    }

    let mut defence_bonus = 0.0;
    if opponent_info.1.check(PlayerState::BEND_DOWN) {
        defence_bonus += 50.0;
    }

    // Apply damage reduction based on opponent defense
    return (damage * DEFENCE_COEFICIENCY
        / (opponent_profile.defense + defence_bonus + DEFENCE_OFFSET))
        .floor() as u32;
}

fn update_fire_animation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<GameConfig>,
    mut fire_query: Query<(Entity, &PlayerID, &mut FireAnimation, &mut Transform, &mut Sprite), (Without<Player>, Without<DamageDisplay>)>,
    mut player_query: Query<(&mut Player, &PlayerID, &Transform), (Without<FireAnimation>, Without<DamageDisplay>)>,
    mut damage_display_query: Query<(&PlayerID, &mut Text, &mut TextColor, &mut DamageDisplay), (Without<Player>, Without<FireAnimation>)>,
    mut fire_charge_query: Query<(&mut FireBar, &mut Mesh2d, &PlayerID)>,
) {
    for (entity, fire_player_id, fire_animation, mut fire_transform, mut sprite) in fire_query.iter_mut() {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index += 1;
            if atlas.index == 10 {
                atlas.index = 0;
            }
        }
        if fire_animation.facing {
            fire_transform.translation.x += 20.0;
        } else {
            fire_transform.translation.x -= 20.0;
        }
        if fire_transform.translation.x < -config.window_size.x / 2.0
            || fire_transform.translation.x > config.window_size.x / 2.0
        {
            commands.entity(entity).despawn();
            if let Some((_, player_id, _)) = player_query
                .iter_mut()
                .find(|(_, id, _)| id.0 == fire_player_id.0)
            {
                for (_, mesh_handler, fire_id) in fire_charge_query.iter_mut() {
                    if player_id == fire_id {
                        if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                            if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                            {
                                for i in 0..4 {
                                    colors[i][0] = 1.0;
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut hit = false;
        for (mut player, player_id, transform) in player_query.iter_mut() {
            if player_id != fire_player_id {
                if transform.translation.x > fire_transform.translation.x - 150.0
                && transform.translation.x < fire_transform.translation.x + 150.0
                && transform.translation.y > fire_transform.translation.y - 150.0
                && transform.translation.y < fire_transform.translation.y + 150.0 {
                    let mut damage = 50;
                    if player.state.check(PlayerState::BEND_DOWN) {
                        damage = 40;
                    }
                    player.health = player.health.saturating_sub(damage);
                    commands.entity(entity).despawn();
                    hit = true;

                    if !player.state.check(PlayerState::BEND_DOWN) {
                        if player.state.is_idle() {
                            player.state = PlayerState::STUN | PlayerState::BEND_DOWN;
                            player.set_animation(STUN_POSE, 0, 5);
                        } else {
                            player.state = PlayerState::STUN;
                            player.set_animation(STUN_POSE, 0, 5);
                        }
                    }

                    for (damage_player_id, mut damage_text, mut damage_color, mut damage_display) in
                        damage_display_query.iter_mut()
                    {
                        if damage_player_id.0 == player_id.0 {
                            damage_text.0 = format!("{}", damage);
                            damage_color.0 = Color::srgba(0.0, 0.0, 5.0, 1.0);
                            damage_display.is_red = false;
                            damage_display.alpha = 1.0;
                        }
                    }
                }
            }
        }
        if hit {
            if let Some((_, player_id, _)) = player_query
                .iter_mut()
                .find(|(_, id, _)| id.0 == fire_player_id.0)
            {
                for (_, mesh_handler, fire_id) in fire_charge_query.iter_mut() {
                    if player_id == fire_id {
                        if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                            if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                            {
                                for i in 0..4 {
                                    colors[i][0] = 1.0;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_damage_display(
    mut damage_display_query: Query<(&mut TextColor, &mut DamageDisplay)>,
) {
    for (mut color, mut damage_display) in damage_display_query.iter_mut() {
        if damage_display.alpha != 0.0 {
            damage_display.alpha -= 0.05;
            if damage_display.alpha < 0.0 {
                damage_display.alpha = 0.0;
            }
            if damage_display.is_red {
                color.0 = Color::srgba(5.0, 0.0, 0.0, damage_display.alpha);
            } else {
                color.0 = Color::srgba(0.0, 0.0, 5.0, damage_display.alpha);
            }
        }
    }
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
                let target_ratio = player.health as f32 / profile.health as f32;
                if health_bar.0 == target_ratio {
                    continue;
                };
                health_bar.0 -= 0.005;
                if health_bar.0 < target_ratio {
                    health_bar.0 = target_ratio;
                }
                if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                    if let Some(VertexAttributeValues::Float32x3(ref mut positions)) =
                        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
                    {
                        positions[3][0] = health_bar.1 * health_bar.0;
                        if cfg!(target_arch = "wasm32") {
                            positions[2][0] = health_bar.1 * health_bar.0
                                + if player_id.0 == 0 { 25.0 } else { -25.0 };
                        } else {
                            positions[2][0] = health_bar.1 * health_bar.0
                                + if player_id.0 == 0 { 50.0 } else { -50.0 };
                        }
                    }
                }
            }
        }
    }
}

/// Updates the energy bar of the player character based on their current energy.
fn update_energy_bar(
    mut meshes: ResMut<Assets<Mesh>>,
    mut player_query: Query<(&mut Player, &PlayerID)>,
    mut energy_query: Query<(&mut EnergyBar, &mut Mesh2d, &PlayerID)>,
) {
    for (mut player, player_id) in player_query.iter_mut() {
        for (mut energy_bar, mesh_handler, energy_id) in energy_query.iter_mut() {
            if player_id == energy_id {
                if player.energy > ENERGY_MAX {
                    player.energy = ENERGY_MAX;
                }
                let target_ratio = player.energy as f32 / ENERGY_MAX as f32;
                if energy_bar.0 == target_ratio {
                    continue;
                };
                energy_bar.0 += 0.002;
                if energy_bar.0 > target_ratio {
                    energy_bar.0 = target_ratio;
                }
                if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                    if let Some(VertexAttributeValues::Float32x3(ref mut positions)) =
                        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
                    {
                        positions[3][0] = energy_bar.1 * energy_bar.0;
                        if cfg!(target_arch = "wasm32") {
                            positions[2][0] = energy_bar.1 * energy_bar.0
                                + if player_id.0 == 0 { 25.0 } else { -25.0 };
                        } else {
                            positions[2][0] = energy_bar.1 * energy_bar.0
                                + if player_id.0 == 0 { 50.0 } else { -50.0 };
                        }
                    }
                    if energy_bar.0 == 1.0 {
                        if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                        {
                            for i in 2..4 {
                                colors[i][0] = 10.0;
                                colors[i][2] = 0.0;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Updates the fire_charge bar of the player character based on their current fire_charge.
fn update_fire_bar(
    mut meshes: ResMut<Assets<Mesh>>,
    mut player_query: Query<(&mut Player, &PlayerID)>,
    mut fire_charge_query: Query<(&mut FireBar, &mut Mesh2d, &PlayerID)>
) {
    for (player, player_id) in player_query.iter_mut() {
        for (mut fire_bar, mesh_handler, fire_id) in fire_charge_query.iter_mut() {
            if player_id == fire_id {
                fire_bar.0 = player.fire_charge as f32 / FIRE_CHARGE_MAX as f32;
                if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                    if let Some(VertexAttributeValues::Float32x3(ref mut positions)) =
                        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
                    {
                        positions[3][0] = fire_bar.1 * fire_bar.0;
                        if cfg!(target_arch = "wasm32") {
                            positions[2][0] = fire_bar.1 * fire_bar.0
                                + if player_id.0 == 0 { 25.0 } else { -25.0 };
                        } else {
                            positions[2][0] = fire_bar.1 * fire_bar.0
                                + if player_id.0 == 0 { 50.0 } else { -50.0 };
                        }
                    }
                    if fire_bar.0 == 1.0 {
                        if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                        {
                            for i in 0..4 {
                                colors[i][0] = 20.0;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_facing(mut player_query: Query<(&mut Player, &PlayerID, &mut Sprite, &Transform)>) {
    let mut positions = [0.0; 2];
    for (_, player_id, _, transform) in player_query.iter_mut() {
        positions[player_id.0 as usize] = transform.translation.x;
    }
    for (mut player, player_id, mut sprite, _) in player_query.iter_mut() {
        if !player
            .state
            .check(!(PlayerState::COOLDOWN | PlayerState::DIRECTION | PlayerState::WALKING))
        {
            if player_id.0 == 0 {
                if positions[0] < positions[1] {
                    player.pose.facing = true;
                    sprite.flip_x = false;
                } else {
                    println!("flip texture");
                    player.pose.facing = false;
                    sprite.flip_x = true;
                }
            } else {
                if positions[1] < positions[0] {
                    player.pose.facing = true;
                    sprite.flip_x = false;
                } else {
                    player.pose.facing = false;
                    sprite.flip_x = true;
                }
            }
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationTimer {
            timer: Timer::from_seconds(1.0 / FPS, TimerMode::Repeating),
        })
        .insert_resource(PlayerCollision(2))
        .add_systems(
            Update,
            player_movement.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        )
        .add_systems(
            Update,
            skill_animation.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        )
        .add_systems(
            Update,
            check_ground.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        )
        .add_systems(Update, update_pose.run_if(in_state(AppState::Ingame)))
        .add_systems(
            Update,
            check_attack.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        )
        .add_systems(
            Update,
            avoid_collision.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        )
        .add_systems(
            Update,
            update_damage_display.run_if(in_state(AppState::Ingame)),
        )
        .add_systems(
            Update,
            update_health_bar.run_if(in_state(AppState::Ingame)),
        )
        .add_systems(
            Update,
            update_energy_bar.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        )
        .add_systems(
            Update,
            update_fire_bar.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>))
        )
        .add_systems(
            Update,
            update_fire_animation.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        )
        .add_systems(
            Update,
            update_facing.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        )
        .add_systems(
            Update,
            update_soul_absorb_animation
                .run_if(in_state(AppState::Ingame).and(resource_exists::<SoulAbsorb>)),
        );

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            keyboard_input.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        );
    }
}
