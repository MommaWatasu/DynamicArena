use crate::{
    ingame::agent::Level, AppState, GameConfig, GameMode, DEFAULT_FONT_SIZE, PATH_BOLD_FONT,
    PATH_BOLD_JP_FONT, PATH_EXTRA_BOLD_JP_FONT, PATH_IMAGE_PREFIX, TITLE_FONT_SIZE,
};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode},
};
use std::fmt::Display;

#[derive(Component)]
struct Settings;

#[derive(Resource)]
struct SettingIndex {
    idx: u8
};

#[derive(Component, Clone)]
struct SettingItem<T: Clone + ToString + Send + Sync + 'static> {
    name: String,
    min: T,
    max: T,
    step: T,
    value: T,
    list: Option<Vec<String>>,
}

impl<T: Clone + ToString + Send + Sync + Display> SettingItem<T> {
    pub fn new(name: String, min: T, max: T, step: T, value: T, list: Option<Vec<String>>) -> Self {
        SettingItem {
            name,
            min,
            max,
            step,
            value,
            list,
        }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_string(&self) -> String {
        if let Some(list) = &self.list {
            return list[self.value.to_string().parse::<usize>().unwrap() - 1].clone();
        } else {
            format!("{:.1}", self.value)
        }
    }

    pub fn is_list(&self) -> bool {
        self.list.is_some()
    }
}

#[derive(Component)]
struct ConfigElement(u32);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<GameConfig>) {
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
            Settings,
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(10.0),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            Node {
                                justify_self: JustifySelf::Start,
                                align_self: AlignSelf::Start,
                                #[cfg(not(target_arch = "wasm32"))]
                                border: UiRect::all(Val::Px(5.0)),
                                #[cfg(target_arch = "wasm32")]
                                border: UiRect::all(Val::Px(1.0)),
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
                });
            builder
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
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("設定"),
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
                    builder
                        .spawn((
                            Node {
                                width: Val::Percent(90.0),
                                height: Val::Percent(90.0),
                                flex_direction: FlexDirection::Column,
                                align_self: AlignSelf::Center,
                                justify_self: JustifySelf::Center,
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.1, 0.8))),
                            BorderRadius::all(Val::Px(20.0)),
                        ))
                        .with_children(|builder| {
                            create_setting_item(
                                &asset_server,
                                builder,
                                SettingItem::new(
                                    "音量".to_string(),
                                    0f32,
                                    1.0,
                                    0.1,
                                    config.sound_volume,
                                    None,
                                ),
                                0,
                            );
                            #[cfg(not(target_arch = "wasm32"))]
                            create_setting_item(
                                &asset_server,
                                builder,
                                SettingItem::new(
                                    "ゲームモード".to_string(),
                                    1u32,
                                    2,
                                    1,
                                    config.mode as u32,
                                    Some(vec!["シングル".to_string(), "マルチ".to_string()]),
                                ),
                                1,
                            );
                            create_setting_item(
                                &asset_server,
                                builder,
                                SettingItem::new(
                                    "ボットの強さ".to_string(),
                                    1u32,
                                    3,
                                    1,
                                    config.level as u32,
                                    Some(vec![
                                        "弱い".to_string(),
                                        "普通".to_string(),
                                        "強い".to_string(),
                                    ]),
                                ),
                                2,
                            );
                            #[cfg(target_arch = "wasm32")]
                            create_setting_item(
                                &asset_server,
                                builder,
                                SettingItem::new(
                                    "フルスクリーン".to_string(),
                                    1u32,
                                    2,
                                    1,
                                    1,
                                    Some(vec![
                                        "ウィンドウ".to_string(),
                                        "フルスクリーン".to_string(),
                                    ]),
                                ),
                                3,
                            );
                        });
                });
        });
}

fn create_setting_item<T: Clone + ToString + Send + Sync + Display>(
    asset_server: &Res<AssetServer>,
    builder: &mut ChildBuilder,
    item: SettingItem<T>,
    config_num: u32,
) {
    builder
        .spawn((
            Node {
                width: Val::Percent(90.0),
                height: Val::Percent(10.0),
                margin: UiRect {
                    top: Val::Px(5.0),
                    bottom: Val::Px(5.0),
                    ..Default::default()
                },
                flex_direction: FlexDirection::Row,
                align_content: AlignContent::Center,
                ..Default::default()
            },
            BackgroundColor(Color::WHITE),
            BorderRadius::all(Val::Px(20.0)),
        ))
        .with_children(|builder| {
            builder
                .spawn((Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new(item.get_name()),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_JP_FONT),
                            font_size: DEFAULT_FONT_SIZE,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                        ConfigElement(config_num),
                    ));
                });
            builder
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(5.0),
                        height: Val::Percent(100.0),
                        #[cfg(not(target_arch = "wasm32"))]
                        border: UiRect::all(Val::Px(5.0)),
                        #[cfg(target_arch = "wasm32")]
                        border: UiRect::all(Val::Px(1.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    ConfigElement(config_num),
                ))
                .with_child((
                    Text::new("-"),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: DEFAULT_FONT_SIZE,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(Color::BLACK),
                ));
            builder
                .spawn((Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new(item.get_string()),
                        TextFont {
                            font: asset_server.load(if item.is_list() {
                                PATH_BOLD_JP_FONT
                            } else {
                                PATH_BOLD_FONT
                            }),
                            font_size: DEFAULT_FONT_SIZE,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                        item.clone(),
                        ConfigElement(config_num),
                    ));
                });
            builder
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(5.0),
                        height: Val::Percent(100.0),
                        #[cfg(not(target_arch = "wasm32"))]
                        border: UiRect::all(Val::Px(5.0)),
                        #[cfg(target_arch = "wasm32")]
                        border: UiRect::all(Val::Px(1.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    ConfigElement(config_num),
                ))
                .with_child((
                    Text::new("+"),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: DEFAULT_FONT_SIZE,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(Color::BLACK),
                ));
        });
}

