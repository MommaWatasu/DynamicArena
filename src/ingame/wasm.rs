use bevy::{
    prelude::*,
    input::touch::TouchPhase
};
use std::f32::consts::PI;
use crate::{
    GameConfig,
    character_def::*,
    ingame::{
        player::*,
        pose::*
    }
};

pub const CONTROLLER_CIRCLE_RADIUS: f32 = 70.0;

#[derive(Component)]
pub struct ControllerCircle;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum CircleState {
    Right,
    UpRight,
    Up,
    UpLeft,
    Left,
    DownLeft,
    Down,
    DownRight,
    None,
}

#[derive(Resource)]
pub struct TouchState {
    pub start_position: Vec2,
    pub id: u64,
}

/// Convert the touch position to the world position
fn convert_touch_to_world(touch_position: Vec2, window_size: Vec2) -> Vec2 {
    Vec2::new(touch_position.x - window_size.x/2.0, - touch_position.y + window_size.y/2.0)
}

pub fn touch_input(
    config: Res<GameConfig>,
    mut touch_state: ResMut<TouchState>,
    mut touch_evr: EventReader<TouchInput>,
    mut circle_query: Query<&mut Transform, With<ControllerCircle>>,
    mut player_query: Query<(&mut Player, &PlayerID)>,
) {
    for event in touch_evr.read() {
        match event.phase {
            TouchPhase::Started => {
                // check if the touch is in the controller circle
                if convert_touch_to_world(event.position, config.window_size).distance(Vec2::new(-config.window_size.x/2.0+100.0, -config.window_size.y/4.0)) <= CONTROLLER_CIRCLE_RADIUS && touch_state.id == u64::MAX {
                    touch_state.start_position = convert_touch_to_world(event.position, config.window_size);
                    touch_state.id = event.id;
                }
            }
            TouchPhase::Moved => {
                // check if the finger is the same to the one that started the touch in controller circle
                if touch_state.id == event.id {
                    let mut circle_transform = circle_query.single_mut();
                    circle_transform.translation.x = convert_touch_to_world(event.position, config.window_size).x - touch_state.start_position.x;
                    circle_transform.translation.y = convert_touch_to_world(event.position, config.window_size).y - touch_state.start_position.y;
                    // limit the circle movement to the radius
                    if Vec2::new(circle_transform.translation.x, circle_transform.translation.y).distance(Vec2::ZERO) > CONTROLLER_CIRCLE_RADIUS {
                        let angle = Vec2::new(circle_transform.translation.x, circle_transform.translation.y).angle_to(Vec2::X);
                        circle_transform.translation.x = angle.cos() * CONTROLLER_CIRCLE_RADIUS;
                        circle_transform.translation.y = -angle.sin() * CONTROLLER_CIRCLE_RADIUS;
                    }
                }
            }
            TouchPhase::Ended => {
                // check if the finger is the same to the one that started the touch in controller circle
                if touch_state.id == event.id {
                    touch_state.start_position = Vec2::ZERO;
                    touch_state.id = u64::MAX;
                    let mut circle_transform = circle_query.single_mut();
                    circle_transform.translation = Vec3::new(0.0, 0.0, 1.0);
                }
            }
            TouchPhase::Canceled => {
            }
        }
    }

    // get the angle of the controller circle
    let circle_transform = circle_query.single();
    let circle_radian = Vec2::new(circle_transform.translation.x, circle_transform.translation.y).angle_to(Vec2::X);
    // convert angle to state
    let mut circle_state = CircleState::None;
    if circle_transform.translation == Vec3::new(0.0, 0.0, 1.0) {
        // none
        circle_state = CircleState::None;
    } else if circle_radian >= -PI/8.0 && circle_radian < PI/8.0 {
        // right
        circle_state = CircleState::Right;
    } else if circle_radian >= PI/8.0 && circle_radian < 3.0*PI/8.0 {
        // down right
        circle_state = CircleState::DownRight;
    } else if circle_radian >= 3.0*PI/8.0 && circle_radian < 5.0*PI/8.0 {
        // down
        circle_state = CircleState::Down;
    } else if circle_radian >= 5.0*PI/8.0 && circle_radian < 7.0*PI/8.0 {
        // down left
        circle_state = CircleState::DownLeft;
    } else if circle_radian >= 7.0*PI/8.0 || circle_radian < -7.0*PI/8.0 {
        // left
        circle_state = CircleState::Left;
    } else if circle_radian >= -7.0*PI/8.0 && circle_radian < -5.0*PI/8.0 {
        // up left
        circle_state = CircleState::UpLeft;
    } else if circle_radian >= -5.0*PI/8.0 && circle_radian < -3.0*PI/8.0 {
        // up
        circle_state = CircleState::Up;
    } else if circle_radian >= -3.0*PI/8.0 && circle_radian < -PI/8.0 {
        // up right
        circle_state = CircleState::UpRight;
    }
    // change state of player 1
    if let Some((mut player, _)) = player_query.iter_mut().find(|(_, player_id)| player_id.0 == 0) {
        match circle_state {
            CircleState::Right => {
                if player.state.check(PlayerState::JUMP_UP | PlayerState::DOUBLE_JUMP) {
                    player.velocity.x = CHARACTER_PROFILES[player.character_id as usize].agility;
                } else if !player.state.check(PlayerState::WALKING) {
                    player.state |= PlayerState::WALKING;
                    player.set_animation(WALKING_POSE1, 0, 10);
                }
                player.state |= PlayerState::DIRECTION;
            }
            CircleState::Left => {
                if player.state.check(PlayerState::JUMP_UP | PlayerState::DOUBLE_JUMP) {
                    player.velocity.x = -CHARACTER_PROFILES[player.character_id as usize].agility;
                } else if !player.state.check(PlayerState::WALKING) {
                    player.state |= PlayerState::WALKING;
                    player.set_animation(WALKING_POSE1, 0, 10);
                }
                player.state &= !PlayerState::DIRECTION;
            }
            CircleState::Up => {
                if player.state.is_idle() {
                    player.state |= PlayerState::JUMP_UP;
                    player.set_animation(JUMP_UP_POSE1, 0, 10);
                    player.velocity = Vec2::ZERO;
                }
            }
            CircleState::UpRight => {
                if player.state.is_idle() {
                    player.state |= PlayerState::JUMP_FORWARD;
                    player.set_animation(JUMP_FORWARD_POSE1, 0, 10);
                    let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                    player.velocity = Vec2::ZERO;
                }
            }
            CircleState::UpLeft => {
                if player.state.is_idle() {
                    player.state |= PlayerState::JUMP_BACKWARD;
                    player.set_animation(JUMP_BACKWARD_POSE1, 0, 10);
                    let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                    player.velocity = Vec2::ZERO;
                }
            }
            CircleState::Down => {
                if player.state.is_idle() {
                    player.state |= PlayerState::BEND_DOWN;
                    player.set_animation(BEND_DOWN_POSE1, 0, 10);
                }
            }
            CircleState::DownRight => {
                if player.state.is_idle() {
                    player.state |= PlayerState::ROLL_FORWARD;
                    player.set_animation(ROLL_FORWARD_POSE1, 0, 10);
                }
            }
            CircleState::DownLeft => {
                if player.state.is_idle() {
                    player.state |= PlayerState::ROLL_BACK;
                    player.set_animation(ROLL_BACK_POSE1, 0, 10);
                }
            }
            CircleState::None => {
                if player.state.check(PlayerState::WALKING) {
                    player.state &= !PlayerState::WALKING;
                    player.set_animation(IDLE_POSE1, 0, 10);
                }
            }
        }
    }
}