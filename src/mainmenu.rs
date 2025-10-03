use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::GameConfig;
use crate::{
    AppState, SoundEffect, BGM, DEFAULT_FONT_SIZE, GAMETITLE, PATH_BOLD_FONT, PATH_EXTRA_BOLD_FONT, PATH_IMAGE_PREFIX, PATH_SOUND_PREFIX, TITLE_FONT_SIZE
};

#[derive(Component)]
struct Mainmenu;

#[derive(Resource)]
struct ButtonIndex {
    idx: u8
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    #[cfg(not(target_arch = "wasm32"))] button_idx: Res<ButtonIndex>,
    #[cfg(not(target_arch = "wasm32"))] mut config: ResMut<GameConfig>,
    #[cfg(not(target_arch = "wasm32"))] gamepads: Query<(&Name, Entity), With<Gamepad>>,
    audio_query: Query<(Entity, &BGM)>,
) {
    info!("setup");

    // if audio query is empty, spawn bgm
    if audio_query.is_empty() {
        commands.spawn((
            AudioPlayer::new(asset_server.load(format!("{}Lobby.ogg", PATH_SOUND_PREFIX))),
            PlaybackSettings::LOOP,
            GlobalTransform::default(),
            BGM(true),
        ));
    } else {
        for (entity, bgm) in audio_query.iter() {
            if !bgm.0 {
                commands.entity(entity).despawn();
                commands.spawn((
                    AudioPlayer::new(asset_server.load(format!("{}Lobby.ogg", PATH_SOUND_PREFIX))),
                    PlaybackSettings::LOOP,
                    GlobalTransform::default(),
                    BGM(true),
                ));
            }
        }
    }

    // detect gamepads
    #[cfg(not(target_arch = "wasm32"))]
    for (_, entity) in gamepads.iter() {
        if config.gamepads[0] == Entity::from_raw(0) {
            config.gamepads[0] = entity;
        } else {
            config.gamepads[1] = entity;
        }
        info!("detect gamepad: {:?}", entity);
    }

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
            Mainmenu,
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Percent(80.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(20.0)),
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new(GAMETITLE),
                        TextFont {
                            font: asset_server.load(PATH_EXTRA_BOLD_FONT),
                            font_size: TITLE_FONT_SIZE,
                            ..Default::default()
                        },
                        TextColor(Color::BLACK),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Node {
                            align_self: AlignSelf::Center,
                            justify_self: JustifySelf::Center,
                            margin: UiRect {
                                left: Val::Px(0.0),
                                right: Val::Px(0.0),
                                top: Val::Px(0.0),
                                bottom: Val::Percent(15.0),
                            },
                            ..default()
                        },
                    ));
                    builder
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Percent(10.0),
                                #[cfg(not(target_arch = "wasm32"))]
                                border: UiRect::all(Val::Px(5.0)),
                                #[cfg(target_arch = "wasm32")]
                                border: UiRect::all(Val::Px(2.0)),
                                margin: UiRect::all(Val::Percent(1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            #[cfg(not(target_arch = "wasm32"))]
                            if button_idx.idx == 0 {
                                BorderColor(Color::srgba(10.0, 0.0, 0.0, 0.8))
                            } else {
                                BorderColor(Color::srgba(10.0, 0.0, 0.0, 0.0))
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .with_child((
                            Text::new("Start"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_FONT),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    builder
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Percent(10.0),
                                #[cfg(not(target_arch = "wasm32"))]
                                border: UiRect::all(Val::Px(5.0)),
                                #[cfg(target_arch = "wasm32")]
                                border: UiRect::all(Val::Px(2.0)),
                                margin: UiRect::all(Val::Percent(1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            #[cfg(not(target_arch = "wasm32"))]
                            if button_idx.idx == 1 {
                                BorderColor(Color::srgba(10.0, 0.0, 0.0, 0.8))
                            } else {
                                BorderColor(Color::srgba(10.0, 0.0, 0.0, 0.0))
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .with_child((
                            Text::new("Settings"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_FONT),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    builder
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Percent(10.0),
                                #[cfg(not(target_arch = "wasm32"))]
                                border: UiRect::all(Val::Px(5.0)),
                                #[cfg(target_arch = "wasm32")]
                                border: UiRect::all(Val::Px(2.0)),
                                margin: UiRect::all(Val::Percent(1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            #[cfg(not(target_arch = "wasm32"))]
                            BorderColor(Color::srgba(10.0, 0.0, 0.0, 0.0)),
                            BackgroundColor(Color::BLACK),
                        ))
                        .with_child((
                            Text::new("Exit"),
                            TextFont {
                                font: asset_server.load(PATH_BOLD_FONT),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                });
        });
}

fn controller_input(
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut button_idx: ResMut<ButtonIndex>,
    gamepads: Query<&Gamepad>,
    mut border_query: Query<(&mut BorderColor, &Children)>,
    text_query: Query<&Text>,
) {
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            if button_idx.idx != 0 {
                button_idx.idx -= 1;
                for (mut border_color, children) in border_query.iter_mut() {
                    if text_query.get(children[0]).is_err() {
                        continue;
                    }                    
                    if border_color.0.alpha() != 0.0 {
                        border_color.0.set_alpha(0.0);
                    } else {
                        if text_query.get(children[0]).unwrap().0 == "Start" && button_idx.idx == 0 {
                            border_color.0.set_alpha(0.8);
                        } else if text_query.get(children[0]).unwrap().0 == "Settings" && button_idx.idx == 1 {
                            border_color.0.set_alpha(0.8);
                        } else if text_query.get(children[0]).unwrap().0 == "Exit" && button_idx.idx == 2 {
                            border_color.0.set_alpha(0.8);
                        }
                    }
                }
            }
        } else if gamepad.just_pressed(GamepadButton::DPadDown) {
            if button_idx.idx != 2 {
                button_idx.idx += 1;
                for (mut border_color, children) in border_query.iter_mut() {
                    if text_query.get(children[0]).is_err() {
                        continue;
                    }                    
                    if border_color.0.alpha() != 0.0 {
                        border_color.0.set_alpha(0.0);
                    } else {
                        if text_query.get(children[0]).unwrap().0 == "Start" && button_idx.idx == 0 {
                            border_color.0.set_alpha(0.8);
                        } else if text_query.get(children[0]).unwrap().0 == "Settings" && button_idx.idx == 1 {
                            border_color.0.set_alpha(0.8);
                        } else if text_query.get(children[0]).unwrap().0 == "Exit" && button_idx.idx == 2 {
                            border_color.0.set_alpha(0.8);
                        }
                    }
                }
            }
        } else if gamepad.just_pressed(GamepadButton::South) {
            match button_idx.idx {
                0 => {
                    #[cfg(not(target_arch = "wasm32"))]
                    next_state.set(AppState::ConnectController);
                    #[cfg(target_arch = "wasm32")]
                    next_state.set(AppState::ChooseCharacter);
                }
                1 => {
                    next_state.set(AppState::Settings);
                }
                2 => {
                    app_exit_events.send(AppExit::Success);
                }
                _ => {}
            }
        }
    }
}

fn update(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    asset_server: Res<AssetServer>,
    text_query: Query<&Text>,
    sound_query: Query<Entity, With<SoundEffect>>,
) {
    for (interaction, children) in &mut interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                if children.len() > 0 {
                    let text = text_query.get(children[0]).unwrap();
                    // reset audio player(unused sound effect entity)
                    for entity in sound_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    match text.0.as_str() {
                        "Start" => {
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
                            next_state.set(AppState::ChooseCharacter);
                        }
                        "Settings" => {
                            commands.spawn((
                                AudioPlayer::new(asset_server.load(format!(
                                    "{}button_click.ogg",
                                    PATH_SOUND_PREFIX,
                                ))),
                                SoundEffect,
                            ));
                            next_state.set(AppState::Settings);
                        }
                        "Exit" => {
                            app_exit_events.send(AppExit::Success);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn exit(mut commands: Commands, query: Query<Entity, With<Mainmenu>>) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub struct MainmenuPlugin;

impl Plugin for MainmenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ButtonIndex{ idx: 0 })
            .add_systems(OnEnter(AppState::Mainmenu), setup)
            .add_systems(OnExit(AppState::Mainmenu), exit)
            .add_systems(Update, update.run_if(in_state(AppState::Mainmenu)))
            .add_systems(Update, controller_input.run_if(in_state(AppState::Mainmenu)));
    }
}
