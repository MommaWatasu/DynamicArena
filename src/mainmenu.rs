use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::GameConfig;
use crate::{
    AppState,
    GAMETITLE,
    TITLE_FONT_SIZE,
    PATH_BOLD_FONT,
    PATH_EXTRA_BOLD_FONT,
    PATH_IMAGE_PREFIX,
};

#[cfg(not(target_arch = "wasm32"))]
const BUTTON_FONT_SIZE: f32 = 50.0;
#[cfg(target_arch = "wasm32")]
const BUTTON_FONT_SIZE: f32 = 10.0;
const PATH_SOUND_BGM: &str = "sounds/bgm.ogg";

#[derive(Component)]
struct Mainmenu;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    #[cfg(not(target_arch = "wasm32"))]
    mut config: ResMut<GameConfig>,
    #[cfg(not(target_arch = "wasm32"))]
    gamepads: Query<(&Name, Entity), With<Gamepad>>,
    audio: Query<&AudioPlayer>,
) {
    info!("setup");

    // if audio query is empty, spawn bgm
    if audio.is_empty() {
        commands.spawn((
            AudioPlayer::new(asset_server.load(PATH_SOUND_BGM)),
            PlaybackSettings::LOOP,
            GlobalTransform::default(),
        ));
    }

    // detect gamepads
    #[cfg(not(target_arch = "wasm32"))]
    for (name, entity) in gamepads.iter() {
        if **name == *"DynamicArena Controller" {
            if config.gamepads[0] == Entity::from_raw(0) {
                config.gamepads[0] = entity;
            } else {
                config.gamepads[1] = entity;
            }
            info!("detect gamepad: {:?}", entity);
        }
    }

    commands
        .spawn((
            #[cfg(not(target_arch = "wasm32"))]
            ImageNode::new(asset_server.load(format!("{}background_mainmenu.png", PATH_IMAGE_PREFIX))),
            #[cfg(target_arch = "wasm32")]
            ImageNode::new(asset_server.load(format!("{}web/background_mainmenu.png", PATH_IMAGE_PREFIX))),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Mainmenu
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
                    builder.spawn((
                        Button,
                        Node {
                            width: Val::Percent(50.0),
                            height: Val::Percent(10.0),
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::all(Val::Percent(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    ))
                    .with_child((
                        Text::new("Start"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: BUTTON_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    builder.spawn((
                        Button,
                        Node {
                            width: Val::Percent(50.0),
                            height: Val::Percent(10.0),
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::all(Val::Percent(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    ))
                    .with_child((
                        Text::new("Settings"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: BUTTON_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    builder.spawn((
                        Button,
                        Node {
                            width: Val::Percent(50.0),
                            height: Val::Percent(10.0),
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::all(Val::Percent(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    ))
                    .with_child((
                        Text::new("Exit"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: BUTTON_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn update(
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    text_query: Query<&Text>,
) {
    for (interaction, children) in &mut interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                if children.len() > 0 {
                    let text = text_query.get(children[0]).unwrap();
                    match text.0.as_str() {
                        "Start" => {
                            #[cfg(not(target_arch = "wasm32"))]
                            next_state.set(AppState::ConnectController);
                            #[cfg(target_arch = "wasm32")]
                            next_state.set(AppState::ChooseCharacter);
                        }
                        "Settings" => {
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

fn exit(
    mut commands: Commands,
    query: Query<Entity, With<Mainmenu>>,
) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct MainmenuPlugin;

impl Plugin for MainmenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Mainmenu), setup)
            .add_systems(OnExit(AppState::Mainmenu), exit)
            .add_systems(Update, update.run_if(in_state(AppState::Mainmenu)));
    }
}