use bevy::prelude::*;

use crate::{
    ingame::{player::*, pose::*, Fighting},
    character_def::{CHARACTER_PROFILES, FIRE_CHARGE_MAX},
    AppState, GameConfig, GameMode, CharacterTextures,
};

// TODO: controller system is old and needs to be updated based on the new animation system
fn controller_system(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut fighting: ResMut<Fighting>,
    character_textures: Res<CharacterTextures>,
    gamepads: Query<(&Gamepad, Entity)>,
    mut player_query: Query<(&mut Player, &PlayerID, &mut Sprite, &mut Transform)>,
) {
    // skill animation
    // ignore controller input during skill animation
    if fighting.0 != 0 {
        return;
    }

    update_facing(&mut player_query);

    #[allow(unused_assignments)]
    let mut id = 0;
    for (gamepad, entity) in gamepads.iter() {
        if game_config.gamepads[0] == entity {
            id = 0;
        } else {
            id = 1;
        }
        
        // シングルプレイの場合、コントローラー2の入力は無視
        if game_config.mode == GameMode::SinglePlayer && id == 1 {
            continue;
        }
        for (mut player, player_id, mut sprite, _) in player_query.iter_mut() {
            if player_id.0 != id {
                continue;
            } else if player.state.check(PlayerState::COOLDOWN) {
                continue;
            }

            if gamepad.pressed(GamepadButton::DPadLeft) {
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
                    player.pose.set(WALKING_POSE1);
                    player.set_animation(WALKING_POSE2, 1, 15);
                }
                // direction is left
                player.state &= !PlayerState::DIRECTION;
            } else if gamepad.pressed(GamepadButton::DPadRight) {
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
                    player.pose.set(WALKING_POSE1);
                    player.set_animation(WALKING_POSE2, 1, 15);
                }
                // direction is right
                player.state |= PlayerState::DIRECTION;
            } else {
                // player is not walking
                if player.state.check(PlayerState::WALKING) {
                    player.state &= !PlayerState::WALKING;
                    player.velocity = Vec2::ZERO;
                    if player.state.is_idle() {
                        sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                        sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                        player.animation_frame_max = FRAMES_IDLE;
                        player.pose.set(IDLE_POSE1);
                        player.set_animation(IDLE_POSE2, 1, 15);
                    }
                }
            }

            if gamepad.pressed(GamepadButton::DPadDown) {
                if player.state.is_idle() {
                    // player is idle
                    // then player will bend down
                    sprite.image = character_textures.textures[player.character_id as usize].bend_down.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_BEND_DOWN;
                    player.state |= PlayerState::BEND_DOWN;
                    player.pose.set(BEND_DOWN_POSE1);
                    player.set_animation(BEND_DOWN_POSE2, 0, 27);
                    // This is for testing purpose
                    player.energy += 1;
                } else if player.state.is_just_walk() {
                    if player.pose.facing {
                        if player.state.check(PlayerState::DIRECTION) {
                            // player is walking right
                            // then player will roll forward
                            sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                            player.animation_frame_max = FRAMES_ROLL;
                            player.state |= PlayerState::ROLL_FORWARD;
                            player.pose.set(ROLL_FORWARD_POSE1);
                            player.set_animation(ROLL_FORWARD_POSE2, 0, 11);
                        } else {
                            // player is walking left
                            // then player will roll back
                            sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = FRAMES_ROLL - 1);
                            player.animation_frame_max = FRAMES_ROLL;
                            player.state |= PlayerState::ROLL_BACK;
                            player.pose.set(ROLL_BACK_POSE1);
                            player.set_animation(ROLL_BACK_POSE2, 0, 4);
                        }
                    } else {
                        if !player.state.check(PlayerState::DIRECTION) {
                            // player is walking right
                            // then player will roll forward
                            sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                            player.animation_frame_max = FRAMES_ROLL;
                            player.state |= PlayerState::ROLL_FORWARD;
                            player.pose.set(ROLL_FORWARD_POSE1);
                            player.set_animation(ROLL_FORWARD_POSE2, 0, 11);
                        } else {
                            // player is walking left
                            // then player will roll back
                            sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = FRAMES_ROLL - 1);
                            player.animation_frame_max = FRAMES_ROLL;
                            player.state |= PlayerState::ROLL_BACK;
                            player.pose.set(ROLL_BACK_POSE1);
                            player.set_animation(ROLL_BACK_POSE2, 0, 4);
                        }
                    }
                    let x_vel = if player.state.is_forward() { 1.0 } else { -1.0 }
                        * CHARACTER_PROFILES[player.character_id as usize].agility * 2.0;
                    player.velocity = Vec2::new(x_vel, 0.0);
                }
            } else if player.state.check(PlayerState::BEND_DOWN) && player.animation.phase != 2 {
                // player is bending down
                // then stop bending down
                player.set_animation(BEND_DOWN_POSE1, 2, 23);
            }

            if gamepad.just_pressed(GamepadButton::DPadUp) {
                if player.state.is_idle() {
                    // player is idle
                    // then player will jump up
                    sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_JUMP;
                    player.state |= PlayerState::JUMP_UP;
                    player.pose.set(JUMP_POSE1);
                    player.set_animation(JUMP_POSE2, 0, 11);
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
                            player.state = PlayerState::JUMP_FORWARD;
                            player.pose.set(JUMP_POSE1);
                            player.set_animation(JUMP_POSE2, 0, 11);
                            // stop moving for preparing motion
                            player.velocity = Vec2::ZERO;
                            player.energy += 1;
                        } else {
                            // player is walking left
                            // then player will jump backward
                            sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                            player.animation_frame_max = FRAMES_JUMP;
                            player.state = PlayerState::JUMP_BACKWARD;
                            player.pose.set(JUMP_POSE1);
                            player.set_animation(JUMP_POSE2, 0, 11);
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
                            player.state = PlayerState::JUMP_FORWARD;
                            player.pose.set(JUMP_POSE1);
                            player.set_animation(JUMP_POSE2, 0, 11);
                            // stop moving for preparing motion
                            player.velocity = Vec2::ZERO;
                            player.energy += 1;
                        } else {
                            // player is walking left
                            // then player will jump backward
                            sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                            player.animation_frame_max = FRAMES_JUMP;
                            player.state = PlayerState::JUMP_BACKWARD;
                            player.pose.set(JUMP_POSE1);
                            player.set_animation(JUMP_POSE2, 0, 11);
                            // stop moving for preparing motion
                            player.velocity = Vec2::ZERO;
                            player.energy += 1;
                        }
                    }
                }
            }

            if gamepad.just_pressed(GamepadButton::West) {
                if ((player.pose.facing && gamepad.pressed(GamepadButton::DPadLeft))
                    || (!player.pose.facing && gamepad.pressed(GamepadButton::DPadRight)))
                    && !player.state.check(PlayerState::BACK_KICKING) {
                    // player is idle
                    // then player will back kick
                    sprite.image = character_textures.textures[player.character_id as usize].back_kick.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_BACK_KICK;
                    player.state |= PlayerState::BACK_KICKING;
                    player.pose.set(BACK_KICK_POSE1);
                    player.set_animation(BACK_KICK_POSE2, 0, 6);
                    player.energy += 2;
                }
                if player.state.is_idle() {
                    // player is idle
                    // then player will kick
                    sprite.image = character_textures.textures[player.character_id as usize].kick.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_KICK;
                    player.state |= PlayerState::KICKING;
                    player.pose.set(KICK_POSE1);
                    player.set_animation(KICK_POSE2, 0, 21);
                    player.energy += 2;
                } else if player
                    .state
                    .check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD)
                    && !player.state.check(PlayerState::KICKING)
                    && player.animation.phase > 0
                    && player.animation.phase < 4
                {
                    // player is jumping
                    // then just adding state
                    sprite.image = character_textures.textures[player.character_id as usize].jump_kick.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = 1;
                    player.pose.set(JUMP_KICK_POSE);
                    player.state |= PlayerState::KICKING;
                    player.energy += 2;
                }
            }
            if gamepad.just_pressed(GamepadButton::East) {
                if player.state.is_idle() {
                    // player is idle
                    // then player will punch
                    sprite.image = character_textures.textures[player.character_id as usize].punch.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_PUNCH;
                    player.state |= PlayerState::PUNCHING;
                    // the first pose for punch similar to that of kick
                    player.pose.set(KICK_POSE1);
                    player.set_animation(PUNCH_POSE, 0, 19);
                    player.energy += 2;
                }
            }
            if gamepad.just_pressed(GamepadButton::North) {
                if player.state.is_idle() && player.fire_charge == FIRE_CHARGE_MAX {
                    // player is idle
                    // player will do ranged attack
                    sprite.image = character_textures.textures[player.character_id as usize].punch.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_PUNCH;
                    player.fire_charge = 0;
                    player.state |= PlayerState::RANGED_ATTACK;
                    // the first pose for punch similar to that of kick
                    player.pose.set(KICK_POSE1);
                    player.set_animation(PUNCH_POSE, 0, 19);
                    player.energy += 2;
                }
            }
            if gamepad.just_pressed(GamepadButton::South) && player.energy == 100 {
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
