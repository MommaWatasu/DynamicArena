use bevy::prelude::*;
use crate::{
    AppState, GameConfig, GameMode, PATH_BOLD_FONT, PATH_BOLD_JP_FONT, PATH_EXTRA_BOLD_JP_FONT, TITLE_FONT_SIZE,
};

#[derive(Component)]
struct Settings;

#[derive(Component, Clone)]
struct SettingItem<T: Clone + ToString + Send + Sync + 'static> {
    name: String,
    min: T,
    max: T,
    step: T,
    value: T,
    list: Option<Vec<String>>
}

impl<T: Clone + ToString + Send + Sync> SettingItem<T> {
    pub fn new(name: String, min: T, max: T, step: T, value: T, list: Option<Vec<String>>) -> Self {
        SettingItem { name, min, max, step, value, list }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_value(&self) -> String {
        self.value.to_string()
    }
}

#[derive(Component)]
struct ConfigElement(u32);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("settings: setup");
    commands.spawn((
        Button,
        Node {
            justify_self: JustifySelf::Start,
            align_self: AlignSelf::Start,
            border: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        BorderRadius::MAX,
        BorderColor(Color::BLACK),
        Settings
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
    commands.spawn((
        Node {
            width: Val::Percent(90.0),
            height: Val::Percent(90.0),
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            ..Default::default()
        },
        Settings
    ))
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
            builder.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    align_items: AlignItems::Start,
                    justify_items: JustifyItems::Start,
                    ..default()
                },
                BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.1, 0.8))),
                BorderRadius::all(Val::Px(20.0)),
            ))
                .with_children(|builder| {
                    create_setting_item(&asset_server, builder, SettingItem::new("音量".to_string(), 0f32, 1.0, 0.1, 0.5, None), 0);
                    create_setting_item(&asset_server, builder, SettingItem::new("プレイヤー数".to_string(), 1u32, 2, 1, 1, None), 1);
                });
    });
}

fn create_setting_item<T: Clone + ToString + Send + Sync>(
    asset_server: &Res<AssetServer>,
    builder: &mut ChildBuilder,
    item: SettingItem<T>,
    config_num: u32,
) {
    builder.spawn((
        Node{
            width: Val::Percent(100.0),
            height: Val::Percent(10.0),
            flex_direction: FlexDirection::Row,
            align_content: AlignContent::Center,
            ..Default::default()
        },
    ))
        .with_children(|builder| {
            builder.spawn((
                Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new(item.get_name()),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_JP_FONT),
                            font_size: 50.0,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                        ConfigElement(config_num),
                    ));
                });
            builder.spawn((
                Button,
                Node {
                    width: Val::Percent(5.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(5.0)),
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
                        font_size: 50.0,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(Color::BLACK),
                ));
            builder.spawn((
                Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new(item.get_value()),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: 50.0,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                        item.clone(),
                        ConfigElement(config_num),
                    ));
                });
            builder.spawn((
                Button,
                Node {
                    width: Val::Percent(5.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(5.0)),
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
                        font_size: 50.0,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(Color::BLACK),
                ));
        });
}

fn update_setting(
    button_query: Query<(&Interaction, &ConfigElement, &Children), (Changed<Interaction>, With<Button>)>,
    mut value_query: Query<(&ConfigElement, &mut SettingItem<f32>, &mut Text),(With<ConfigElement>, Without<SettingItem<u32>>),>,
    mut value_query_int: Query<(&ConfigElement, &mut SettingItem<u32>, &mut Text),(With<ConfigElement>, Without<SettingItem<f32>>)>,
    text_query: Query<&Text, Without<ConfigElement>>,
    mut config: ResMut<GameConfig>,
) {
    for (interaction, config_element, children) in &mut button_query.iter() {
        if interaction != &Interaction::Pressed { continue }
        let sign;
        match text_query.get(children[0]).unwrap().0.as_str() {
            "-" => {
                sign = false;
            }
            "+" => {
                sign = true;
            }
            _ => {
                sign = false
            }
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
                text.0 = new_value.to_string();
                if element.0 == 1 {
                    config.mode = if {new_value} == 1 {GameMode::SinglePlayer} else {GameMode::MultiPlayer};
                }
            }
        }
    }
}

fn check_back(
    mut state: ResMut<NextState<AppState>>,
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
                            state.set(AppState::Mainmenu);
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

fn exit(
    mut commands: Commands,
    query: Query<Entity, With<Settings>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("settings: exit");
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Settings), setup)
            .add_systems(OnExit(AppState::Settings), exit)
            .add_systems(Update, update_setting.run_if(in_state(AppState::Settings)))
            .add_systems(Update, check_back.run_if(in_state(AppState::Settings)));
    }
}