use crate::{
    ingame::GameState, AppState, SoundEffect, Score, PATH_SOUND_PREFIX, DEFAULT_FONT_SIZE, PATH_BOLD_FONT, PATH_EXTRA_BOLD_FONT,
    PATH_IMAGE_PREFIX, TITLE_FONT_SIZE,
};
use bevy::prelude::*;

#[derive(Component)]
struct ShowResult;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, gamestate: Res<GameState>, mut score: ResMut<Score>) {
    info!("setup");
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
                        height: Val::Percent(80.0),
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
        if gamepad.just_pressed(GamepadButton::West) {
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
