use crate::{
    ingame::GameState, AppState, GameConfig, GameMode, SoundEffect, Score, BGM, PATH_SOUND_PREFIX, DEFAULT_FONT_SIZE, PATH_BOLD_FONT, PATH_BOLD_JP_FONT, PATH_EXTRA_BOLD_FONT,
    PATH_IMAGE_PREFIX, TITLE_FONT_SIZE,
};
use bevy::prelude::*;

#[derive(Component)]
struct ShowResult;

/// Structure to record play count
#[derive(Debug, Default)]
struct PlayCount {
    single_mode: u32,
    multi_mode: u32,
    time_slot: String,
}

impl PlayCount {
    const FILE_PATH: &'static str = "statistics.txt";
    
    /// Get current time slot (e.g., "2025-10-29 14:00-15:00")
    fn get_current_time_slot() -> String {
        use std::time::SystemTime;
        
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = duration_since_epoch.as_secs();
        
        // Get UTC time (in seconds)
        let total_seconds = timestamp;
        let total_minutes = total_seconds / 60;
        let total_hours = total_minutes / 60;
        let total_days = total_hours / 24;
        
        // Calculate date from days since January 1, 1970
        let year = 1970 + (total_days / 365) as i32;
        let day_of_year = (total_days % 365) as u32;
        
        // Simple month/day calculation (ignoring leap years)
        let (month, day) = Self::day_of_year_to_month_day(day_of_year);
        
        // JST (+9 hours)
        let hour_utc = (total_hours % 24) as u32;
        let hour_jst = (hour_utc + 9) % 24;
        
        format!("{:04}-{:02}-{:02} {:02}:00-{:02}:00", 
                year, month, day, hour_jst, (hour_jst + 1) % 24)
    }
    
    /// Convert day of year to month and day
    fn day_of_year_to_month_day(day_of_year: u32) -> (u32, u32) {
        let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut remaining_days = day_of_year;
        
        for (month_idx, &days) in days_in_month.iter().enumerate() {
            if remaining_days < days {
                return ((month_idx + 1) as u32, remaining_days + 1);
            }
            remaining_days -= days;
        }
        (12, 31) // Fallback
    }
    
