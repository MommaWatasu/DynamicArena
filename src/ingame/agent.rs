use bevy::prelude::*;
use crate::{
    AppState,
    GameMode,
    GameConfig,
    character_def::*,
    ingame::{
        player::*,
        pose::*,
        Fighting
    }
};

// Agent select action every 1/AGENT_FREQUENCY seconds
const AGENT_FREQUENCY: f32 = 30.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Level {
    Easy = 1,
    Normal = 2,
    Hard = 3
}

impl From<u32> for Level {
    fn from(value: u32) -> Self {
        match value {
            1 => Level::Easy,
            2 => Level::Normal,
            3 => Level::Hard,
            _ => panic!("Invalid Level: {}", value)
        }
    }
}

enum Policy {
    Offensive,
    Defensive,
    Neutral
}

enum Action {
    MoveRight,
    MoveLeft,
    JumpUP,
    JumpRight,
    JumpLeft,
    Bend,
    Kick,
    BackKick,
    FrontKick,
    Punch,
    Ignore,
    None
}

#[derive(Default)]
struct Environment {
    agent_health: f32,
    player_health: f32,
    distance: f32,
    player_state: PlayerState,
    agent_facing: bool
}

#[derive(Resource)]
pub struct Agent {
    timer: Timer,
    count: u8,
    level: Level,
    policy: Policy
}

#[cfg(not(target_arch="wasm32"))]
fn rand() -> f32 {
    rand::random::<f32>()
}
#[cfg(target_arch="wasm32")]
fn rand() -> f32 {
    web_sys::js_sys::Math::random() as f32
}

impl Agent {
    pub fn new(level: Level) -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / AGENT_FREQUENCY, TimerMode::Repeating),
            count: 0,
            level,
            policy: Policy::Neutral
        }
    }
    fn select_policy(&mut self, environment: &Environment) {
        if environment.agent_health < 0.3 {
            self.policy = Policy::Defensive;
        } else if environment.agent_health > 0.6 {
            self.policy = Policy::Offensive;
        } else {
            self.policy = Policy::Neutral;
        }
    }
    fn select_action(&self, environment: &Environment) -> Action {
        match self.level {
            Level::Easy => {
                if rand() > 0.7 {
                    return Action::Ignore;
                }
            }
            Level::Normal => {
                if rand() > 0.3 {
                    return Action::Ignore;
                }
            }
            Level::Hard => {
                if rand() > 0.1 {
                    return Action::Ignore;
                }
            }
        }
        match self.policy {
            Policy::Offensive => {
                if environment.distance < 150.0 {
                    return Action::Kick;
                } else if environment.distance > 500.0 && (environment.player_state.check(PlayerState::BEND_DOWN) || environment.player_state.is_idle()) {
                    if environment.agent_facing {
                        return Action::JumpRight;
                    } else {
                        return Action::JumpLeft;
                    }
                } else {
                    if environment.agent_facing {
                        return Action::MoveRight;
                    } else {
                        return Action::MoveLeft;
                    }
                }
            }
            Policy::Defensive => {
                if environment.player_state.check(PlayerState::KICKING) {
                    return Action::JumpUP;
                }
                if environment.distance < 250.0 {
                    if environment.agent_facing {
                        return Action::JumpLeft;
                    } else {
                        return Action::MoveRight;
                    }
                }
                if environment.player_state.check(PlayerState::PUNCHING) {
                    return Action::Bend;
                }
                if environment.agent_facing {
                    return Action::MoveLeft;
                } else {
                    return Action::MoveRight;
                }
            }
            Policy::Neutral => {
                return Action::None;
            }
        }
    }
}

