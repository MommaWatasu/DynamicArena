use bevy::prelude::*;

use crate::{
    character_def::*, AppState, GameConfig, GameMode, SoundEffect, PATH_SOUND_PREFIX, DEFAULT_FONT_SIZE, PATH_BOLD_FONT, PATH_BOLD_JP_FONT,
    PATH_EXTRA_BOLD_JP_FONT, PATH_IMAGE_PREFIX, TITLE_FONT_SIZE,
};

#[derive(Component)]
struct ChooseCharacter;

#[derive(Component)]
struct CharacterID(isize);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut config: ResMut<GameConfig>,
) {
    info!("setup");
    commands
        .spawn((
            ImageNode::new(
                asset_server.load(format!("{}background_mainmenu.png", PATH_IMAGE_PREFIX)),
            ),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChooseCharacter,
        ))
        .with_children(|spawner| {
            spawner
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(10.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|spawner| {
                    spawner
                        .spawn((
                            Button,
                            Node {
                                justify_self: JustifySelf::Start,
                                align_self: AlignSelf::Start,
                                #[cfg(not(feature="phone"))]
                                border: UiRect::all(Val::Px(5.0)),
                                #[cfg(feature="phone")]
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BorderRadius::MAX,
                            BorderColor(Color::BLACK),
                        ))
                        .with_child((
                            Text::new("<Back"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_FONT),
                                font_size: DEFAULT_FONT_SIZE,
                                ..Default::default()
                            },
                            TextLayout::new_with_justify(JustifyText::Center),
                            TextColor(Color::BLACK),
                        ));
                    spawner
                        .spawn((
                            Button,
                            Node {
                                justify_self: JustifySelf::End,
                                align_self: AlignSelf::Start,
                                #[cfg(not(feature="phone"))]
                                border: UiRect::all(Val::Px(5.0)),
                                #[cfg(feature="phone")]
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BorderRadius::MAX,
                            BorderColor(Color::BLACK),
                        ))
                        .with_child((
                            Text::new("Next>"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_FONT),
                                font_size: DEFAULT_FONT_SIZE,
                                ..Default::default()
                            },
                            TextLayout::new_with_justify(JustifyText::Center),
                            TextColor(Color::BLACK),
                        ));
                });
            if config.mode == GameMode::SinglePlayer {
                config.characters_id = [0, choose_rand_character(0)];
            } else {
                config.characters_id = [0, 2];
            }
            spawner
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    ..default()
                })
                .with_children(|spawner| {
                    spawner.spawn((
                        Text::new("キャラクターを選んでください"),
                        TextFont {
                            font: asset_server.load(PATH_EXTRA_BOLD_JP_FONT),
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
                        .spawn((
                            Node {
                                width: Val::Percent(90.0),
                                height: Val::Percent(90.0),
                                flex_direction: FlexDirection::Row,
                                align_self: AlignSelf::Center,
                                justify_self: JustifySelf::Center,
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,
                                justify_content: JustifyContent::SpaceEvenly,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                            BorderRadius::all(Val::Px(20.0)),
                        ))
                        .with_children(|spawner| {
                            for i in 0..3 {
                                create_character_box(spawner, &asset_server, &mut config, i as isize);
                            }
                        });
                });
        });
}

// TODO: now the character selection is not available for wasm target
fn create_character_box(
    spawner: &mut ChildSpawnerCommands,
    asset_server: &Res<AssetServer>,
    config: &mut GameConfig,
    character_id: isize,
) {
    let profile = &CHARACTER_PROFILES[character_id as usize];
    spawner
        .spawn(
            Node {
                width: Val::Percent(30.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        )
        .with_children(|spawner| {
            spawner.spawn(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(10.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                }
            )
            .with_child((
                if character_id == 0 {
                    Text::new("Player 1")
                } else {
                    Text::new("")
                },
                CharacterID(character_id),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont {
                    font: asset_server.load(PATH_BOLD_FONT),
                    font_size: DEFAULT_FONT_SIZE,
                    ..Default::default()
                },
                TextColor(Color::srgba(10.0, 0.0, 0.0, 0.8)),
            ));
            spawner.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(85.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BorderRadius::all(Val::Px(20.0)),
                BackgroundColor(Color::srgba(0.6, 0.8, 0.9, 0.8)),
            ))
            .with_children(|spawner| {
                spawner.spawn((
                    Text::new(profile.name),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        #[cfg(not(feature="phone"))]
                        font_size: 40.0,
                        #[cfg(feature="phone")]
                        font_size: 15.0,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(Color::BLACK),
                ));
                spawner.spawn((
                    Text::new(profile.description),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_JP_FONT),
                        #[cfg(not(feature="phone"))]
                        font_size: 30.0,
                        #[cfg(feature="phone")]
                        font_size: 10.0,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Left),
                    TextColor(Color::BLACK),
                ));
                spawner.spawn((
                    Text::new(format!(
                        "<スキル> {}\n{}",
                        profile.skill_name, profile.skill_description
                    )),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_JP_FONT),
                        #[cfg(not(feature="phone"))]
                        font_size: 30.0,
                        #[cfg(feature="phone")]
                        font_size: 10.0,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Left),
                    TextColor(Color::BLACK),
                ));
                spawner.spawn((ImageNode::new(asset_server.load(format!(
                    "{}character_{}_chart.png",
                    PATH_IMAGE_PREFIX, character_id
                ))),));
            });
            if config.mode == GameMode::MultiPlayer {
                spawner.spawn(
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(10.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    }
                )
                .with_child((
                    if character_id == 2 {
                        Text::new("Player 2")
                    } else {
                        Text::new("")
                    },
                    CharacterID(character_id),
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: DEFAULT_FONT_SIZE,
                        ..Default::default()
                    },
                    TextColor(Color::srgba(0.0, 0.0, 10.0, 0.8)),
                ));
            }
        });
}

