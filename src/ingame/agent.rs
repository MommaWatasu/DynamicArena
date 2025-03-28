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
    None
}

#[derive(Default)]
struct Environment {
    agent_health: u32,
    player_health: u32,
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
        if environment.agent_health < 50 {
            self.policy = Policy::Defensive;
        } else if environment.agent_health > 50 {
            self.policy = Policy::Offensive;
        } else {
            self.policy = Policy::Neutral;
        }
    }
    fn select_action(&self, environment: &Environment) -> Action {
        match self.policy {
            Policy::Offensive => {
                if environment.distance < 100.0 {
                    return Action::Kick;
                } else {
                    if environment.agent_facing {
                        return Action::MoveRight;
                    } else {
                        return Action::MoveLeft;
                    }
                }
            }
            Policy::Defensive => {
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
            environment.player_health = player.health;
            environment.player_state = player.state;
            environment.distance = transform.translation.x;
        }
        if let Some((player, _, transform)) = player_query.iter().find(|(_, id, _)| id.0 == 1) {
            environment.agent_health = player.health;
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
                }
                Action::MoveLeft => {
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
                }
                Action::JumpUP => {
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
                Action::JumpRight => {}
                Action::JumpLeft => {}
                Action::Bend => {}
                Action::Kick => {}
                Action::None => {}
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