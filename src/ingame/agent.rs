use crate::{
    CharacterTextures,
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
    RollForward,
    RollBackward,
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

// Track the current action state to maintain continuity
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct ActionState {
    current_action: Action,
    frames_since_start: u8,
    planned_duration: u8,
    priority: ActionPriority,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
enum ActionPriority {
    Low,      // Can be interrupted easily (movement, bend)
    Medium,   // Should complete but can be interrupted for high priority (attacks)
    High,     // Must complete (skills, special moves)
    //Critical, // Cannot be interrupted (recovery states)
}

impl ActionState {
    fn new(action: Action) -> Self {
        let (planned_duration, priority) = match action {
            Action::MoveForward | Action::MoveBackward => (30, ActionPriority::Low),
            Action::Bend => (50, ActionPriority::Medium),
            Action::RollForward => (36, ActionPriority::Medium),
            Action::RollBackward => (36, ActionPriority::Medium),
            Action::JumpUP | Action::JumpForward | Action::JumpBackward => (58, ActionPriority::Medium),
            Action::JumpKick => (1, ActionPriority::Medium), // Short duration as it's added to existing jump
            Action::Kick => (46, ActionPriority::Medium),
            Action::Punch => (32, ActionPriority::Medium),
            Action::BackKick => (49, ActionPriority::Medium),
            Action::RangedAttack => (32, ActionPriority::Medium),
            Action::Skill => (180, ActionPriority::High),
            Action::None => (0, ActionPriority::Low),
        };
        
        Self {
            current_action: action,
            frames_since_start: 0,
            planned_duration,
            priority,
        }
    }
    
    fn should_continue(&self) -> bool {
        self.frames_since_start < self.planned_duration
    }
    
    fn can_be_interrupted_by(&self, new_priority: ActionPriority) -> bool {
        new_priority > self.priority || !self.should_continue()
    }
    
    fn tick(&mut self) {
        self.frames_since_start += 1;
    }
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
    action_state: ActionState,
    last_player_health: f32,
    engagement_timer: u8,  // Timer to start engagement even without being attacked
}

impl Agent {
    /// Create a new agent with specified difficulty level
    pub fn new(level: Level) -> Self {
        Self {
            timer: Timer::from_seconds(0.12 / AGENT_FREQUENCY, TimerMode::Repeating),
            count: 0,
            level,
            policy: Policy::Neutral,
            action_state: ActionState::new(Action::None),
            last_player_health: 1.0,
            engagement_timer: 0,
        }
    }
    /// Select the appropriate combat policy based on environment and difficulty level
    fn select_policy(&mut self, environment: &Environment) {
        // Update engagement timer for proactive behavior
        self.engagement_timer += 1;
        
        // Check if player health decreased (we attacked them or they took damage)
        let player_took_damage = environment.player_health < self.last_player_health;
        self.last_player_health = environment.player_health;
        
        // Determine if we should engage proactively
        let should_engage_proactively = self.should_engage_proactively(environment, player_took_damage);
        
        match self.level {
            // Easy: Simplified strategy with basic patterns
            Level::Easy => {
                if should_engage_proactively || environment.agent_health < environment.player_health {
                    let policy_score = self.calculate_policy_score(environment);
                    
                    if environment.agent_health < 0.5 {
                        // Low health - be more cautious but still engage
                        if environment.distance < 200.0 {
                            let rand = rand();
                            if rand < 0.6 {  // More defensive when low health
                                self.policy = Policy::Defensive;
                            } else {
                                self.policy = Policy::Offensive;
                            }
                        } else if environment.distance > 400.0 {
                            self.policy = if policy_score.offensive > 0.3 { Policy::Offensive } else { Policy::Neutral };
                        } else {
                            // Medium distance - consider player state
                            if environment.is_player_vulnerable {
                                self.policy = Policy::Offensive;
                            } else {
                                self.policy = Policy::Neutral;
                            }
                        }
                    } else {
                        // Good health - more aggressive engagement
                        if environment.distance < 200.0 {
                            self.policy = if environment.is_player_vulnerable { Policy::Offensive } else { 
                                if rand() < 0.7 { Policy::Offensive } else { Policy::Defensive }
                            };
                        } else if environment.distance > 500.0 {
                            self.policy = if policy_score.offensive > 0.4 { Policy::Offensive } else { Policy::Neutral };
                        } else {
                            // Proactive engagement at medium range
                            self.policy = if environment.player_energy > 80 { Policy::Defensive } else { Policy::Offensive };
                        }
                    }
                } else if !environment.agent_state.is_idle() {
                    // Continue current action if already engaged
                    self.policy = Policy::Neutral;
                } else {
                    // Stay neutral but ready to react
                    self.policy = Policy::Neutral;
                }
            }
            // Normal: Enhanced strategy with better situational awareness
            Level::Normal => {
                if should_engage_proactively || environment.health_advantage >= -0.1 {
                    let policy_score = self.calculate_policy_score(environment);
                    
                    // Critical health threshold
                    if environment.agent_health < 0.3 {
                        self.policy = self.select_desperate_policy(environment);
                    } else if environment.agent_health < 0.6 {
                        // Medium health - balanced but proactive approach
                        if environment.is_player_vulnerable {
                            self.policy = Policy::Offensive;
                        } else if environment.distance < 250.0 && environment.player_energy < 50 {
                            // Player is tired at close range - be aggressive
                            self.policy = Policy::Offensive;
                        } else if environment.distance < 350.0 {
                            let combined_score = policy_score.offensive - policy_score.defensive;
                            if combined_score > 0.1 {
                                self.policy = Policy::Offensive;
                            } else if combined_score < -0.1 {
                                self.policy = Policy::Defensive;
                            } else {
                                self.policy = Policy::Neutral;
                            }
                        } else {
                            // Long range strategy - engage if we have ranged advantage
                            if environment.agent_fire_charge > environment.player_fire_charge + 20 {
                                self.policy = Policy::Offensive;
                            } else {
                                self.policy = Policy::Neutral;
                            }
                        }
                    } else {
                        // Good health - confident and proactive
                        if environment.distance > 600.0 {
                            // Too far - close distance
                            self.policy = Policy::Offensive;
                        } else if environment.distance < 100.0 {
                            // Very close - use situation to our advantage
                            self.policy = if environment.player_energy < 30 { Policy::Offensive } else { Policy::Defensive };
                        } else {
                            // Optimal engagement range
                            let best_policy = policy_score.get_best_policy();
                            self.policy = best_policy;
                        }
                    }
                } else {
                    // Health disadvantage - be more cautious but still engage when advantageous
                    if environment.is_player_vulnerable {
                        self.policy = Policy::Offensive;
                    } else if environment.distance < 150.0 {
                        self.policy = Policy::Defensive;
                    } else {
                        self.policy = Policy::Neutral;
                    }
                }
            }
            Level::Hard => {
                // Always engaging and calculating - most proactive behavior
                if environment.is_player_vulnerable {
                    self.policy = Policy::Offensive;
                } else if should_engage_proactively || environment.health_advantage >= -0.2 {
                    if environment.health_advantage > 0.2 {
                        // Health advantage - maintain pressure
                        if environment.distance < 400.0 {
                            self.policy = Policy::Offensive;
                        } else {
                            // Close distance first
                            self.policy = Policy::Offensive;
                        }
                    } else if environment.health_advantage < -0.2 {
                        // Health disadvantage but still engage strategically
                        if environment.distance < 180.0 && environment.player_energy > 70 {
                            self.policy = Policy::Defensive;
                        } else if environment.energy_advantage > 15 || environment.fire_charge_advantage > 30 {
                            // Use resource advantage
                            self.policy = Policy::Offensive;
                        } else {
                            self.policy = Policy::Neutral;
                        }
                    } else {
                        // Balanced situation - proactive engagement
                        if environment.distance > 500.0 {
                            // Close distance for engagement
                            self.policy = Policy::Offensive;
                        } else if environment.distance < 200.0 {
                            let rand = rand();
                            if rand < 0.7 {
                                self.policy = Policy::Offensive;
                            } else {
                                self.policy = Policy::Defensive;
                            }
                        } else {
                            // Optimal fighting range
                            self.policy = Policy::Offensive;
                        }
                    }
                } else {
                    // Severe health disadvantage - desperate measures
                    self.policy = self.select_desperate_policy(environment);
                }
            }
        }
    }
    
    /// Determine if agent should engage proactively without waiting for damage
    fn should_engage_proactively(&self, environment: &Environment, player_took_damage: bool) -> bool {
        // Engage if we successfully attacked the player
        if player_took_damage {
            return true;
        }
        
        // Engage based on engagement timer (prevents standing around)
        let engagement_threshold = match self.level {
            Level::Easy => 90,   // 3 seconds at 30 FPS
            Level::Normal => 60, // 2 seconds
            Level::Hard => 30,   // 1 second
        };
        
        if self.engagement_timer > engagement_threshold {
            return true;
        }
        
        // Engage if player is within striking distance
        if environment.distance < 300.0 && environment.distance > 50.0 {
            return true;
        }
        
        // Engage if we have significant resource advantages
        if environment.energy_advantage > 30 || environment.fire_charge_advantage > 50 {
            return true;
        }
        
        // Engage if player is vulnerable
        if environment.is_player_vulnerable {
            return true;
        }
        
        // Engage if player is approaching aggressively
        if environment.player_state.check(PlayerState::WALKING | PlayerState::JUMP_FORWARD) && environment.distance < 400.0 {
            return true;
        }
        
        false
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
    fn select_action(&mut self, environment: &Environment) -> Action {
        // First, update the current action state timer
        self.action_state.tick();
        
        if environment.agent_state.check(PlayerState::COOLDOWN) {
            return Action::None;
        }
        
        // Check if we should continue current action
        if self.should_continue_current_action(environment) {
            return self.action_state.current_action;
        }
        
        // Priority actions based on game state
        if self.should_use_skill(environment) {
            let new_action = Action::Skill;
            if self.can_interrupt_for_action(new_action) {
                self.action_state = ActionState::new(new_action);
                return new_action;
            }
        }
        
        if self.should_counter_attack(environment) {
            let new_action = self.get_counter_action(environment);
            if self.can_interrupt_for_action(new_action) {
                self.action_state = ActionState::new(new_action);
                return new_action;
            }
        }
        
        if self.should_punish_vulnerability(environment) {
            let new_action = self.get_punishment_action(environment);
            if self.can_interrupt_for_action(new_action) {
                self.action_state = ActionState::new(new_action);
                return new_action;
            }
        }
        
        // Select action based on policy
        let new_action = match self.policy {
            Policy::Offensive => self.select_offensive_action(environment),
            Policy::Defensive => self.select_defensive_action(environment),
            Policy::Neutral => self.select_neutral_action(environment),
        };
        
        // Check if we can change to this new action
        if self.can_interrupt_for_action(new_action) {
            self.action_state = ActionState::new(new_action);
            new_action
        } else {
            // Continue current action if we can't interrupt
            self.action_state.current_action
        }
    }
    
    /// Check if current action should continue
    fn should_continue_current_action(&self, environment: &Environment) -> bool {
        // Don't continue if action is complete
        if !self.action_state.should_continue() {
            return false;
        }
        
        // Always continue high priority actions unless they're complete
        if self.action_state.priority >= ActionPriority::High {
            return true;
        }
        
        // Continue medium priority actions unless there's urgent need to change
        if self.action_state.priority == ActionPriority::Medium {
            // Stop if player is about to attack and we're vulnerable
            if environment.player_state.check(PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::BACK_KICKING) 
               && environment.distance < 200.0 {
                return false;
            }
            
            // Stop if we're doing something ineffective
            match self.action_state.current_action {
                Action::MoveForward => {
                    // Stop if we're too close or player moved away
                    environment.distance > 80.0 && environment.distance < 500.0
                }
                Action::MoveBackward => {
                    // Stop if we're far enough or player is vulnerable
                    environment.distance < 300.0 && !environment.is_player_vulnerable
                }
                _ => true, // Continue other medium priority actions
            }
        } else {
            // Low priority actions can be interrupted more easily
            // Continue only if it makes sense
            match self.action_state.current_action {
                Action::MoveForward => {
                    // Continue if we need to get closer
                    environment.distance > 150.0 && !environment.is_player_vulnerable
                }
                Action::MoveBackward => {
                    // Continue if we need distance
                    environment.distance < 200.0 && environment.player_state.check(PlayerState::WALKING)
                }
                Action::RollForward => {
                    // Continue rolling forward - it's a committed action
                    true
                }
                Action::RollBackward => {
                    // Continue rolling backward - it's a committed action
                    true
                }
                Action::Bend => {
                    // Continue bending unless immediate threat
                    !environment.player_state.check(PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::BACK_KICKING)
                }
                _ => false, // Don't continue other low priority actions
            }
        }
    }
    
    /// Check if current action can be interrupted for new action
    fn can_interrupt_for_action(&self, new_action: Action) -> bool {
        let new_priority = match new_action {
            Action::MoveForward | Action::MoveBackward => ActionPriority::Low,
            Action::Bend => ActionPriority::Low,
            Action::RollForward | Action::RollBackward => ActionPriority::Medium,
            Action::JumpUP | Action::JumpForward | Action::JumpBackward => ActionPriority::Medium,
            Action::JumpKick => ActionPriority::Medium, // Can be added to existing jump
            Action::Kick | Action::BackKick | Action::Punch => ActionPriority::Medium,
            Action::RangedAttack => ActionPriority::Medium,
            Action::Skill => ActionPriority::High,
            Action::None => ActionPriority::Low,
        };
        
        self.action_state.can_be_interrupted_by(new_priority)
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
                // Close range kick - jump up to avoid
                return Action::JumpUP;
            } else if environment.distance < 250.0 {
                // Medium range kick - roll backward for evasion
                return Action::RollBackward;
            } else {
                return Action::MoveBackward;
            }
        }
        if environment.player_state.check(PlayerState::PUNCHING) {
            if environment.distance < 120.0 {
                return Action::Bend;
            } else if environment.distance < 200.0 {
                // Roll backward to avoid punch and create counter-attack opportunity
                return Action::RollBackward;
            } else {
                return Action::BackKick;
            }
        }
        if environment.player_state.check(PlayerState::BACK_KICKING) {
            if environment.distance < 180.0 {
                // Roll forward to get behind player after their back kick
                return Action::RollForward;
            } else {
                return Action::MoveForward;
            }
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
            } else if environment.distance > 200.0 {
                // Use roll forward for quick approach to vulnerable player
                Action::RollForward
            } else {
                // Jump forward first, then kick on next frame
                if environment.agent_state.check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD) {
                    Action::JumpKick  // Add kick to existing jump
                } else {
                    Action::JumpForward  // Jump first
                }
            }
        } else if environment.distance < 500.0 {
            // Medium-long range - roll forward to close distance quickly for punishment
            Action::RollForward
        } else {
            Action::MoveForward
        }
    }
    
    /// Select aggressive action based on distance and resources
    fn select_offensive_action(&self, environment: &Environment) -> Action {
        // Improved offensive strategy with better resource management and rolling actions
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
            // More aggressive approach at medium distance with rolling for positioning
            if environment.player_state.check(PlayerState::WALKING) {
                let rand = rand();
                if rand < 0.3 {
                    // Use roll forward for faster, evasive approach
                    return Action::RollForward;
                } else if environment.agent_state.check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD) {
                    return Action::JumpKick;  // Add kick to existing jump
                } else {
                    return Action::JumpForward;  // Jump first
                }
            } else if environment.is_player_vulnerable {
                // Quick roll forward to punish vulnerable player
                return Action::RollForward;
            } else {
                return Action::MoveForward;
            }
        } else if environment.distance < 500.0 {
            if environment.player_state.check(PlayerState::BEND_DOWN) || 
               environment.player_state.is_idle() {
                let rand = rand();
                if rand < 0.4 {
                    // Roll forward for surprise attack on idle/bending player
                    return Action::RollForward;
                } else if environment.agent_state.check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD) {
                    return Action::JumpKick;  // Add kick to existing jump
                } else {
                    return Action::JumpForward;  // Jump first
                }
            } else if environment.player_state.check(PlayerState::WALKING) && 
                     environment.agent_fire_charge == FIRE_CHARGE_MAX {
                return Action::RangedAttack;
            } else if environment.player_state.check(PlayerState::JUMP_FORWARD) {
                return Action::BackKick;
            } else {
                let rand = rand();
                if rand < 0.25 {
                    // Occasional roll forward for positioning
                    return Action::RollForward;
                } else {
                    return Action::MoveForward;
                }
            }
        } else if environment.distance < 800.0 {
            let rand = rand();
            if rand < 0.6 && environment.agent_fire_charge == FIRE_CHARGE_MAX {
                return Action::RangedAttack;
            } else if rand < 0.2 {
                // Long range roll forward for closing distance quickly
                return Action::RollForward;
            } else if rand < 0.8 {
                return Action::MoveForward;
            } else {
                return Action::JumpForward;
            }
        } else {
            let rand = rand();
            if rand < 0.15 {
                // Use roll forward to close extreme distances quickly
                return Action::RollForward;
            } else if rand < 0.3 {
                return Action::JumpForward;
            } else {
                return Action::MoveForward;
            }
        }
    }
    
    /// Select defensive action to avoid damage and maintain distance
    fn select_defensive_action(&self, environment: &Environment) -> Action {
        // Enhanced defensive strategy with rolling evasion
        if environment.distance < 80.0 {
            let rand = rand();
            if environment.player_state.check(PlayerState::KICKING | PlayerState::PUNCHING | PlayerState::BACK_KICKING) {
                // Player is attacking at close range - use roll backward for quick escape
                if rand < 0.7 {
                    return Action::RollBackward;
                } else {
                    return Action::JumpBackward;
                }
            } else if rand < 0.4 {
                // Use roll backward for safer retreat
                return Action::RollBackward;
            } else if rand < 0.8 {
                return Action::JumpBackward;
            } else {
                return Action::MoveBackward;
            }
        } else if environment.distance < 200.0 {
            if environment.player_state.check(PlayerState::WALKING) {
                let rand = rand();
                if rand < 0.3 {
                    // Roll backward to maintain distance from approaching player
                    return Action::RollBackward;
                } else {
                    return Action::MoveBackward;
                }
            } else if environment.player_state.check(PlayerState::JUMP_FORWARD) {
                // Player jumping toward us - roll backward to avoid
                return Action::RollBackward;
            } else {
                let rand = rand();
                if rand < 0.3 && environment.agent_fire_charge == FIRE_CHARGE_MAX {
                    return Action::RangedAttack;
                } else if rand < 0.2 {
                    // Occasional defensive roll
                    return Action::RollBackward;
                } else {
                    return Action::MoveBackward;
                }
            }
        } else if environment.distance < 400.0 {
            // Maintain distance and look for opportunities
            if environment.player_state.check(PlayerState::JUMP_FORWARD) {
                return Action::BackKick;
            } else if environment.player_state.check(PlayerState::KICKING | PlayerState::PUNCHING) {
                // Player attacking but not in range - use roll backward for repositioning
                return Action::RollBackward;
            } else if environment.agent_fire_charge == FIRE_CHARGE_MAX && 
                     environment.player_state.check(PlayerState::WALKING) {
                return Action::RangedAttack;
            } else {
                let rand = rand();
                if rand < 0.15 {
                    // Occasional roll backward for positioning
                    return Action::RollBackward;
                } else {
                    return Action::MoveBackward;
                }
            }
        } else {
            // Safe distance - prepare for counter-attack or maintain position
            let rand = rand();
            if rand < 0.4 {
                return Action::None;
            } else if rand < 0.1 {
                // Very occasional roll backward at safe distance for unpredictability
                return Action::RollBackward;
            } else {
                return Action::Bend;
            }
        }
    }
    
    /// Select neutral action for positioning and resource management
    fn select_neutral_action(&self, environment: &Environment) -> Action {
        let rand = rand();
        
        // Add some rolling for positioning and unpredictability
        if environment.distance > 400.0 && environment.distance < 600.0 {
            if rand < 0.15 {
                // Occasional roll forward for positioning at medium-long range
                return Action::RollForward;
            }
        } else if environment.distance > 150.0 && environment.distance < 300.0 {
            if rand < 0.1 {
                // Occasional roll backward for spacing at medium range
                return Action::RollBackward;
            }
        }
        
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
    character_textures: Res<CharacterTextures>,
    mut player_query: Query<(&mut Player, &PlayerID, &mut Sprite, &mut Transform)>,
) {
    // Skip if multiplayer
    if game_config.mode == GameMode::MultiPlayer {
        return;
    }
    agent.timer.tick(time.delta());
    if agent.timer.finished() {
        // update facing
        update_facing(&mut player_query);

        agent.count += 1;
        let mut environment = Environment::default();
        
        // Collect environment data
        if let Some((player, _, _, transform)) = player_query.iter().find(|(_, id, _, _)| id.0 == 0) {
            environment.player_health = player.health as f32
                / CHARACTER_PROFILES[player.character_id as usize].health as f32;
            environment.player_state = player.state;
            environment.distance = transform.translation.x;
            environment.player_energy = player.energy;
            environment.player_fire_charge = player.fire_charge;
        }
        if let Some((player, _, _, transform)) = player_query.iter().find(|(_, id, _, _)| id.0 == 1) {
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
        
        // Update policy every 2 seconds (was every 2 seconds before)
        if agent.count >= AGENT_FREQUENCY as u8 * 2 {
            agent.count = 0;
            agent.select_policy(&environment);
        }
        
        // Select action with continuity
        let action = agent.select_action(&environment);
        
        // Execute action on agent
        if let Some((mut player, player_id, mut sprite, _)) = player_query.iter_mut().find(|(_, id, _, _)| id.0 == 1) {
            // Reset engagement timer when taking action
            if action != Action::None {
                agent.engagement_timer = 0;
            }
            
            // Handle action execution with better state management
            execute_agent_action(action, &mut player, player_id, &mut sprite, &character_textures, &mut commands, &mut fighting);
        }
    }
}

/// Execute the selected action on the agent player
fn execute_agent_action(
    action: Action,
    player: &mut Player,
    player_id: &PlayerID,
    sprite: &mut Sprite,
    character_textures: &CharacterTextures,
    commands: &mut Commands,
    fighting: &mut Fighting,
) {
    // Only reset to idle state if we're changing to a non-movement action
    if action != Action::MoveForward && action != Action::MoveBackward && player.state.check(PlayerState::WALKING) {
        // Reset to idle for new actions
        player.state &= !PlayerState::WALKING;
        player.velocity = Vec2::ZERO;
        if player.state.is_idle() {
            sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
            sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
            player.animation_frame_max = FRAMES_IDLE;
            player.pose.set(IDLE_POSE1);
            player.set_animation(IDLE_POSE2, 0, 15);
        }
    }
    
    // Only remove BEND_DOWN state when not bending
    if action != Action::Bend && player.state.check(PlayerState::BEND_DOWN) && player.animation.phase != 2 {
        // player is bending down
        // then stop bending down
        player.set_animation(BEND_DOWN_POSE1, 2, 23);
        return;
    }
    
    match action {
        Action::MoveForward => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].walk.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_WALK;
                player.state |= PlayerState::WALKING;
                player.pose.set(WALKING_POSE1);
                player.set_animation(WALKING_POSE2, 1, 15);
            }
            if player.pose.facing {
                player.state |= PlayerState::DIRECTION;
            } else {
                player.state &= !PlayerState::DIRECTION;
            }
        }
        Action::MoveBackward => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].walk.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = FRAMES_WALK - 1);
                player.animation_frame_max = FRAMES_WALK;
                player.state |= PlayerState::WALKING;
                player.pose.set(WALKING_POSE1);
                player.set_animation(WALKING_POSE2, 1, 15);
            }
            if player.pose.facing {
                player.state &= !PlayerState::DIRECTION;
            } else {
                player.state |= PlayerState::DIRECTION;
            }
        }
        Action::RollForward => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_ROLL;
                player.state |= PlayerState::ROLL_FORWARD;
                player.pose.set(ROLL_FORWARD_POSE1);
                player.set_animation(ROLL_FORWARD_POSE2, 0, 11);
                if player.pose.facing {
                    player.state |= PlayerState::DIRECTION;
                } else {
                    player.state &= !PlayerState::DIRECTION;
                }
                let x_vel = if player.state.is_forward() { 1.0 } else { -1.0 }
                    * CHARACTER_PROFILES[player.character_id as usize].agility * 2.0;
                player.velocity = Vec2::new(x_vel, 0.0);
            }
        }
        Action::RollBackward => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].roll.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = FRAMES_ROLL - 1);
                player.animation_frame_max = FRAMES_ROLL;
                player.state |= PlayerState::ROLL_BACK;
                player.pose.set(ROLL_BACK_POSE1);
                player.set_animation(ROLL_BACK_POSE2, 0, 4);
                if player.pose.facing {
                    player.state &= !PlayerState::DIRECTION;
                } else {
                    player.state |= PlayerState::DIRECTION;
                }
                let x_vel = if player.state.is_forward() { -1.0 } else { 1.0 }
                    * CHARACTER_PROFILES[player.character_id as usize].agility * 2.0;
                player.velocity = Vec2::new(x_vel, 0.0);
            }
        }
        Action::JumpUP => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_JUMP;
                player.state |= PlayerState::JUMP_UP;
                player.pose.set(JUMP_POSE1);
                player.set_animation(JUMP_POSE2, 0, 11);
                player.velocity = Vec2::new(0.0, 12.0);
                player.energy += 1;
            } else if !player.state.check(
                PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD | PlayerState::JUMP_BACKWARD,
            ) && player.state.check(PlayerState::WALKING) {
                if player.state.check(PlayerState::DIRECTION) {
                    sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_JUMP;
                    player.state |= PlayerState::JUMP_FORWARD;
                    player.pose.set(JUMP_POSE1);
                    player.set_animation(JUMP_POSE2, 0, 11);
                    let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                    player.velocity = Vec2::new(x_vel, 12.0);
                } else {
                    sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_JUMP;
                    player.state |= PlayerState::JUMP_BACKWARD;
                    player.pose.set(JUMP_POSE1);
                    player.set_animation(JUMP_POSE2, 0, 11);
                    let x_vel = CHARACTER_PROFILES[player.character_id as usize].agility;
                    player.velocity = Vec2::new(-x_vel, 12.0);
                }
                player.energy += 1;
            }
        }
        Action::JumpForward => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_JUMP;
                if player.pose.facing {
                    player.state |= PlayerState::DIRECTION;
                } else {
                    player.state &= !PlayerState::DIRECTION;
                }
                player.state |= PlayerState::JUMP_FORWARD;
                player.pose.set(JUMP_POSE1);
                player.set_animation(JUMP_POSE2, 0, 11);
                player.velocity = Vec2::ZERO;
                player.energy += 1;
            }
        }
        Action::JumpBackward => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].jump.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_JUMP;
                if !player.pose.facing {
                    player.state &= !PlayerState::DIRECTION;
                } else {
                    player.state |= PlayerState::DIRECTION;
                }
                player.state |= PlayerState::JUMP_BACKWARD;
                player.pose.set(JUMP_POSE1);
                player.set_animation(JUMP_POSE2, 0, 11);
                player.velocity = Vec2::ZERO;
                player.energy += 1;
            }
        }
        Action::JumpKick => {
            // JumpKick can only be executed when already jumping
            if player.state.check(PlayerState::JUMP_UP | PlayerState::JUMP_FORWARD) 
               && !player.state.check(PlayerState::KICKING) {
                // Add kicking to existing jump state
                player.state |= PlayerState::KICKING;
                player.energy += 2;
            }
            // If not jumping, this action has no effect (should not happen with proper logic)
        }
        Action::Bend => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].bend_down.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_BEND_DOWN;
                player.state |= PlayerState::BEND_DOWN;
                player.pose.set(BEND_DOWN_POSE1);
                player.set_animation(BEND_DOWN_POSE2, 0, 27);
            }
        }
        Action::Kick => {
            if player.state.is_idle() {
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
        Action::BackKick => {
            if player.state.is_idle() {
                sprite.image = character_textures.textures[player.character_id as usize].back_kick.clone();
                sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                player.animation_frame_max = FRAMES_BACK_KICK;
                player.state |= PlayerState::BACK_KICKING;
                player.pose.set(BACK_KICK_POSE1);
                player.set_animation(BACK_KICK_POSE2, 0, 11);
                player.energy += 2;
            }
        }
        Action::RangedAttack => {
            if player.state.is_idle() && player.fire_charge == FIRE_CHARGE_MAX {
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
        Action::Punch => {
            if player.state.is_idle() {
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
        Action::Skill => {
            if player.state.is_idle() && player.energy == ENERGY_MAX {
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
                player.state &= !PlayerState::WALKING;
                if player.state.is_idle() {
                    sprite.image = character_textures.textures[player.character_id as usize].idle.clone();
                    sprite.texture_atlas.as_mut().map(|atlas| atlas.index = 0);
                    player.animation_frame_max = FRAMES_IDLE;
                    player.pose.set(IDLE_POSE1);
                    player.set_animation(IDLE_POSE2, 0, 15);
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
