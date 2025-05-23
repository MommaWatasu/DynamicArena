use crate::{
    character_def::*,
    ingame::{player::*, pose::*, rand, Fighting},
    AppState, GameConfig, GameMode,
};
use bevy::prelude::*;

// Agent select action every 1/AGENT_FREQUENCY seconds
const AGENT_FREQUENCY: f32 = 30.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Level {
    Easy = 1,
    Normal = 2,
    Hard = 3,
}

impl From<u32> for Level {
    fn from(value: u32) -> Self {
        match value {
            1 => Level::Easy,
            2 => Level::Normal,
            3 => Level::Hard,
            _ => panic!("Invalid Level: {}", value),
        }
    }
}

enum Policy {
    Offensive,
    Defensive,
    Neutral,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Action {
    MoveForward,
    MoveBackward,
    JumpUP,
    JumpForward,
    JumpBackward,
    JumpKick,
    Bend,
    Kick,
    BackKick,
    RangedAttack,
    Punch,
    None,
}

#[derive(Default)]
struct Environment {
    agent_state: PlayerState,
    agent_health: f32,
    player_health: f32,
    distance: f32,
    player_state: PlayerState,
    agent_facing: bool,
    agent_energy: u8,
    agent_fire_charge: u16,
    player_energy: u8,
    player_fire_charge: u16,
}

#[derive(Resource)]
pub struct Agent {
    timer: Timer,
    count: u8,
    level: Level,
    policy: Policy,
}

impl Agent {
    pub fn new(level: Level) -> Self {
        Self {
            timer: Timer::from_seconds(0.3 / AGENT_FREQUENCY, TimerMode::Repeating),
            count: 0,
            level,
            policy: Policy::Neutral,
        }
    }
    fn select_policy(&mut self, environment: &Environment) {
        match self.level {
            // Easy: weak strategy
            // in this mode, agent performs few patterns
            Level::Easy => {
                if environment.agent_health < 0.5 {
                    if environment.distance < 150.0 {
                        self.policy = Policy::Defensive;
                    } else if environment.distance > 500.0 {
                        self.policy = Policy::Neutral;
                    } else {
                        self.policy = Policy::Offensive;
                    }
                } else {
                    if environment.distance < 150.0 {
                        self.policy = Policy::Offensive;
                    } else if environment.distance > 500.0 {
                        self.policy = Policy::Neutral;
                    } else {
                        self.policy = Policy::Defensive;
                    }
                }
            }
            // Normal: normal strategy
            // in this mode, agent performs adequate patterns
            // and more aggressive than easy mode
            Level::Normal => {
                if environment.agent_health < 0.3 {
                    if environment.distance < 150.0 {
                        self.policy = Policy::Defensive;
                    } else if environment.distance < 500.0 {
                        let rand = rand();
                        if rand < 0.4 {
                            self.policy = Policy::Defensive;
                        } else if rand < 0.7 {
                            self.policy = Policy::Neutral;
                        } else {
                            self.policy = Policy::Offensive;
                        }
                    } else {
                        let rand = rand();
                        if rand < 0.2 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    }
                } else if environment.agent_health < 0.6 {
                    if environment.distance < 300.0 {
                        let rand = rand();
                        if rand < 0.8 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Defensive;
                        }
                    } else if environment.distance < 600.0 {
                        let rand = rand();
                        if rand < 0.4 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    } else {
                        let rand = rand();
                        if rand < 0.4 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    }
                } else {
                    if environment.distance < 400.0 {
                        let rand = rand();
                        if rand < 0.8 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Defensive;
                        }
                    } else if environment.distance < 600.0 {
                        let rand = rand();
                        if rand < 0.5 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    } else {
                        self.policy = Policy::Offensive;
                    }
                }
            }
            Level::Hard => {
                if environment.agent_health < 0.3 {
                    if environment.distance < 150.0 {
                        self.policy = Policy::Defensive;
                    } else if environment.distance < 500.0 {
                        let rand = rand();
                        if rand < 0.4 {
                            self.policy = Policy::Defensive;
                        } else if rand < 0.7 {
                            self.policy = Policy::Neutral;
                        } else {
                            self.policy = Policy::Offensive;
                        }
                    } else {
                        let rand = rand();
                        if rand < 0.2 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    }
                } else if environment.agent_health < 0.6 {
                    if environment.distance < 300.0 {
                        let rand = rand();
                        if rand < 0.7 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Defensive;
                        }
                    } else if environment.distance < 600.0 {
                        let rand = rand();
                        if rand < 0.5 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    } else {
                        let rand = rand();
                        if rand < 0.5 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    }
                } else {
                    if environment.distance < 400.0 {
                        self.policy = Policy::Offensive;
                    } else if environment.distance < 600.0 {
                        let rand = rand();
                        if rand < 0.6 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    } else {
                        self.policy = Policy::Offensive;
                    }
                }
            }
        }
    }
    fn select_action(&self, environment: &Environment) -> Action {
        if environment.agent_state.check(PlayerState::COOLDOWN) {
            return Action::None;
        }
        match self.policy {
            Policy::Offensive => {
                if environment.distance < 150.0 {
                    let rand = rand();
                    if rand < 0.2 {
                        return Action::Kick;
                    } else if rand < 0.5 {
                        return Action::Punch;
                    // FIXME: FRONT KICK is removed.
                    // new action should be added...
                    } else if rand < 0.95 {
                        return Action::BackKick;
                    } else {
                        return Action::RangedAttack;
                    }
                } else if environment.distance < 225.0 {
                    let rand = rand();
                    if rand < 0.8 {
                        return Action::BackKick;
                    } else {
                        return Action::RangedAttack;
                    }
                } else if environment.distance < 325.0 {
                    return Action::MoveForward;
                } else if environment.distance < 500.0
                    && (environment.player_state.check(PlayerState::BEND_DOWN)
                        || environment.player_state.is_idle()) {
                    return Action::JumpKick;
                } else if environment.distance < 500.0 
                    && (environment.player_state.check(PlayerState::MoveForward)){
                    return Action::RangedAttack;
                } else if environment.distance < 500.0
                    && (environment.player_state.check(PlayerState::JumpAttack)
                        || environment.player_state.check(PlayerState::Jumpforward)) {
                        return Action::BackKick;
                } else if environment.distance < 1050.0 {
                    let rand = rand();
                    if rand < 0.6 {
                        return Action::MoveForward;
                    } else {
                        return Action::RangedAttack;
                    }
                } else {
                    let rand = rand();
                    if rand < 0.2 {
                        return Action::JumpForward;
                    } else {
                        return Action::MoveForward;
                    }
                }
            }
            Policy::Defensive => {
                if environment.player_state.check(PlayerState::KICKING) {
                    return Action::JumpUP;
                }
                if environment.distance < 100.0 {
                    let rand = rand();
                    if rand < 0.5 {
                        return Action::JumpBackward;
                    } else {
                        return Action::MoveBackward;
                    }
                } else if environment.distance < 250.0 {
                    return Action::MoveBackward;
                }
                if environment.player_state.check(PlayerState::PUNCHING) {
                    return Action::Bend;
                }
                return Action::MoveBackward;
            }
            Policy::Neutral => {
                let rand = rand();
                if rand < 0.7 {
                    return Action::None;
                }
                else {
                    return Action::Bend;
                }
            }
        }
    }
}

