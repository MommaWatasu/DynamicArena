use bevy::{
    input::gamepad::{Gamepad, GamepadConnectionEvent},
    prelude::*,
};

use crate::{
    AppState, GameConfig, GameMode, SoundEffect, PATH_SOUND_PREFIX, PATH_BOLD_FONT, PATH_BOLD_JP_FONT, PATH_EXTRA_BOLD_JP_FONT,
    PATH_IMAGE_PREFIX, TITLE_FONT_SIZE,
};

#[derive(Component)]
struct ConnectController;

#[derive(Component)]
pub struct GamepadID(u8);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, game_config: Res<GameConfig>) {
    info!("setup");
    commands
        .spawn((
            #[cfg(not(target_arch = "wasm32"))]
            ImageNode::new(
                asset_server.load(format!("{}background_mainmenu.png", PATH_IMAGE_PREFIX)),
            ),
            #[cfg(target_arch = "wasm32")]
            ImageNode::new(
                asset_server.load(format!("{}web/background_mainmenu.png", PATH_IMAGE_PREFIX)),
            ),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ConnectController,
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
                                border: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BorderRadius::MAX,
                            BorderColor(Color::BLACK),
                        ))
                        .with_child((
                            Text::new("<Back"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_FONT),
                                font_size: 50.0,
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
                                border: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BorderRadius::MAX,
                            BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                        ))
                        .with_child((
                            Text::new("Next>"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_FONT),
                                font_size: 50.0,
                                ..Default::default()
                            },
                            TextLayout::new_with_justify(JustifyText::Center),
                            TextColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                        ));
                });
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
                        Text::new("コントローラーを接続してください"),
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
                            if game_config.mode == GameMode::SinglePlayer {
                                create_controller_box(
                                    spawner,
                                    &asset_server,
                                    &game_config.gamepads,
                                    0,
                                );
                            } else {
                                create_controller_box(
                                    spawner,
                                    &asset_server,
                                    &game_config.gamepads,
                                    0,
                                );
                                create_controller_box(
                                    spawner,
                                    &asset_server,
                                    &game_config.gamepads,
                                    1,
                                );
                            }
                        });
                });
        });
}

