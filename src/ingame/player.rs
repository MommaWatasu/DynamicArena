use bevy::prelude::*;
use crate::{ingame::InGame, AppState};

const PLAYER_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);

const LIMB_LENGTH: f32 = 30.0;
const LIMB_RADIUS: f32 = 15.0;

const HEAD_OFFSET: f32 = 100.0;
const BODY_OFFSET: f32 = 40.0;
const UPPER_ARM_OFFSET: f32 = 0.0;
const LOWER_ARM_OFFSET: f32 = -60.0;
const UPPER_LEG_OFFSET: f32 = -100.0;
const LOWER_LEG_OFFSET: f32 = -60.0;

const FPS: f32 = 60.0;

#[derive(Debug, Clone, Copy, Default)]
struct Pose {
    // true means right facing, false means left facing
    facing: bool,
    head: f32,
    body: f32,
    right_upper_arm: f32,
    right_lower_arm: f32,
    right_upper_leg: f32,
    right_lower_leg: f32,
    left_upper_arm: f32,
    left_lower_arm: f32,
    left_upper_leg: f32,
    left_lower_leg: f32,
}

#[derive(Component, Clone, Copy)]
struct PlayerID(u8);

#[derive(Default, Clone, Copy)]
enum PlayerState {
    #[default]
    Idle,
    Running,
    Jumping,
    Falling,
}

#[derive(Resource)]
struct AnimationTimer {
    timer: Timer
}

struct PlayerAnimation {
    phase: u8,
    count: u8,
}

#[derive(Component)]
struct Player {
    pose: Pose,
    animation: PlayerAnimation,
    state: PlayerState,
    speed: Vec2,
    health: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pose: Pose {
                facing: true,
                head: 0.0,
                body: 0.0,
                right_upper_arm: 10.0,
                right_lower_arm: 90.0,
                right_upper_leg: 10.0,
                right_lower_leg: -40.0,
                left_upper_arm: 30.0,
                left_lower_arm: 90.0,
                left_upper_leg: 40.0,
                left_lower_leg: -50.0,
            },
            animation: PlayerAnimation { phase: 0, count: 0 },
            state: PlayerState::default(),
            speed: Vec2::ZERO,
            health: 100,
        }
    }
}

#[derive(Component)]
struct BodyParts {
    flags: u8
}

impl BodyParts {
    const HEAD: Self = Self { flags: 0b10000 };
    const BODY: Self = Self { flags: 0b01000 };
    pub fn new(head: bool, body: bool, arm: bool, right: bool, upper: bool) -> Self {
        Self {
            flags: (head as u8) << 4 | (body as u8) << 3 | (arm as u8) << 2 | (right as u8) << 1 | (upper as u8)
        }
    }
    /*
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
    */
}