pub fn agent_system(
    time: Res<Time>,
    game_config: Res<GameConfig>,
    mut agent: ResMut<Agent>,
    mut player_query: Query<(&mut Player, &PlayerID, &Transform)>
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
            environment.player_health = player.health as f32 / CHARACTER_PROFILES[player.character_id as usize].health as f32;
            environment.player_state = player.state;
            environment.distance = transform.translation.x;
        }
        if let Some((player, _, transform)) = player_query.iter().find(|(_, id, _)| id.0 == 1) {
            environment.agent_health = player.health as f32 / CHARACTER_PROFILES[player.character_id as usize].health as f32;
            environment.agent_facing = player.pose.facing;
            environment.distance = (transform.translation.x - environment.distance).abs();
        }
        if agent.count == AGENT_FREQUENCY as u8 {
            agent.count = 0;
            agent.select_policy(&environment);
        }
        let action = agent.select_action(&environment);
        if let Some((mut player, _, _)) = player_query.iter_mut().find(|(_, id, _)| id.0 == 1) {
            match action {
                Action::MoveRight => {
                    if player.state.check(
                        PlayerState::JUMP_UP
                       
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
                }
                Action::MoveLeft => {
                    if player.state.check(
                        PlayerState::JUMP_UP
                       
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
                }
                Action::JumpUP => {
                    if player.character_id == 1 {
                        // character 1 can double jump
                        if player.state.is_idle() {
                            // player is idle
                            // then player will jump up
                            player.state |= PlayerState::JUMP_UP;
                            player.set_animation(JUMP_UP_POSE1, 0, 10);
                            player.velocity = Vec2::new(0.0, 12.0);                    
                        }
                    } else {
                        // character 0 and 2 can only single jump
                        if player.state.is_idle() {
                            // player is idle
                            // then player will jump up
                            player.state |= PlayerState::JUMP_UP;
                            player.set_animation(JUMP_UP_POSE1, 0, 10);
                            player.velocity = Vec2::new(0.0, 12.0);                    
                        } else if !player.state.check(
                            PlayerState::JUMP_UP
                               
                                | PlayerState::JUMP_FORWARD
                                | PlayerState::JUMP_BACKWARD
                        ) && player.state.check(PlayerState::WALKING) {
                            if player.state.check(PlayerState::DIRECTION) {
                                // player is walking right
                                // then player will jump forward
                                player.state |= PlayerState::JUMP_FORWARD;
                                player.set_animation(JUMP_FORWARD_POSE1, 0, 10);
                                let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                                player.velocity = Vec2::new(x_vel, 12.0);
                            } else {
                                // player is walking left
                                // then player will jump backward
                                player.state |= PlayerState::JUMP_BACKWARD;
                                player.set_animation(JUMP_BACKWARD_POSE1, 0, 10);
                                let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                                player.velocity = Vec2::new(-x_vel, 12.0);
                            }
                        }
                    }
                }
                Action::JumpRight => {
                    if player.state.is_idle() {
                        // agent is walking right
                        // then player will jump forward
                        player.state |= PlayerState::JUMP_FORWARD;
                        player.set_animation(JUMP_FORWARD_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
                    }
                }
                Action::JumpLeft => {
                    if player.state.is_idle() {
                        // agent is walking left
                        // then player will jump backward
                        player.state |= PlayerState::JUMP_BACKWARD;
                        player.set_animation(JUMP_BACKWARD_POSE1, 0, 10);
                        // stop moving for preparing motion
                        player.velocity = Vec2::ZERO;
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
                    } else if player.state.check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD | PlayerState::JUMP_BACKWARD) {
                        // player is jumping
                        // then just adding state
                        player.state |= PlayerState::KICKING;
                    }
                }
                Action::BackKick => {
                    if player.state.is_idle() {
                        // player is idle
                        // then player will back kick
                        player.state |= PlayerState::BACK_KICKING;
                        player.set_animation(BACK_KICK_POSE1, 0, 10);
                    } else if player.state.check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD | PlayerState::JUMP_BACKWARD) {
                        // player is jumping
                        // then just adding state
                        player.state |= PlayerState::BACK_KICKING;
                    }
                }
                Action::FrontKick => {
                    if player.state.is_idle() {
                        // player is idle
                        // then player will knee kick
                        player.state |= PlayerState::FRONT_KICKING;
                        player.set_animation(FRONT_KICK_POSE, 0, 10);
                    } else if player.state.check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD | PlayerState::JUMP_BACKWARD) {
                        // player is jumping
                        // then just adding state
                        player.state |= PlayerState::FRONT_KICKING;
                    }
                }
                Action::Punch => {
                    if player.state.is_idle() {
                        // player is idle
                        // then player will punch
                        player.state |= PlayerState::PUNCHING;
                        player.set_animation(PUNCH_POSE, 0, 10);
                    } else if player.state.check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD | PlayerState::JUMP_BACKWARD) {
                        // player is jumping
                        // then just adding state
                        player.state |= PlayerState::PUNCHING;
                    }
                }
                Action::Ignore => {}
                Action::None => {
                    if player.state.check(PlayerState::WALKING) {
                        // player is walking
                        // then player will idle
                        player.state &= !PlayerState::WALKING;
                        player.set_animation(IDLE_POSE1, 0, 10);
                    }
                }
            }
        }
    }
}

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Agent::new(Level::Normal))
            .add_systems(Update, agent_system.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)));
    }
}