fn update_setting(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    button_query: Query<
        (&Interaction, &ConfigElement, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut value_query: Query<
        (&ConfigElement, &mut SettingItem<f32>, &mut Text),
        (With<ConfigElement>, Without<SettingItem<u32>>),
    >,
    mut value_query_int: Query<
        (&ConfigElement, &mut SettingItem<u32>, &mut Text),
        (With<ConfigElement>, Without<SettingItem<f32>>),
    >,
    text_query: Query<&Text, Without<ConfigElement>>,
    mut config: ResMut<GameConfig>,
    mut audio: Query<&mut AudioSink>,
) {
    for (interaction, config_element, children) in &mut button_query.iter() {
        if interaction != &Interaction::Pressed {
            continue;
        }
        let sign;
        match text_query.get(children[0]).unwrap().0.as_str() {
            "-" => {
                sign = false;
            }
            "+" => {
                sign = true;
            }
            _ => sign = false,
        }
        for (element, mut item, mut text) in &mut value_query.iter_mut() {
            if element.0 == config_element.0 {
                let mut new_value = item.value.clone();
                if sign {
                    new_value += item.step.clone();
                } else {
                    new_value -= item.step.clone();
                }
                if new_value < item.min {
                    new_value = item.min.clone();
                } else if new_value > item.max {
                    new_value = item.max.clone();
                }
                item.value = new_value.clone();
                text.0 = format!("{:.1}", new_value);
                if element.0 == 0 {
                    config.sound_volume = new_value;
                    let sink = audio.single_mut();
                    sink.set_volume(new_value);
                }
            }
        }
        for (element, mut item, mut text) in &mut value_query_int.iter_mut() {
            if element.0 == config_element.0 {
                let mut new_value = item.value.clone();
                if sign {
                    new_value += item.step.clone();
                } else {
                    new_value -= item.step.clone();
                }
                if new_value < item.min {
                    new_value = item.min.clone();
                } else if new_value > item.max {
                    new_value = item.max.clone();
                }
                item.value = new_value.clone();
                text.0 = item.get_string();
                if element.0 == 1 {
                    config.mode = GameMode::from(new_value);
                } else if element.0 == 2 {
                    config.level = Level::from(new_value);
                } else if element.0 == 3 {
                    windows.single_mut().mode = if { new_value } == 1 {
                        WindowMode::Windowed
                    } else {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                    };
                }
            }
        }
    }
}

fn controller_input(
    mut next_state: ResMut<NextState<AppState>>,
    mut setting_index: ResMut<SettingIndex>,
    gamepads: Query<&Gamepad>,
) {
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            if setting_index.idx != 0 {
                setting_index.idx -= 1;
            }

        } else if gamepad.just_pressed(GamepadButton::DPadDown) {
            if setting_index.idx != 2 {
                setting_index.idx += 1;
            }
        }
        if gamepad.just_pressed(GamepadButton::DPadLeft) {
        } else if gamepad.just_pressed(GamepadButton::DPadRight) {
        }
        if gamepad.just_pressed(GamepadButton::West) {
            next_state.set(AppState::Mainmenu);
        }
    }
}

fn check_back(
    mut next_state: ResMut<NextState<AppState>>,
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<&Text>,
) {
    for (interaction, children) in &mut interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                if children.len() > 0 {
                    let text = text_query.get(children[0]).unwrap();
                    match text.0.as_str() {
                        "<Back" => {
                            next_state.set(AppState::Mainmenu);
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

fn exit(mut commands: Commands, query: Query<Entity, With<Settings>>) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SettingIndex { idx: 0 })
            .add_systems(OnEnter(AppState::Settings), setup)
            .add_systems(OnExit(AppState::Settings), exit)
            .add_systems(Update, update_setting.run_if(in_state(AppState::Settings)))
            .add_systems(Update, check_back.run_if(in_state(AppState::Settings)))
            .add_systems(Update, controller_input.run_if(in_state(AppState::Settings)));
    }
}