pub fn agent_system(
    time: Res<Time>,
    game_config: Res<GameConfig>,
    mut agent: ResMut<Agent>,
    mut player_query: Query<(&mut Player, &PlayerID, &Transform)>,
) {
    // Skip if multiplayer
    if game_config.mode == GameMode::MultiPlayer {
        return;
    }
    agent.timer.tick(time.delta());
    if agent.timer.finished() {
        agent.count += 1;
        let mut environment = Environment::default();
        if let Some((player, _, transform)) = player_query.iter().find(|(_, id, _)| id.0 == 0) {
            environment.player_health = player.health as f32
                / CHARACTER_PROFILES[player.character_id as usize].health as f32;
            environment.player_state = player.state;
            environment.distance = transform.translation.x;
            environment.player_energy = player.energy;
            environment.player_fire_charge = player.fire_charge;
        }
        if let Some((player, _, transform)) = player_query.iter().find(|(_, id, _)| id.0 == 1) {
            environment.agent_health = player.health as f32
                / CHARACTER_PROFILES[player.character_id as usize].health as f32;
            environment.agent_facing = player.pose.facing;
            environment.distance = (transform.translation.x - environment.distance).abs();
            environment.agent_state = player.state;
            environment.agent_energy = player.energy;
            environment.agent_fire_charge = player.fire_charge;
        }
        if agent.count == AGENT_FREQUENCY as u8 * 2 {
            agent.count = 0;
            agent.select_policy(&environment);
        }
        let action = agent.select_action(&environment);
        if let Some((mut player, _, _)) = player_query.iter_mut().find(|(_, id, _)| id.0 == 1) {
            if action != Action::MoveForward && action != Action::MoveBackward {
                // agent is idle
                player.state &= !PlayerState::WALKING;
            }
            match action {
                Action::MoveForward => {
                    if !player.state.check(
                        PlayerState::JUMP_UP
                            | PlayerState::JUMP_BACKWARD
                            | PlayerState::JUMP_FORWARD
                            | PlayerState::BEND_DOWN
                            | PlayerState::ROLL_BACK
                            | PlayerState::ROLL_FORWARD
                            | PlayerState::WALKING
                    ) {
                        // player is just walking
                        player.state |= PlayerState::WALKING;
                        player.set_animation(WALKING_POSE1, 0, 10);
                    }
                    if player.pose.facing {
                        // direction is right
                        player.state |= PlayerState::DIRECTION;
                    } else {
                        // direction is left
                        player.state &= !PlayerState::DIRECTION;
                    }
                }
                Action::MoveBackward => {
                    if !player.state.check(
                        PlayerState::JUMP_UP
                            | PlayerState::JUMP_BACKWARD
                            | PlayerState::JUMP_FORWARD
                            | PlayerState::BEND_DOWN
                            | PlayerState::ROLL_BACK
                            | PlayerState::ROLL_FORWARD
                            | PlayerState::WALKING
                    ) {
                        // player is just walking
                        player.state |= PlayerState::WALKING;
                        player.set_animation(WALKING_POSE1, 0, 10);
                    }
                    if player.pose.facing {
                        // direction is right
                        player.state &= !PlayerState::DIRECTION;
                    } else {
                        // direction is left
                        player.state |= PlayerState::DIRECTION;
                    }
                }
                Action::JumpUP => {
                    // character 0 and 2 can only single jump
                    if player.state.is_idle() {
                        // player is idle
                        // then player will jump up
                        player.state |= PlayerState::JUMP_UP;
                        player.set_animation(JUMP_UP_POSE1, 0, 10);
                        player.velocity = Vec2::new(0.0, 12.0);
                        player.energy += 1;
                    } else if !player.state.check(
                        PlayerState::JUMP_UP
                            | PlayerState::JUMP_FORWARD
                            | PlayerState::JUMP_BACKWARD,
                    ) && player.state.check(PlayerState::WALKING)
                    {
                        if player.state.check(PlayerState::DIRECTION) {
                            // player is walking right
                            // then player will jump forward
                            player.state |= PlayerState::JUMP_FORWARD;
                            player.set_animation(JUMP_FORWARD_POSE1, 0, 10);
                            let x_vel =
                                CHARACTER_PROFILES[player.character_id as usize].agility;
                            player.velocity = Vec2::new(x_vel, 12.0);
                        } else {
                            // player is walking left
                            // then player will jump backward
                            player.state |= PlayerState::JUMP_BACKWARD;
                            player.set_animation(JUMP_BACKWARD_POSE1, 0, 10);
                            let x_vel =
                                CHARACTER_PROFILES[player.character_id as usize].agility;
                            player.velocity = Vec2::new(-x_vel, 12.0);
                        }
                        player.energy += 1;
                    }
                }
                Action::JumpForward => {
                    if player.state.is_idle() {
                        // agent is walking right
                        // then player will jump forward
                        if player.pose.facing {
                            player.state |= PlayerState::DIRECTION;
                        } else {
                            player.state &= !PlayerState::DIRECTION;
                        }
                        player.state |= PlayerState::JUMP_FORWARD;
                        player.set_animation(JUMP_FORWARD_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
                        player.energy += 1;
                    }
                }
                Action::JumpBackward => {
                    if player.state.is_idle() {
                        // agent is walking left
                        // then player will jump backward
                        if !player.pose.facing {
                            player.state &= !PlayerState::DIRECTION;
                        } else {
                            player.state |= PlayerState::DIRECTION;
                        }
                        player.state |= PlayerState::JUMP_BACKWARD;
                        player.set_animation(JUMP_BACKWARD_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
                        player.energy += 1;
                    }
                }
                Action::JumpKick => {
                    if player.state.is_idle() {
                        // agent is walking right
                        // then player will jump kick
                        if player.pose.facing {
                            player.state |= PlayerState::DIRECTION;
                        } else {
                            player.state &= !PlayerState::DIRECTION;
                        }
                        player.state |= PlayerState::JUMP_FORWARD | PlayerState::KICKING;
                        player.set_animation(JUMP_FORWARD_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
                        player.energy += 2;
                    }
                }
                Action::Bend => {
                    if player.state.is_idle() {
                        // player is idle
                        // then player will bend
                        player.state |= PlayerState::BEND_DOWN;
                        player.set_animation(BEND_DOWN_POSE1, 0, 5);
                    }
                }
                Action::Kick => {
                    if player.state.is_idle() {
                        // player is idle
                        // then player will kick
                        player.state |= PlayerState::KICKING;
                        player.set_animation(KICK_POSE1, 0, 10);
                        player.energy += 2;
                    } else if player.state.check(
                        PlayerState::JUMP_UP
                            | PlayerState::JUMP_FORWARD
                            | PlayerState::JUMP_BACKWARD,
                    ) {
                        // player is jumping
                        // then just adding state
                        player.state |= PlayerState::KICKING;
                        player.energy += 2;
                    }
                }
                Action::BackKick => {
                    if player.state.is_idle() {
                        // player is idle
                        // then player will back kick
                        player.state |= PlayerState::BACK_KICKING;
                        player.set_animation(BACK_KICK_POSE1, 0, 10);
                        player.energy += 2;
                    }
                }
                Action::RangedAttack => {
                    if player.state.is_idle() && player.fire_charge == FIRE_CHARGE_MAX {
                        // player is idle
                        // then player will knee kick
                        player.state |= PlayerState::RANGED_ATTACK;
                        player.set_animation(PUNCH_POSE, 0, 10);
                        player.energy += 2;
                    }
                }
                Action::Punch => {
                    if player.state.is_idle() {
                        // player is idle
                        // then player will punch
                        player.state |= PlayerState::PUNCHING;
                        player.set_animation(PUNCH_POSE, 0, 10);
                        player.energy += 2;
                    }
                }
                Action::None => {
                    if player.state.check(PlayerState::WALKING) {
                        // player is walking
                        // then player will idle
                        player.state &= !PlayerState::WALKING;
                        if player.state.is_idle() {
                            player.set_animation(IDLE_POSE1, 0, 10);
                        }
                    }
                }
            }
        }
    }
}

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Agent::new(Level::Normal)).add_systems(
            Update,
            agent_system.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        );
    }
}
