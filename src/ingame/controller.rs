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

            if gamepad.just_pressed(GamepadButton::West) {
                if player.state.is_idle() {
                    // player is idle
                    // then player will kick
                    player.state |= PlayerState::KICKING;
                    player.set_animation(KICK_POSE1, 0, 5);
                    player.energy += 2;
                }
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
