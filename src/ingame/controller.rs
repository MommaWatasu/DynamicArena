use bevy::prelude::*;

use crate::{
    ingame::{player::*, pose::*, Fighting},
    AppState, GameConfig,
};

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

            if gamepad.just_pressed(GamepadButton::South) {
                if !player
                    .state
                    .check(PlayerState::KICKING | PlayerState::PUNCHING)
                {
                    if player.state.check(PlayerState::JUMP_UP) {
                        player.state |= PlayerState::KICKING;
                        player.set_animation(JUMPING_KICK_POSE, 0, 10);
                    } else if !player.state.check(PlayerState::WALKING) {
                        player.state |= PlayerState::KICKING;
                        player.set_animation(KICK_POSE1, 0, 10);
                    }
                }
            }

            if gamepad.pressed(GamepadButton::South) {
                if player.state.check(PlayerState::JUMP_UP)
                    & !player.state.check(PlayerState::KICKING)
                {
                    player.state |= PlayerState::KICKING;
                    player.set_animation(JUMPING_KICK_POSE, 0, 10);
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
