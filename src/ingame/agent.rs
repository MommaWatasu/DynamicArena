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
    /// Convert a u32 value to a Level enum
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

struct PolicyScore {
    offensive: f32,
    defensive: f32,
    neutral: f32,
}

impl PolicyScore {
    /// Determine the best policy based on highest score
    fn get_best_policy(&self) -> Policy {
        if self.offensive >= self.defensive && self.offensive >= self.neutral {
            Policy::Offensive
        } else if self.defensive >= self.neutral {
            Policy::Defensive
        } else {
            Policy::Neutral
        }
    }
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
    Skill,
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
    health_advantage: f32,  // positive if agent has more health
    energy_advantage: i16,  // positive if agent has more energy
    fire_charge_advantage: i32, // positive if agent has more fire charge
    is_player_vulnerable: bool, // player is in stunned or cooldown state
}

#[derive(Resource)]
pub struct Agent {
    timer: Timer,
    count: u8,
    level: Level,
    policy: Policy,
}

impl Agent {
    /// Create a new agent with specified difficulty level
    pub fn new(level: Level) -> Self {
        Self {
            timer: Timer::from_seconds(0.12 / AGENT_FREQUENCY, TimerMode::Repeating),
            count: 0,
            level,
            policy: Policy::Neutral,
        }
    }
    /// Select the appropriate combat policy based on environment and difficulty level
    fn select_policy(&mut self, environment: &Environment) {
        match self.level {
            // Easy: Simplified strategy with basic patterns
            Level::Easy => {
                let policy_score = self.calculate_policy_score(environment);
                
                if environment.agent_health < 0.5 {
                    // Low health - be more cautious
                    if !environment.agent_state.is_idle() {
                        self.policy = Policy::Neutral;
                    } else if environment.distance < 150.0 {
                        let rand = rand();
                        if rand < 0.8 {  // Increased defensive tendency
                            self.policy = Policy::Defensive;
                        } else {
                            self.policy = Policy::Offensive;
                        }
                    } else if environment.distance > 500.0 {
                        self.policy = if policy_score.offensive > 0.3 { Policy::Offensive } else { Policy::Neutral };
                    } else {
                        // Medium distance - consider player state
                        if environment.is_player_vulnerable {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Defensive;
                        }
                    }
                } else {
                    // Good health - more aggressive
                    if environment.distance < 150.0 {
                        self.policy = if environment.is_player_vulnerable { Policy::Offensive } else { 
                            if rand() < 0.7 { Policy::Offensive } else { Policy::Defensive }
                        };
                    } else if environment.distance > 500.0 {
                        self.policy = if policy_score.offensive > 0.4 { Policy::Offensive } else { Policy::Neutral };
                    } else {
                        self.policy = if environment.player_energy > 80 { Policy::Defensive } else { Policy::Offensive };
                    }
                }
            }
            // Normal: Enhanced strategy with better situational awareness
            Level::Normal => {
                let policy_score = self.calculate_policy_score(environment);
                
                // Critical health threshold
                if environment.agent_health < 0.3 {
                    self.policy = self.select_desperate_policy(environment);
                } else if environment.agent_health < 0.6 {
                    // Medium health - balanced approach
                    if environment.is_player_vulnerable {
                        self.policy = Policy::Offensive;
                    } else if environment.distance < 200.0 && environment.player_energy < 30 {
                        // Player is tired at close range - be aggressive
                        self.policy = Policy::Offensive;
                    } else if environment.distance < 300.0 {
                        let combined_score = policy_score.offensive - policy_score.defensive;
                        if combined_score > 0.2 {
                            self.policy = Policy::Offensive;
                        } else if combined_score < -0.2 {
                            self.policy = Policy::Defensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    } else {
                        // Long range strategy
                        if environment.agent_fire_charge > environment.player_fire_charge + 30 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    }
                } else {
                    // Good health - more confident
                    if environment.health_advantage > 0.2 {
                        // Health advantage - maintain pressure
                        self.policy = if environment.distance > 400.0 { Policy::Neutral } else { Policy::Offensive };
                    } else {
                        // Use comprehensive scoring
                        let best_policy = policy_score.get_best_policy();
                        self.policy = best_policy;
                    }
                }
            }
            Level::Hard => {
                // Advanced strategy for Hard mode
                // Priority: Resource advantage, precise timing, prediction
                if environment.is_player_vulnerable {
                    self.policy = Policy::Offensive;
                } else if environment.health_advantage > 0.3 {
                    // Significant health advantage - maintain pressure
                    if environment.distance < 300.0 {
                        self.policy = Policy::Offensive;
                    } else {
                        let rand = rand();
                        if rand < 0.7 {
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    }
                } else if environment.health_advantage < -0.3 {
                    // Health disadvantage - be cautious
                    if environment.distance < 150.0 {
                        self.policy = Policy::Defensive;
                    } else {
                        let rand = rand();
                        if rand < 0.5 {
                            self.policy = Policy::Defensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    }
                } else {
                    // Balanced situation - use resource advantage
                    if environment.energy_advantage > 20 || environment.fire_charge_advantage > 50 {
                        self.policy = Policy::Offensive;
                    } else if environment.energy_advantage < -20 {
                        self.policy = Policy::Defensive;
                    } else {
                        // Dynamic policy based on distance and player state
                        if environment.distance < 200.0 {
                            let rand = rand();
                            if rand < 0.6 {
                                self.policy = Policy::Offensive;
                            } else {
                                self.policy = Policy::Defensive;
                            }
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    }
                }
            }
        }
    }
    
    /// Calculate scores for each policy based on environmental factors
    fn calculate_policy_score(&self, environment: &Environment) -> PolicyScore {
        let mut offensive_score: f32 = 0.0;
        let mut defensive_score: f32 = 0.0;
        let mut neutral_score: f32 = 0.0;
        
        // Health-based scoring
        if environment.health_advantage > 0.2 {
            offensive_score += 0.3;
        } else if environment.health_advantage < -0.2 {
            defensive_score += 0.3;
        } else {
            neutral_score += 0.1;
        }
        
        // Energy advantage scoring
        if environment.energy_advantage > 15 {
            offensive_score += 0.2;
        } else if environment.energy_advantage < -15 {
            defensive_score += 0.2;
        }
        
        // Fire charge advantage
        if environment.fire_charge_advantage > 30 {
            offensive_score += 0.15;
        }
        
        // Distance-based scoring
        if environment.distance < 200.0 {
            if environment.agent_health > 0.6 {
                offensive_score += 0.2;
            } else {
                defensive_score += 0.15;
            }
        } else if environment.distance > 500.0 {
            neutral_score += 0.2;
            if environment.agent_fire_charge == FIRE_CHARGE_MAX {
                offensive_score += 0.1;
            }
        }
        
        // Player state considerations
        if environment.is_player_vulnerable {
            offensive_score += 0.4; // Strong incentive to attack vulnerable opponents
        }
        
        if environment.player_state.check(PlayerState::WALKING) && environment.distance < 300.0 {
            offensive_score += 0.1;
        }
        
        // Agent state considerations
        if !environment.agent_state.is_idle() {
            neutral_score += 0.1; // Prefer neutral when not idle
        }
        
        // Level-specific adjustments
        match self.level {
            Level::Easy => {
                defensive_score += 0.1; // Slightly more defensive
            }
            Level::Normal => {
                // Balanced - no adjustment
            }
            Level::Hard => {
                offensive_score += 0.15; // More aggressive
                if environment.agent_energy == ENERGY_MAX {
                    offensive_score += 0.1; // Ready for skill usage
                }
            }
        }
        
        PolicyScore {
            offensive: offensive_score.max(0.0).min(1.0),
            defensive: defensive_score.max(0.0).min(1.0),
            neutral: neutral_score.max(0.0).min(1.0),
        }
    }
    
    /// Choose policy when agent has critically low health
    fn select_desperate_policy(&self, environment: &Environment) -> Policy {
        // When in critical health, make calculated risks
        if environment.is_player_vulnerable {
            // Take the opportunity to attack when player is vulnerable
            Policy::Offensive
        } else if environment.distance < 150.0 {
            // Too close when low health - be defensive
            Policy::Defensive
        } else if environment.distance > 400.0 {
            // Safe distance - try to recover or use ranged attacks
            if environment.agent_fire_charge == FIRE_CHARGE_MAX {
                Policy::Offensive
            } else {
                Policy::Neutral
            }
        } else {
            // Medium distance - assess situation carefully
            if environment.player_energy < 20 {
                // Player is tired - risky but potentially rewarding attack
                Policy::Offensive
            } else if environment.energy_advantage > 10 {
                // We have energy advantage despite low health
                Policy::Offensive
            } else {
                // Play it safe
                Policy::Defensive
            }
        }
    }
    
    /// Select the best action based on current policy and environment
    fn select_action(&self, environment: &Environment) -> Action {
        if environment.agent_state.check(PlayerState::COOLDOWN) {
            return Action::None;
        }
        
        // Priority actions based on game state
        if self.should_use_skill(environment) {
            return Action::Skill;
        }
        
        if self.should_counter_attack(environment) {
            return self.get_counter_action(environment);
        }
        
        if self.should_punish_vulnerability(environment) {
            return self.get_punishment_action(environment);
        }
        
        match self.policy {
            Policy::Offensive => self.select_offensive_action(environment),
            Policy::Defensive => self.select_defensive_action(environment),
            Policy::Neutral => self.select_neutral_action(environment),
        }
    }
    
    /// Check if agent should use skill ability
    fn should_use_skill(&self, environment: &Environment) -> bool {
        environment.agent_energy == ENERGY_MAX && 
        (environment.player_state.check(PlayerState::IDLE) || 
         environment.player_state.check(PlayerState::WALKING) ||
         environment.is_player_vulnerable) &&
        environment.distance < 400.0
    }
    
    /// Check if agent should counter-attack player's action
    fn should_counter_attack(&self, environment: &Environment) -> bool {
        environment.player_state.check(PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::BACK_KICKING)
    }
    
    /// Get appropriate counter-action based on player's attack
    fn get_counter_action(&self, environment: &Environment) -> Action {
        if environment.player_state.check(PlayerState::KICKING) {
            if environment.distance < 150.0 {
                return Action::JumpUP;
            } else {
                return Action::MoveBackward;
            }
        }
        if environment.player_state.check(PlayerState::PUNCHING) {
            if environment.distance < 120.0 {
                return Action::Bend;
            } else {
                return Action::BackKick;
            }
        }
        if environment.player_state.check(PlayerState::BACK_KICKING) {
            return Action::MoveForward;
        }
        Action::None
    }
    
    /// Check if player is vulnerable and within punish range
    fn should_punish_vulnerability(&self, environment: &Environment) -> bool {
        environment.is_player_vulnerable && environment.distance < 300.0
    }
    
    /// Get action to punish vulnerable player
    fn get_punishment_action(&self, environment: &Environment) -> Action {
        if environment.distance < 150.0 {
            let rand = rand();
            if rand < 0.4 {
                Action::Kick
            } else if rand < 0.7 {
                Action::Punch
            } else {
                Action::BackKick
            }
        } else if environment.distance < 300.0 {
            if environment.agent_fire_charge == FIRE_CHARGE_MAX {
                Action::RangedAttack
            } else {
                Action::JumpKick
            }
        } else {
            Action::MoveForward
        }
    }
    
    /// Select aggressive action based on distance and resources
    fn select_offensive_action(&self, environment: &Environment) -> Action {
        // Improved offensive strategy with better resource management
        if environment.distance < 150.0 {
            let rand = rand();
            if environment.agent_energy >= 80 && rand < 0.3 {
                return Action::Kick;
            } else if rand < 0.4 {
                return Action::Punch;
            } else if rand < 0.6 && environment.agent_fire_charge == FIRE_CHARGE_MAX {
                return Action::RangedAttack;
            } else {
                return Action::BackKick;
            }
        } else if environment.distance < 200.0 {
            let rand = rand();
            if rand < 0.4 && environment.agent_fire_charge == FIRE_CHARGE_MAX {
                return Action::RangedAttack;
            } else if rand < 0.7 {
                return Action::BackKick;
            } else {
                return Action::MoveForward;
            }
        } else if environment.distance < 350.0 {
            // More aggressive approach at medium distance
            if environment.player_state.check(PlayerState::WALKING) {
                return Action::JumpKick;
            } else {
                return Action::MoveForward;
            }
        } else if environment.distance < 500.0 {
            if environment.player_state.check(PlayerState::BEND_DOWN) || 
               environment.player_state.is_idle() {
                return Action::JumpKick;
            } else if environment.player_state.check(PlayerState::WALKING) && 
                     environment.agent_fire_charge == FIRE_CHARGE_MAX {
                return Action::RangedAttack;
            } else if environment.player_state.check(PlayerState::JUMP_FORWARD) {
                return Action::BackKick;
            } else {
                return Action::MoveForward;
            }
        } else if environment.distance < 800.0 {
            let rand = rand();
            if rand < 0.6 && environment.agent_fire_charge == FIRE_CHARGE_MAX {
                return Action::RangedAttack;
            } else if rand < 0.8 {
                return Action::MoveForward;
            } else {
                return Action::JumpForward;
            }
        } else {
            let rand = rand();
            if rand < 0.3 {
                return Action::JumpForward;
            } else {
                return Action::MoveForward;
            }
        }
    }
    
    /// Select defensive action to avoid damage and maintain distance
    fn select_defensive_action(&self, environment: &Environment) -> Action {
        // Enhanced defensive strategy
        if environment.distance < 80.0 {
            let rand = rand();
            if rand < 0.6 {
                return Action::JumpBackward;
            } else {
                return Action::MoveBackward;
            }
        } else if environment.distance < 200.0 {
            if environment.player_state.check(PlayerState::WALKING) {
                return Action::MoveBackward;
            } else {
                let rand = rand();
                if rand < 0.3 && environment.agent_fire_charge == FIRE_CHARGE_MAX {
                    return Action::RangedAttack;
                } else {
                    return Action::MoveBackward;
                }
            }
        } else if environment.distance < 400.0 {
            // Maintain distance and look for opportunities
            if environment.player_state.check(PlayerState::JUMP_FORWARD) {
                return Action::BackKick;
            } else if environment.agent_fire_charge == FIRE_CHARGE_MAX && 
                     environment.player_state.check(PlayerState::WALKING) {
                return Action::RangedAttack;
            } else {
                return Action::MoveBackward;
            }
        } else {
            // Safe distance - prepare for counter-attack
            let rand = rand();
            if rand < 0.4 {
                return Action::None;
            } else {
                return Action::Bend;
            }
        }
    }
    
    /// Select neutral action for positioning and resource management
    fn select_neutral_action(&self, environment: &Environment) -> Action {
        let rand = rand();
        if rand < 0.5 {
            return Action::None;
        } else if rand < 0.8 {
            return Action::Bend;
        } else if environment.agent_fire_charge == FIRE_CHARGE_MAX && 
                 environment.distance > 300.0 && environment.distance < 600.0 {
            return Action::RangedAttack;
        } else {
            return Action::None;
        }
    }
}

/// Main agent system that controls AI behavior and decision making
pub fn agent_system(
    mut commands: Commands,
    mut fighting: ResMut<Fighting>,
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
        
        // Calculate enhanced environment variables
        environment.health_advantage = environment.agent_health - environment.player_health;
        environment.energy_advantage = environment.agent_energy as i16 - environment.player_energy as i16;
        environment.fire_charge_advantage = environment.agent_fire_charge as i32 - environment.player_fire_charge as i32;
        environment.is_player_vulnerable = environment.player_state.check(
            PlayerState::STUN | PlayerState::COOLDOWN | PlayerState::SKILL
        ) || (environment.player_state.check(PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::BACK_KICKING) 
              && environment.distance > 200.0);
        
        if agent.count == AGENT_FREQUENCY as u8 * 2 {
            agent.count = 0;
            agent.select_policy(&environment);
        }
        let action = agent.select_action(&environment);
        if let Some((mut player, player_id, _)) = player_query.iter_mut().find(|(_, id, _)| id.0 == 1) {
            if action != Action::MoveForward && action != Action::MoveBackward {
                // agent is idle
                player.state &= !PlayerState::WALKING;
            }
            if action != Action::Bend {
                // agent is not bending
                player.state &= !PlayerState::BEND_DOWN;
            }
            match action {
                Action::MoveForward => {
                    if player.state.is_idle() {
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
                    if player.state.is_idle() {
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
                    ) && !player.state.check(PlayerState::KICKING) {
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
                        player.fire_charge = 0;
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
                Action::Skill => {
                    if player.state.is_idle() && player.energy == ENERGY_MAX {
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
        app.insert_resource(Agent::new(Level::Hard)).add_systems(
            Update,
            agent_system.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        );
    }
}