pub fn spawn_player(
    id: u8,
    builder: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    builder.spawn((
        Player::default(),
        PlayerID(id),
        InGame,
        // Player 0 is on top of the screen
        Transform::from_translation(Vec3::new(if id == 0 {-500.0} else {500.0}, -300.0, if id == 0 { 10.0 } else {1.0})),
        Visibility::Visible,
    ))
        // Body
        .with_children(|builder| {
            builder.spawn((
                Mesh2d(meshes.add(Capsule2d {
                    radius: LIMB_RADIUS,
                    half_length: 60.0,
                })),
                MeshMaterial2d(materials.add(PLAYER_COLOR)),
                Transform::default(),
                BodyParts::BODY,
                PlayerID(id),
            ))
                // Head
                .with_children(|builder| {
                    builder.spawn((
                        Mesh2d(meshes.add(Circle::new(50.0))),
                        MeshMaterial2d(materials.add(PLAYER_COLOR)),
                        BodyParts::HEAD,
                        Transform::from_translation(Vec3::new(0.0, 100.0, 1.0)),
                    ));
                    // Right Upper Arm
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(PLAYER_COLOR)),
                        BodyParts::new(false, false, true, true, true),
                        PlayerID(id),
                        // player 0 is right facing, and player 1 is left facing
                        // so we need to change which arm is on top
                        Transform::from_translation(Vec3::new(0.0, 0.0, if id == 0 { 3.0 } else { 1.0 })),
                    ))
                        // Right Lower Arm
                        .with_child((
                            Mesh2d(meshes.add(Capsule2d {
                                radius: LIMB_RADIUS,
                                half_length: LIMB_LENGTH,
                            })),
                            MeshMaterial2d(materials.add(PLAYER_COLOR)),
                            BodyParts::new(false, false, true, true, false),
                            PlayerID(id),
                            Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
                        ));
                    // Left Upper Arm
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(PLAYER_COLOR)),
                        BodyParts::new(false, false, true, false, true),
                        PlayerID(id),
                        // player 0 is right facing, and player 1 is left facing
                        // so we need to change which arm is on top
                        Transform::from_translation(Vec3::new(0.0, 0.0, if id == 0 { 1.0 } else { 3.0 })),
                    ))
                        // Left Lower Arm
                        .with_child((
                            Mesh2d(meshes.add(Capsule2d {
                                radius: LIMB_RADIUS,
                                half_length: LIMB_LENGTH,
                            })),
                            MeshMaterial2d(materials.add(PLAYER_COLOR)),
                            BodyParts::new(false, false, true, false, false),
                            PlayerID(id),
                            Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
                        ));
                    // Right Upper Leg
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(PLAYER_COLOR)),
                        // right upper leg
                        BodyParts::new(false, false, false, true, true),
                        PlayerID(id),
                        // player 0 is right facing, and player 1 is left facing
                        // so we need to change which leg is on top
                        Transform::from_translation(Vec3::new(0.0, -100.0, if id == 0 { 3.0 } else { 1.0 })),
                    ))
                        // Right Lower Leg
                        .with_children(|builder| {
                            builder.spawn((
                                Mesh2d(meshes.add(Capsule2d {
                                    radius: LIMB_RADIUS,
                                    half_length: LIMB_LENGTH,
                                })),
                                MeshMaterial2d(materials.add(PLAYER_COLOR)),
                                // right lower leg
                                BodyParts::new(false, false, false, true, false),
                                PlayerID(id),
                                Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
                            ));
                        });
                    // Left Upper Leg
                    builder.spawn((
                        Mesh2d(meshes.add(Capsule2d {
                            radius: LIMB_RADIUS,
                            half_length: LIMB_LENGTH,
                        })),
                        MeshMaterial2d(materials.add(PLAYER_COLOR)),
                        BodyParts::new(false, false, false, false, true),
                        PlayerID(id),
                        Transform::from_translation(Vec3::new(0.0, -100.0, if id == 0 { 1.0 } else { 3.0 })),
                    ))
                        // Left Lower Leg
                        .with_children(|builder| {
                            builder.spawn((
                                Mesh2d(meshes.add(Capsule2d {
                                    radius: LIMB_RADIUS,
                                    half_length: LIMB_LENGTH,
                                })),
                                MeshMaterial2d(materials.add(PLAYER_COLOR)),
                                BodyParts::new(false, false, false, false, false),
                                PlayerID(id),
                                Transform::from_translation(Vec3::new(0.0, -60.0, 1.0)),
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
    if keys.pressed(KeyCode::KeyD) {
        for mut player in query.iter_mut() {
            if let PlayerState::Running = player.state {
            } else {
                player.state = PlayerState::Running;
                player.pose.body = -30.0;
                player.pose.left_upper_arm = 60.0;
                player.pose.left_lower_arm = 90.0;
                player.pose.right_upper_arm = -60.0;
                player.pose.right_lower_arm = 90.0;
                player.pose.right_upper_leg = 60.0;
                player.pose.right_lower_leg = -90.0;
                player.pose.left_upper_leg = -30.0;
                player.pose.left_lower_leg = -50.0;
            }
            player.pose.facing = true;
        }

    } else if keys.pressed(KeyCode::KeyA) {
        for mut player in query.iter_mut() {
            if let PlayerState::Running = player.state {
            } else {
                player.state = PlayerState::Running;
                player.pose.body = -30.0;
                player.pose.left_upper_arm = 60.0;
                player.pose.left_lower_arm = 90.0;
                player.pose.right_upper_arm = -60.0;
                player.pose.right_lower_arm = 90.0;
                player.pose.right_upper_leg = 60.0;
                player.pose.right_lower_leg = -90.0;
                player.pose.left_upper_leg = -30.0;
                player.pose.left_lower_leg = -50.0;
            }
            player.pose.facing = false;
        }
    } else if keys.pressed(KeyCode::Space) {
    } else {
        for mut player in query.iter_mut() {
            if let PlayerState::Idle = player.state {
            } else {
                player.state = PlayerState::Idle;
                player.pose.body = 0.0;
                player.pose.right_upper_arm = 10.0;
                player.pose.right_lower_arm = 90.0;
                player.pose.left_upper_arm = 30.0;
                player.pose.left_lower_arm = 90.0;
                player.pose.right_upper_leg = 10.0;
                player.pose.right_lower_leg = -40.0;
                player.pose.left_upper_leg = 40.0;
                player.pose.left_lower_leg = -50.0;
            }
        }
    }
}

fn player_movement(
    time: Res<Time>,
    mut timer: ResMut<AnimationTimer>,
    mut player_query: Query<(&mut Player, &mut Transform)>
) {
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        for (mut player, mut transform) in player_query.iter_mut() {
            match player.state {
                PlayerState::Idle => {
                    if player.animation.phase == 0 {
                        player.animation.count += 1;
                        if player.animation.count == 10 {
                            player.animation.phase = 1;
                            player.animation.count = 0;
                        }
                    } else if player.animation.phase == 1 {
                        player.pose.right_upper_leg += 1.0;
                        player.pose.right_lower_leg -= 1.0;
                        player.pose.left_upper_leg += 1.0;
                        player.pose.left_lower_leg -= 1.0;
                        player.animation.count += 1;
                        if player.animation.count == 10 {
                            player.animation.phase = 2;
                            player.animation.count = 0;
                        }
                    } else if player.animation.phase == 2 {
                        player.pose.right_upper_leg -= 1.0;
                        player.pose.right_lower_leg += 1.0;
                        player.pose.left_upper_leg -= 1.0;
                        player.pose.left_lower_leg += 1.0;
                        player.animation.count += 1;
                        if player.animation.count == 10 {
                            player.animation.phase = 0;
                            player.animation.count = 0;
                        }
                    }
                }
                PlayerState::Running => {
                    if player.pose.facing {
                        transform.translation.x += 6.0;
                        /*if player.animation.phase == 0 {
                            player.pose.right_upper_leg += 1.0;
                            player.animation.count += 1;
                        }*/
                    } else {
                        transform.translation.x -= 6.0;
                    }
                }
                _ => {}
            }
        }
    }
}

fn update_pose(
    player_query: Query<(&Player, &PlayerID)>,
    mut parts_query: Query<(&BodyParts, &PlayerID, &mut Transform)>
) {
    for (player, player_id) in player_query.iter() {
        for (parts, parts_id, mut transform) in parts_query.iter_mut() {
            if player_id.0 == parts_id.0 {
                match parts.flags {
                    // Head
                    0b10000 => rotate_parts(&mut transform, HEAD_OFFSET, player.pose.head),
                    // Body
                    0b01000 => rotate_parts(&mut transform, BODY_OFFSET, player.pose.body),
                    // Right Upper Arm
                    0b00111 => rotate_parts(&mut transform, UPPER_ARM_OFFSET, player.pose.right_upper_arm),
                    // Right Lower Arm
                    0b00110 => rotate_parts(&mut transform, LOWER_ARM_OFFSET, player.pose.right_lower_arm),
                    // Right Upper Leg
                    0b00011 => rotate_parts(&mut transform, UPPER_LEG_OFFSET, player.pose.right_upper_leg),
                    // Right Lower Leg
                    0b00010 => rotate_parts(&mut transform, LOWER_LEG_OFFSET, player.pose.right_lower_leg),
                    // Left Upper Arm
                    0b00101 => rotate_parts(&mut transform, UPPER_ARM_OFFSET, player.pose.left_upper_arm),
                    // Left Lower Arm
                    0b00100 => rotate_parts(&mut transform, LOWER_ARM_OFFSET, player.pose.left_lower_arm),
                    // Left Upper Leg
                    0b00001 => rotate_parts(&mut transform, UPPER_LEG_OFFSET, player.pose.left_upper_leg),
                    // Left Lower Leg
                    0b00000 => rotate_parts(&mut transform, LOWER_LEG_OFFSET, player.pose.left_lower_leg),
                    _ => {}
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
            .add_systems(Update, player_input.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, player_movement.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, update_pose.run_if(in_state(AppState::Ingame)));
    }
}