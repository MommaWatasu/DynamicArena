use bevy::prelude::*;

use crate::{
    AppState, GameConfig, PATH_BOLD_FONT, PATH_BOLD_JP_FONT, PATH_EXTRA_BOLD_JP_FONT, TITLE_FONT_SIZE
};

#[derive(Component)]
struct Settings;

#[derive(Component, Clone)]
struct SettingItem {
    name: String,
    min: f32,
    max: f32,
    value: f32,
}

impl SettingItem {
    pub fn new(name: String, min: f32, max: f32, value: f32) -> Self {
        SettingItem { name, min, max, value, }
    }
}

#[derive(Component)]
struct ConfigElement {
    value: u32,
}

impl ConfigElement {
    pub fn new(value: u32) -> Self {
        ConfigElement { value }
    }
}

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
        BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.1, 0.8))),
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
                    create_setting_item(&asset_server, builder, SettingItem::new("音量".to_string(), 0.0, 1.0, 0.5), 0);
                    create_setting_item(&asset_server, builder, SettingItem::new("プレイヤー数".to_string(), 1.0, 2.0, 1.0), 0);
                });
    });
}

fn create_setting_item(
    asset_server: &Res<AssetServer>,
    builder: &mut ChildBuilder,
    item: SettingItem,
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
                        Text::new(format!("{}", &item.name)),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_JP_FONT),
                            font_size: 50.0,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                        item.clone(),
                        ConfigElement::new(config_num),
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
                    ConfigElement::new(config_num),
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
                        Text::new(format!("{:.2}", &item.value)),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: 50.0,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                        item.clone(),
                        ConfigElement::new(config_num),
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
                    ConfigElement::new(config_num),
                ));
        });
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

fn update(
    query: Query<(Entity, &ConfigElement, &SettingItem)>,
    mut config: ResMut<GameConfig>,
) {}

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

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Settings), setup)
            .add_systems(OnExit(AppState::Settings), exit)
            .add_systems(Update, update.run_if(in_state(AppState::Settings)))
            .add_systems(Update, check_back.run_if(in_state(AppState::Settings)));
    }
}