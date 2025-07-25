use bevy::prelude::*;

use crate::{
    AppState, GameConfig, GameMode, DEFAULT_FONT_SIZE, PATH_BOLD_FONT, PATH_BOLD_JP_FONT,
    PATH_EXTRA_BOLD_JP_FONT, PATH_IMAGE_PREFIX, TITLE_FONT_SIZE,
};

#[derive(Component)]
struct Confirm;

#[derive(Resource)]
struct StartGameTimer(Timer);

#[derive(Component)]
struct CountText;

fn setup(mut commands: Commands, config: Res<GameConfig>, asset_server: Res<AssetServer>) {
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
        Confirm,
    ))
    .with_children(|builder| {
        builder
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(10.0),
                justify_content: JustifyContent::SpaceBetween,
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
                    Text::new("まもなく開始します"),
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
                            justify_content: JustifyContent::SpaceEvenly,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                        BorderRadius::all(Val::Px(20.0)),
                    ))
                    .with_children(|builder| {
                        builder.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(10.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                            BorderRadius::all(Val::Px(10.0)),
                        ))
                            .with_child((
                                Text::new("10病後に自動開始します"),
                                CountText,
                                TextFont {
                                    font: asset_server.load(PATH_BOLD_JP_FONT),
                                    font_size: DEFAULT_FONT_SIZE,
                                    ..Default::default()
                                },
                                TextColor(Color::BLACK),
                                TextLayout::new_with_justify(JustifyText::Center),
                            ));
                        builder.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(70.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.6, 0.8, 0.9, 0.8)),
                        ))
                            .with_children(|builder| {
                                create_player_box(builder, &asset_server, 0, config.characters_id[0], false);
                                builder.spawn((
                                    Text::new("VS"),
                                    TextFont {
                                        font: asset_server.load(PATH_BOLD_JP_FONT),
                                        font_size: DEFAULT_FONT_SIZE,
                                        ..Default::default()
                                    },
                                    TextColor(Color::srgba(20.0, 0.0, 0.0, 1.0)),
                                    TextLayout::new_with_justify(JustifyText::Center),
                                ));
                                create_player_box(builder, &asset_server, 1, config.characters_id[1], config.mode == GameMode::SinglePlayer);
                            });
                        builder.spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(10.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                            BorderRadius::all(Val::Px(10.0)),
                        ))
                            .with_child((
                                Text::new("戦闘開始"),
                                TextFont {
                                    font: asset_server.load(PATH_BOLD_FONT),
                                    font_size: DEFAULT_FONT_SIZE,
                                    ..Default::default()
                                },
                                TextLayout::new_with_justify(JustifyText::Center),
                                TextColor(Color::BLACK),
                            ));
                    });
            });
    });
}

fn create_player_box(
    builder: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    player_id: u8,
    character_id: isize,
    agent: bool,
) {
    builder.spawn((
        Node {
            width: Val::Percent(40.0),
            height: Val::Percent(90.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.6, 0.8, 0.9, 0.8)),
    ))
    .with_children(|builder| {
        builder.spawn((
            if agent {
                Text::new("Bot")
            } else {
                Text::new(format!("Player {}", player_id + 1)) 
            },
            TextFont {
                font: asset_server.load(PATH_BOLD_JP_FONT),
                font_size: DEFAULT_FONT_SIZE,
                ..Default::default()
            },
            TextColor(Color::BLACK),
            TextLayout::new_with_justify(JustifyText::Center),
        ));
        builder.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                ..default()
            },
        ))
        .with_children(|builder| {
            builder.spawn((
                ImageNode::new(asset_server.load(format!(
                    "{}character_{}.png",
                    PATH_IMAGE_PREFIX, character_id + 1
                ))),
            ));
        });
    });
}

fn update(
    time: Res<Time>,
    mut timer: ResMut<StartGameTimer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut text_query: Query<&mut Text, With<CountText>>,
) {
    timer.0.tick(time.delta());
    for mut text in text_query.iter_mut() {
        text.0 = format!("{}秒後に自動開始", 10 - timer.0.elapsed_secs().round() as u8);
    }
    // automatically start the game after 10 seconds
    if timer.0.just_finished() {
        next_state.set(AppState::Ingame);
    }
}

fn controller_input(
    mut next_state: ResMut<NextState<AppState>>,
    gamepads: Query<&Gamepad>,
) {
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            next_state.set(AppState::ChooseCharacter);
        } else if gamepad.just_pressed(GamepadButton::West) {
            next_state.set(AppState::Ingame);
        }
    }
}

fn check_buttons(
    mut next_state: ResMut<NextState<AppState>>,
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<(&Text, &TextColor)>,
) {
    for (interaction, children) in &mut interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                if children.len() > 0 {
                    let text = text_query.get(children[0]).unwrap();
                    match text.0.as_str() {
                        "<Back" => {
                            next_state.set(AppState::ChooseCharacter);
                            break;
                        }
                        "戦闘開始" => {
                            next_state.set(AppState::Ingame);
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

fn exit(mut commands: Commands, mut timer: ResMut<StartGameTimer>, query: Query<Entity, With<Confirm>>) {
    info!("exit");
    timer.0.reset();
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct ConfirmPlugin;

impl Plugin for ConfirmPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(StartGameTimer(Timer::from_seconds(10.0, TimerMode::Once)))
            .add_systems(OnEnter(AppState::Confirm), setup)
            .add_systems(OnExit(AppState::Confirm), exit)
            .add_systems(Update, update.run_if(in_state(AppState::Confirm)))
            .add_systems(Update, check_buttons.run_if(in_state(AppState::Confirm)))
            .add_systems(Update, controller_input.run_if(in_state(AppState::Confirm)));
    }
}