    /// Load from statistics.txt (read only the latest section)
    fn load() -> Self {
        let path = std::path::PathBuf::from(Self::FILE_PATH);

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let mut single = 0;
                let mut multi = 0;
                let mut time_slot = String::new();
                
                // Find the last section by splitting on separators
                let sections: Vec<&str> = content.split("========================================").collect();
                
                // Get the last section (most recent)
                if let Some(last_section) = sections.last() {
                    for line in last_section.lines() {
                        let line = line.trim();
                        
                        if line.starts_with("Time Slot:") {
                            if let Some(time_str) = line.split("Time Slot:").nth(1) {
                                time_slot = time_str.trim().to_string();
                            }
                        } else if line.starts_with("Single Mode:") {
                            if let Some(count_str) = line.split(':').nth(1) {
                                single = count_str.trim().parse().unwrap_or(0);
                            }
                        } else if line.starts_with("Multi Mode:") {
                            if let Some(count_str) = line.split(':').nth(1) {
                                multi = count_str.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                }
                
                PlayCount {
                    single_mode: single,
                    multi_mode: multi,
                    time_slot,
                }
            }
            Err(_) => PlayCount::default(),
        }
    }

    /// save the play count to statistics.txt
    fn save(&self, mode: GameMode) -> std::io::Result<()> {
        let path = std::path::PathBuf::from(Self::FILE_PATH);
        let current_time_slot = Self::get_current_time_slot();
        
        // Read existing content
        let existing_content = std::fs::read_to_string(&path).unwrap_or_default();
        
        // If time slot has changed
        if self.time_slot != current_time_slot {
            let mut new_content = existing_content;
            
            // Add separator if there is existing content
            if !new_content.is_empty() {
                new_content.push_str("\n========================================\n\n");
            }
            
            // Create new section
            new_content.push_str(&format!("Time Slot: {}\n", current_time_slot));
            
            // New count (initialize according to mode)
            let (single, multi) = match mode {
                GameMode::SinglePlayer => (1, 0),
                GameMode::MultiPlayer => (0, 1),
            };
            
            new_content.push_str(&format!("Single Mode: {}\n", single));
            new_content.push_str(&format!("Multi Mode: {}\n", multi));
            
            std::fs::write(&path, new_content)?;
        } else {
            // If same time slot, update the latest section
            let mut lines: Vec<String> = existing_content.lines().map(|s| s.to_string()).collect();
            
            // Update Single Mode and Multi Mode in the last section
            for i in (0..lines.len()).rev() {
                if lines[i].starts_with("Single Mode:") {
                    lines[i] = format!("Single Mode: {}", self.single_mode);
                } else if lines[i].starts_with("Multi Mode:") {
                    lines[i] = format!("Multi Mode: {}", self.multi_mode);
                } else if lines[i] == "========================================" {
                    // Reached the separator, stop updating
                    break;
                }
            }
            
            let new_content = lines.join("\n") + "\n";
            std::fs::write(&path, new_content)?;
        }
        
        Ok(())
    }
    
    /// Increment the play count based on game mode
    fn increment(&mut self, mode: GameMode) {
        match mode {
            GameMode::SinglePlayer => self.single_mode += 1,
            GameMode::MultiPlayer => self.multi_mode += 1,
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gamestate: Res<GameState>,
    game_config: Res<GameConfig>,
    mut score: ResMut<Score>,
    audio_query: Query<Entity, With<BGM>>
) {
    info!("setup");

    // Record play count
    let mut play_count = PlayCount::load();
    play_count.increment(game_config.mode);
    if let Err(e) = play_count.save(game_config.mode) {
        error!("Failed to save play count: {}", e);
    } else {
        info!("Play count saved: Single Mode: {}, Multi Mode: {}", 
              play_count.single_mode, play_count.multi_mode);
    }

    for entity in audio_query.iter() {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        AudioPlayer::new(asset_server.load(format!("{}Result.ogg", PATH_SOUND_PREFIX))),
        PlaybackSettings::LOOP,
        BGM(false),
    ));

    commands
        .spawn((
            ImageNode::new(
                asset_server.load(format!("{}background_mainmenu.png", PATH_IMAGE_PREFIX)),
            ),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ShowResult,
        ))
        .with_children(|spawner| {
            spawner
                .spawn((
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Percent(90.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(20.0)),
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                ))
                .with_children(|spawner| {
                    spawner.spawn((
                        Text::new("対戦結果"),
                        TextFont {
                            font: asset_server.load(PATH_EXTRA_BOLD_FONT),
                            font_size: TITLE_FONT_SIZE,
                            ..Default::default()
                        },
                        TextColor(Color::BLACK),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                    spawner
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(80.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceEvenly,
                            justify_items: JustifyItems::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|spawner| {
                            create_round_result(spawner, &asset_server, 1, gamestate.winners[0]);
                            create_round_result(spawner, &asset_server, 2, gamestate.winners[1]);
                            create_round_result(spawner, &asset_server, 3, gamestate.winners[2]);
                            create_total_result(spawner, &asset_server, gamestate.get_winner(), score.0);
                        });
                    spawner
                        .spawn((
                            Text::new("景品を受け取って、次の人に交代してね！"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_JP_FONT),
                                font_size: DEFAULT_FONT_SIZE,
                                ..Default::default()
                            },
                            TextColor(Color::BLACK),
                            TextLayout::new_with_justify(JustifyText::Center),
                        ));
                    spawner
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(30.0),
                                height: Val::Percent(10.0),
                                justify_self: JustifySelf::Center,
                                align_self: AlignSelf::Center,
                                #[cfg(not(feature="phone"))]
                                border: UiRect::all(Val::Px(10.0)),
                                #[cfg(feature="phone")]
                                border: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(10.0)),
                            BorderColor(Color::BLACK),
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                        ))
                        .with_child((
                            Text::new("Back to Main Menu"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_FONT),
                                font_size: DEFAULT_FONT_SIZE,
                                ..Default::default()
                            },
                            TextColor(Color::BLACK),
                            TextLayout::new_with_justify(JustifyText::Center),
                            Node {
                                width: Val::Percent(100.0),
                                ..default()
                            },
                        ));
                });
        });
    score.0 = 0;
}

