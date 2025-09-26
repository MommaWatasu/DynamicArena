use bevy::prelude::*;

use crate::{
    ingame::{player::*, pose::*, Fighting},
    character_def::CHARACTER_PROFILES,
    AppState, GameConfig,
};

// TODO: controller system is old and needs to be updated based on the new animation system
fn controller_system(
    game_config: Res<GameConfig>,
    gamepads: Query<(&Gamepad, Entity)>,
    mut query: Query<(&mut Player, &PlayerID)>,
) {
    #[allow(unused_assignments)]
    let mut id = 0;
    for (gamepad, entity) in gamepads.iter() {
        if game_config.gamepads[0] == entity {
            id = 0;
        } else {
            id = 1;
        }
        for (mut player, player_id) in query.iter_mut() {
            if player_id.0 != id {
                continue;
            } else if player.state.check(PlayerState::COOLDOWN) {
                continue;
            }

            if gamepad.pressed(GamepadButton::DPadLeft) {
                if player.state.is_idle() {
                    // player is just walking
                    player.state |= PlayerState::WALKING;
                    player.set_animation(WALKING_POSE1, 0, 10);
                }
                // direction is left
                player.state &= !PlayerState::DIRECTION;
            } else if gamepad.pressed(GamepadButton::DPadRight) {
                if player.state.is_idle() {
                    // player is just walking
                    player.state |= PlayerState::WALKING;
                    player.set_animation(WALKING_POSE1, 0, 10);
                }
                // direction is right
                player.state |= PlayerState::DIRECTION;
            } else {
                // player is not walking
                if player.state.check(PlayerState::WALKING) {
                    player.state &= !PlayerState::WALKING;
                    if player.state.is_idle() {
                        player.set_animation(IDLE_POSE1, 0, 10);
                    }
                }
            }

            if gamepad.pressed(GamepadButton::DPadDown) {
                if player.state.is_idle() {
                    // player is idle
                    // then player will bend down
                    player.state |= PlayerState::BEND_DOWN;
                    player.set_animation(BEND_DOWN_POSE1, 0, 5);
                } else if player.state.is_just_walk() && player.state.check(PlayerState::WALKING) {
                    if player.pose.facing {
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
                    } else {
                        if !player.state.check(PlayerState::DIRECTION) {
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
                    }
                    let x_vel = if player.state.is_forward() { 1.0 } else { -1.0 }
                        * CHARACTER_PROFILES[player.character_id as usize].agility * 2.0;
                    player.velocity = Vec2::new(x_vel, 0.0);
                }
            } else if player.state.check(PlayerState::BEND_DOWN) {
                // player is bending down
                // then stop bending down
                player.state &= !PlayerState::BEND_DOWN;
                player.set_animation(IDLE_POSE1, 0, 10);
            }

            if gamepad.just_pressed(GamepadButton::DPadUp) {
                if player.state.is_idle() {
                    // player is idle
                    // then player will jump up
                    player.state |= PlayerState::JUMP_UP;
                    player.set_animation(JUMP_POSE1, 0, 10);
                    player.energy += 1;
                } else if player.state.is_just_walk() && player.state.check(PlayerState::WALKING)
                {
                    if player.pose.facing {
                        if player.state.check(PlayerState::DIRECTION) {
                            // player is walking right
                            // then player will jump forward
                            player.state |= PlayerState::JUMP_FORWARD;
                            player.set_animation(JUMP_POSE1, 0, 10);
                            // stop moving for preparing motion
                            player.velocity = Vec2::ZERO;
                            player.energy += 1;
                        } else {
                            // player is walking left
                            // then player will jump backward
                            player.state |= PlayerState::JUMP_BACKWARD;
                            player.set_animation(JUMP_POSE1, 0, 10);
                            // stop moving for preparing motion
                            player.velocity = Vec2::ZERO;
                            player.energy += 1;
                        }
                    } else {
                        if !player.state.check(PlayerState::DIRECTION) {
                            // player is walking right
                            // then player will jump forward
                            player.state |= PlayerState::JUMP_FORWARD;
                            player.set_animation(JUMP_POSE1, 0, 10);
                            // stop moving for preparing motion
                            player.velocity = Vec2::ZERO;
                            player.energy += 1;
                        } else {
                            // player is walking left
                            // then player will jump backward
                            player.state |= PlayerState::JUMP_BACKWARD;
                            player.set_animation(JUMP_POSE1, 0, 10);
                            // stop moving for preparing motion
                            player.velocity = Vec2::ZERO;
                            player.energy += 1;
                        }
                    }
                }
            }

            if gamepad.just_pressed(GamepadButton::West) {
                if player.state.is_idle() {
                    // player is idle
                    // then player will kick
                    player.state |= PlayerState::KICKING;
                    player.set_animation(KICK_POSE1, 0, 5);
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
        }
    }
}

pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            controller_system.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        );
    }
}