fn create_controller_box(
    spawner: &mut ChildSpawnerCommands,
    asset_server: &Res<AssetServer>,
    gamepads: &[Entity; 2],
    id: u8,
) {
    spawner
        .spawn((
            Node {
                width: Val::Percent(40.0),
                height: Val::Percent(90.0),
                justify_self: JustifySelf::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GamepadID(id),
            BorderRadius::all(Val::Px(20.0)),
            BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
        ))
        .with_children(|spawner| {
            spawner.spawn((
                Text::new(format!("コントローラー {}", id + 1)),
                TextFont {
                    font: asset_server.load(PATH_BOLD_JP_FONT),
                    font_size: 40.0,
                    ..Default::default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::BLACK),
            ));
            spawner.spawn((
                if gamepads[id as usize] == Entity::from_raw(0) {
                    Text::new("未接続")
                } else {
                    Text::new("接続済み")
                },
                TextFont {
                    font: asset_server.load(PATH_BOLD_JP_FONT),
                    font_size: 40.0,
                    ..Default::default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                if gamepads[id as usize] == Entity::from_raw(0) {
                    TextColor(Color::srgb(1.0, 0.0, 0.0))
                } else {
                    TextColor(Color::srgb(0.0, 1.0, 0.0))
                },
            ));
        });
}

fn update_controller(
    mut game_config: ResMut<GameConfig>,
    mut connection_event: EventReader<GamepadConnectionEvent>,
    gamepads: Query<(&Name, &Gamepad)>,
    gamepad_query: Query<(&GamepadID, &Children)>,
    mut button_query: Query<(&Children, &mut BorderColor), With<Button>>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) {
    // process connection event
    for event in connection_event.read() {
        if event.connected() {
            info!("Gamepad connected: {:?}", event.gamepad);
            let mut id: u8 = 0;
            if game_config.gamepads[0] == Entity::from_raw(0) {
                game_config.gamepads[0] = event.gamepad;
            } else if game_config.gamepads[1] == Entity::from_raw(0) {
                game_config.gamepads[1] = event.gamepad;
                id = 1;
            }
            for (gamepad_id, children) in gamepad_query.iter() {
                if gamepad_id.0 == id {
                    if let Ok((mut text, mut text_color)) = text_query.get_mut(children[1]) {
                        if text.0 != "未接続" {
                            continue;
                        }
                        text.0 = "接続済み".to_string();
                        text_color.0 = Color::srgb(0.0, 1.0, 0.0);
                    }
                }
            }
        } else {
            info!("Gamepad disconnected: {:?}", event.gamepad);
            let mut id: u8 = 0;
            if game_config.gamepads[0] == event.gamepad {
                game_config.gamepads[0] = Entity::from_raw(0);
            } else if game_config.gamepads[1] == event.gamepad {
                game_config.gamepads[1] = Entity::from_raw(0);
                id = 1;
            }
            for (gamepad_id, children) in gamepad_query.iter() {
                if gamepad_id.0 == id {
                    if let Ok((mut text, mut text_color)) = text_query.get_mut(children[1]) {
                        if text.0 != "接続済み" {
                            continue;
                        }
                        text.0 = "未接続".to_string();
                        text_color.0 = Color::srgb(1.0, 0.0, 0.0);
                    }
                }
            }
        }
    }
    connection_event.clear();

    // Count required controllers based on game mode
    let required_controllers = if game_config.mode == GameMode::SinglePlayer {
        1
    } else {
        2
    };

    // Track connected gamepads
    let connected_count = gamepads.iter().count();

    // Update buttons based on controller status
    for (children, mut border_color) in button_query.iter_mut() {
        if children.is_empty() {
            continue;
        }

        if let Ok((text, mut text_color)) = text_query.get_mut(children[0]) {
            match text.as_str() {
                "Next>" => {
                    // Enable/disable Next button
                    if connected_count >= required_controllers {
                        *border_color = BorderColor(Color::BLACK);
                        *text_color = TextColor(Color::BLACK);
                    } else {
                        *border_color = BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.8));
                        *text_color = TextColor(Color::srgba(0.0, 0.0, 0.0, 0.8));
                    }
                }
                _ => {}
            }
        }
    }
}

fn controller_input(
    mut next_state: ResMut<NextState<AppState>>,
    gamepads: Query<&Gamepad>,
) {
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::East) {
            next_state.set(AppState::Mainmenu);
        } else if gamepad.just_pressed(GamepadButton::West) {
            next_state.set(AppState::ChooseCharacter);
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
                            next_state.set(AppState::Mainmenu);
                            break;
                        }
                        "Next>" => {
                            // NOTE: For now, we will skip the controller check
                            //if text.1.0 == Color::BLACK {
                            commands.spawn((
                                AudioPlayer::new(asset_server.load(format!(
                                    "{}button_click.ogg",
                                    PATH_SOUND_PREFIX,
                                ))),
                                SoundEffect,
                            ));
                            next_state.set(AppState::ChooseCharacter);
                            //}
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

fn exit(mut commands: Commands, query: Query<Entity, With<ConnectController>>) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub struct ConnectControllerPlugin;

impl Plugin for ConnectControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ConnectController), setup)
            .add_systems(OnExit(AppState::ConnectController), exit)
            .add_systems(
                Update,
                check_buttons.run_if(in_state(AppState::ConnectController)),
            )
            .add_systems(
                Update,
                update_controller.run_if(in_state(AppState::ConnectController)),
            )
            .add_systems(
                Update,
                controller_input.run_if(in_state(AppState::ConnectController)),
            );
    }
}