fn create_round_result(
    spawner: &mut ChildSpawnerCommands,
    asset_server: &Res<AssetServer>,
    round: u8,
    winner_id: u8,
) {
    spawner
        .spawn((
            Node {
                width: Val::Percent(90.0),
                height: Val::Percent(20.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderRadius::all(Val::Px(10.0)),
            BorderColor(Color::BLACK),
        ))
        .with_children(|spawner| {
            spawner.spawn((
                Text::new(format!("Round {} Result", round)),
                TextFont {
                    font: asset_server.load(PATH_BOLD_FONT),
                    font_size: DEFAULT_FONT_SIZE,
                    ..Default::default()
                },
                TextColor(Color::BLACK),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
            ));
            if winner_id == 0 {
                spawner.spawn((
                    Text::new("DRAW"),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: DEFAULT_FONT_SIZE,
                        ..Default::default()
                    },
                    TextColor(Color::BLACK),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Node {
                        width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            } else {
                spawner.spawn((
                    Text::new(format!("Player {} WIN!", winner_id)),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: DEFAULT_FONT_SIZE,
                        ..Default::default()
                    },
                    if winner_id == 1 {
                        TextColor(Color::srgba(1.0, 0.0, 0.0, 0.8))
                    } else {
                        TextColor(Color::srgba(0.0, 0.0, 1.0, 0.8))
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    Node {
                        width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }
        });
}

fn create_total_result(spawner: &mut ChildSpawnerCommands, asset_server: &Res<AssetServer>, winner_id: u8, score: u32) {
    spawner
        .spawn((
            Node {
                width: Val::Percent(90.0),
                height: Val::Percent(20.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BorderRadius::all(Val::Px(10.0)),
            BorderColor(Color::srgba(1.0, 0.0, 0.0, 0.8)),
        ))
        .with_children(|spawner| {
            spawner.spawn((
                Text::new("Total Result"),
                TextFont {
                    font: asset_server.load(PATH_BOLD_FONT),
                    font_size: DEFAULT_FONT_SIZE,
                    ..Default::default()
                },
                TextColor(Color::BLACK),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
            ));
            if winner_id == 0 {
                spawner.spawn((
                    Text::new("DRAW"),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: DEFAULT_FONT_SIZE,
                        ..Default::default()
                    },
                    TextColor(Color::BLACK),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Node {
                        width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            } else {
                spawner.spawn((
                    Text::new(format!("Player {} WIN! Your Score: {}", winner_id, score)),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: DEFAULT_FONT_SIZE,
                        ..Default::default()
                    },
                    TextColor(Color::srgba(1.0, 0.0, 0.0, 0.8)),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Node {
                        width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }
        });
}

fn controller_input(
    mut next_state: ResMut<NextState<AppState>>,
    gamepads: Query<&Gamepad>,
) {
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::South) {
            next_state.set(AppState::Mainmenu);
        }
    }
}

fn check_exit_button(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    sound_query: Query<Entity, With<SoundEffect>>,
) {
    for interaction in query.iter() {
        match *interaction {
            Interaction::Pressed => {
                for sound in sound_query.iter() {
                    commands.entity(sound).despawn();
                }
                commands.spawn((
                    AudioPlayer::new(asset_server.load(format!(
                        "{}button_click.ogg",
                        PATH_SOUND_PREFIX,
                    ))),
                    SoundEffect,
                ));
                next_state.set(AppState::Mainmenu);
            }
            _ => {}
        }
    }
}

fn exit(mut commands: Commands, query: Query<Entity, With<ShowResult>>) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub struct ResultPlugin;

impl Plugin for ResultPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Result), setup)
            .add_systems(OnExit(AppState::Result), exit)
            .add_systems(Update, check_exit_button.run_if(in_state(AppState::Result)))
            .add_systems(Update, controller_input.run_if(in_state(AppState::Result)));
    }
}