#[cfg(not(target_arch = "wasm32"))]
fn choose_rand_character(id: isize) -> isize {
    use rand::seq::IteratorRandom;
    let mut available_nums = vec![0, 1, 2];
    available_nums.retain(|&x| x != id);
    available_nums
        .iter()
        .choose(&mut rand::rng())
        .unwrap()
        .clone()
}

#[cfg(target_arch = "wasm32")]
fn choose_rand_character(id: isize) -> isize {
    let mut available_nums = vec![0, 1, 2];
    available_nums.retain(|&x| x != id);
    let random_index = (web_sys::js_sys::Math::random() * 2.0).floor() as usize;
    available_nums[random_index]
}

#[cfg(not(target_arch = "wasm32"))]
fn controller_input(
    mut next_state: ResMut<NextState<AppState>>,
    mut text_query: Query<(&mut Text, &TextColor, &CharacterID)>,
    mut config: ResMut<GameConfig>,
    gamepads: Query<(&Gamepad, Entity)>,
) {
    #[allow(unused_assignments)]
    let mut id = 0;
    for (gamepad, entity) in gamepads.iter() {
        if config.gamepads[0] == entity {
            id = 0;
        } else {
            id = 1;
        }

        let text_player_color = if id == 0 {
            Color::srgba(0.0, 0.0, 10.0, 0.8)
        } else {
            Color::srgba(10.0, 0.0, 0.0, 0.8)
        };

        let mut character_id = 0;
        for (text, _, character_id_text) in text_query.iter_mut() {
            if text.0 == format!("Player {}", id+1) {
                character_id = character_id_text.0;
            }
        }
        if gamepad.just_pressed(GamepadButton::DPadRight) {
            if character_id != 2 {
                config.characters_id[id] = character_id + 1;
                if config.mode == GameMode::SinglePlayer {
                    config.characters_id[1-id] = choose_rand_character(config.characters_id[id]);
                }
                for (mut text, text_color, character_id_text) in text_query.iter_mut() {
                    if text_color.0 == text_player_color {
                        continue;
                    }
                    if text.0 == format!("Player {}", id+1) {
                        text.0 = "".to_string();
                    } else if character_id_text.0 == config.characters_id[id] {
                        text.0 = format!("Player {}", id+1);
                    }
                }
            }
        } else if gamepad.just_pressed(GamepadButton::DPadLeft) {
            if character_id != 0 {
                config.characters_id[id] = character_id - 1;
                if config.mode == GameMode::SinglePlayer {
                    config.characters_id[1-id] = choose_rand_character(config.characters_id[id]);
                }
                for (mut text, text_color, character_id_text) in text_query.iter_mut() {
                    if text_color.0 == text_player_color {
                        continue;
                    }
                    if text.0 == format!("Player {}", id+1) {
                        text.0 = "".to_string();
                    } else if character_id_text.0 == config.characters_id[id] {
                        text.0 = format!("Player {}", id+1);
                    }
                }
            }
        }
        if gamepad.just_pressed(GamepadButton::East) {
            next_state.set(AppState::ConnectController);
        } else if gamepad.just_pressed(GamepadButton::West) {
            next_state.set(AppState::Confirm);
        }
    }
}

#[cfg(not(feature="phone"))]
fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<GameConfig>,
    mut text_query: Query<(&mut Text, &TextColor, &CharacterID)>,
) {
    /*
    if config.mode == GameMode::MultiPlayer {
        // keyboard is invaild in multi player mode
        return;
    }
    */
    
    let mut character_id = 0;
    for (text, _, character_id_text) in text_query.iter_mut() {
        if config.mode == GameMode::MultiPlayer {
            if text.0 == "Player 2".to_string() {
                character_id = character_id_text.0;
            }
        } else {
            if text.0 == "Player 1".to_string() {
                character_id = character_id_text.0;
            }
        }
    }
    if keys.just_pressed(KeyCode::KeyD) {
        if character_id != 2 {
            config.characters_id[0] = character_id + 1;
            config.characters_id[1] = choose_rand_character(config.characters_id[0]);
            for (mut text, text_color, character_id_text) in text_query.iter_mut() {
                if text_color.0 == Color::srgba(0.0, 0.0, 10.0, 0.8) {
                    continue;
                }
                if config.mode == GameMode::MultiPlayer {
                    if text.0 == "Player 2".to_string() {
                        text.0 = "".to_string();
                    } else if character_id_text.0 == config.characters_id[1] {
                        text.0 = "Player 2".to_string();
                    }
                } else {
                    if text.0 == "Player 1".to_string() {
                        text.0 = "".to_string();
                    } else if character_id_text.0 == config.characters_id[0] {
                        text.0 = "Player 1".to_string();
                    }
                }
            }
        }
    } else if keys.just_pressed(KeyCode::KeyA) {
        if character_id != 0 {
            config.characters_id[0] = character_id - 1;
            config.characters_id[1] = choose_rand_character(config.characters_id[0]);
            for (mut text, text_color, character_id_text) in text_query.iter_mut() {
                if text_color.0 == Color::srgba(0.0, 0.0, 10.0, 0.8) {
                    continue;
                }
                if config.mode == GameMode::MultiPlayer {
                    if text.0 == "Player 2".to_string() {
                        text.0 = "".to_string();
                    } else if character_id_text.0 == config.characters_id[1] {
                        text.0 = "Player 2".to_string();
                    }
                } else {
                    if text.0 == "Player 1".to_string() {
                        text.0 = "".to_string();
                    } else if character_id_text.0 == config.characters_id[0] {
                        text.0 = "Player 1".to_string();
                    }
                }
            }
        }
    }
}

fn check_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<(&Text, &TextColor)>,
    sound_query: Query<Entity, With<SoundEffect>>,
) {
    for (interaction, children) in &mut interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                if children.len() > 0 {
                    let text = text_query.get(children[0]).unwrap();
                    // reset audio player(unused sound effect entity)
                    for sound in sound_query.iter() {
                        commands.entity(sound).despawn();
                    }
                    match text.0.as_str() {
                        "<Back" => {
                            commands.spawn((
                                AudioPlayer::new(asset_server.load(format!(
                                    "{}button_click.ogg",
                                    PATH_SOUND_PREFIX,
                                ))),
                                SoundEffect,
                            ));
                            #[cfg(not(target_arch = "wasm32"))]
                            next_state.set(AppState::ConnectController);
                            #[cfg(target_arch = "wasm32")]
                            next_state.set(AppState::Mainmenu);
                            break;
                        }
                        "Next>" => {
                            commands.spawn((
                                AudioPlayer::new(asset_server.load(format!(
                                    "{}button_click.ogg",
                                    PATH_SOUND_PREFIX,
                                ))),
                                SoundEffect,
                            ));
                            next_state.set(AppState::Confirm);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn exit(mut commands: Commands, query: Query<Entity, With<ChooseCharacter>>) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub struct ChooseCharacterPlugin;

impl Plugin for ChooseCharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ChooseCharacter), setup)
            .add_systems(OnExit(AppState::ChooseCharacter), exit)
            .add_systems(
                Update,
                check_buttons.run_if(in_state(AppState::ChooseCharacter)),
            );
        #[cfg(not(feature="phone"))]
        app
            .add_systems(
                Update,
                keyboard_input.run_if(in_state(AppState::ChooseCharacter)),
            );
        
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            controller_input.run_if(in_state(AppState::ChooseCharacter)),
        );
    }
}
