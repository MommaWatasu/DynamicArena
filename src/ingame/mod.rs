use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues}};
use bevy_rapier2d::prelude::*;
use pose::{LOSER_POSE, WINNER_POSE};

#[cfg(debug_assertions)]
mod pause;
#[cfg(not(target_arch = "wasm32"))]
mod controller;
mod bot;
mod player;
mod pose;

use crate::{
    AppState, GameConfig, PATH_BOLD_FONT, PATH_BOLD_MONOSPACE_FONT, PATH_EXTRA_BOLD_FONT, PATH_IMAGE_PREFIX
};

#[cfg(debug_assertions)]
use pause::*;
#[cfg(not(target_arch = "wasm32"))]
use controller::*;
use player::*;

const FPS: f32 = 60.0;

#[derive(Resource, Default)]
pub struct GameState {
    pub winners: [u8; 3],
    pub win_types: [bool; 3],
    pub round: u8,
    pub phase: u8,
    pub count: u8,
    pub timer: Timer
}

impl GameState {
    // return the total winner
    pub fn get_winner(&self) -> u8 {
        let mut player1 = 0;
        let mut player2 = 0;
        for winner in self.winners.iter() {
            if *winner == 1 {
                player1 += 1;
            } else if *winner == 2 {
                player2 += 1;
            }
        }
        if player1 > player2 {
            return 1;
        } else if player1 < player2 {
            return 2;
        } else {
            return 0;
        }
    }
}

#[derive(Resource)]
struct Fighting;

#[derive(Component)]
struct InGame;

#[derive(Component)]
struct SkyBackground;

#[derive(Component)]
struct BackGround;

#[derive(Component)]
struct StatusBar;

#[derive(Component)]
struct Curtain;

#[derive(Component)]
struct GameTimer(f32);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<GameState>,
    config: Res<GameConfig>,
) {
    info!("setup");
    #[cfg(debug_assertions)]
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
        InGame
    ))
        .with_child((
            Text::new("Pause"),
            TextFont {
                font: asset_server.load(PATH_BOLD_FONT),
                font_size: 50.0,
                ..Default::default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::BLACK),
        ));

    commands
        .spawn((
            InGame,
            Node {
                width: Val::Px(300.0),
                height: Val::Px(100.0),
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Start,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            BorderRadius::all(Val::Px(5.0)),
        ))
            .with_children(|builder| {
                builder.spawn((
                    GameTimer(60.0),
                    Text::new("60.00"),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_MONOSPACE_FONT),
                        font_size: 50.0,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(Color::WHITE),
                ));
            });
    commands
        .spawn((
            InGame,
            PlayerID(1),
            HealthBar(1.0, config.window_size.x / 2.0 - 250.0),
            Mesh2d(meshes.add(
                Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            vec![[0.0, 0.0, 1.0], [0.0, 40.0, 1.0], [config.window_size.x / 2.0 - 300.0, 0.0, 1.0], [config.window_size.x / 2.0 - 250.0, 40.0, 1.0]]
                        )
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_COLOR,
                            vec![[0.0, 1.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.5], [0.0, 1.0, 0.0, 0.5]]
                        )
                        .with_inserted_indices(Indices::U32(vec![
                            0, 1, 2,
                            1, 2, 3]))
            )),
            MeshMaterial2d(materials.add(ColorMaterial::default())),
            Transform::from_translation(Vec3::new(150.0, config.window_size.y / 2.0 - 50.0, 1.0)),
        ));
    commands
        .spawn((
            InGame,
            PlayerID(0),
            HealthBar(1.0, 250.0 - config.window_size.x / 2.0),
            Mesh2d(meshes.add(
                Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            vec![[0.0, 0.0, 1.0], [0.0, 40.0, 1.0], [300.0 - config.window_size.x / 2.0, 0.0, 1.0], [250.0 - config.window_size.x / 2.0, 40.0, 1.0]]
                        )
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_COLOR,
                            vec![[0.0, 1.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.5], [0.0, 1.0, 0.0, 0.5]]
                        )
                        .with_inserted_indices(Indices::U32(vec![
                            0, 1, 2,
                            1, 2, 3]))
            )),
            MeshMaterial2d(materials.add(ColorMaterial::default())),
            Transform::from_translation(Vec3::new(-150.0, config.window_size.y / 2.0 - 50.0, 1.0)),
        ));
    commands.spawn((
        Sprite {
            image: asset_server.load(format!("{}sky_upscaled.png", PATH_IMAGE_PREFIX)),
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0.0, 100.0, -1.0)),
        SkyBackground,
        InGame
    ));
    commands.spawn((
        Sprite {
            image: asset_server.load(format!("{}sky_upscaled.png", PATH_IMAGE_PREFIX)),
            flip_x: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(4800.0, 100.0, -1.0)),
        SkyBackground,
        InGame
    ));
    commands
        .spawn((
            Sprite{
                #[cfg(not(target_arch = "wasm32"))]
                image: asset_server.load(format!("{}background.png", PATH_IMAGE_PREFIX)),
                #[cfg(target_arch = "wasm32")]
                image: asset_server.load(format!("{}web/background.png", PATH_IMAGE_PREFIX)),
                ..Default::default()
            },
            BackGround,
            Transform::default(),
            InGame
        ));
    spawn_player(0, config.characters_id[0], &mut commands, &mut meshes, &mut materials, 270.0-config.window_size.y / 2.0);
    spawn_player(1, config.characters_id[1], &mut commands, &mut meshes, &mut materials, 270.0-config.window_size.y / 2.0);
    game_state.phase = 0;
    game_state.count = 0;
    game_state.round = 1;
    game_state.timer = Timer::from_seconds(1.0 / FPS, TimerMode::Repeating);
}

fn update_timer(
    time: Res<Time>,
    mut gamestate: ResMut<GameState>,
    mut query: Query<(&mut Text, &mut TextColor, &mut GameTimer)>,
    health_bar_query: Query<(&HealthBar, &PlayerID)>,
) {
    let (mut text, mut color, mut timer) = query.single_mut();
    if timer.0 == 0.0 {
        return;
    }
    timer.0 -= time.delta_secs();
    if timer.0 < 0.0 {
        timer.0 = 0.0;
        let bars: Vec<_> = health_bar_query.iter().collect();
        let winner_id = if bars[0].0.0 < bars[1].0.0 {
            bars[1].1.0 + 1
        } else if bars[0].0.0 == bars[1].0.0 {
            0
        } else {
            bars[0].1.0 + 1
        };
        let round = gamestate.round as usize - 1;
        gamestate.winners[round] = winner_id;
        gamestate.win_types[round] = false;
        gamestate.phase = 6;
    } else if timer.0 < 5.0 {
        color.0 = Color::srgb(1.0, 0.0, 0.0);
    }
    text.0 = format!("{:.2}", timer.0);
}

fn check_gameset(
    mut gamestate: ResMut<GameState>,
    query: Query<(&HealthBar, &PlayerID)>
) {
    for (bar, player_id) in query.iter() {
        if bar.0 <= 0.0 {
            let round = gamestate.round as usize - 1;
            gamestate.winners[round] = if player_id.0 == 0 { 2 } else { 1 };
            gamestate.win_types[round] = true;
            gamestate.phase = 6;
            break;
        }
    }
}

fn main_game_system(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut gamestate: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<(&mut BackgroundColor, &mut Text, &mut TextColor), With<StatusBar>>,
    mut curtain_query: Query<&mut BackgroundColor, (With<Curtain>, Without<StatusBar>)>,
    mut player_query: Query<(&PlayerID, &mut Player, &mut Transform)>,
    mut health_query: Query<(&mut HealthBar, &mut Mesh2d, &PlayerID)>,
    mut timer_query: Query<(&mut Text, &mut TextColor, &mut GameTimer), Without<StatusBar>>,
) {
    gamestate.timer.tick(time.delta());
    if gamestate.timer.just_finished() {
        if gamestate.phase == 0 {
            if gamestate.round == 1 {
                commands.spawn((
                    InGame,
                    Curtain,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_self: JustifySelf::Center,
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                ));
                commands.spawn((
                    InGame,
                    StatusBar,
                    Node {
                        width: Val::Percent(100.0),
                        justify_self: JustifySelf::Center,
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),

                    Text::new("ROUND 1"),
                    TextFont {
                        font: asset_server.load(PATH_EXTRA_BOLD_FONT),
                        font_size: 100.0,
                        ..Default::default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                ));
            } else {
                let (mut bar, mut text, mut text_color) = query.get_single_mut().unwrap();
                bar.0 = Color::srgba(0.0, 0.0, 0.0, 0.8);
                text.0 = format!("ROUND {}", gamestate.round);
                text_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.8);
            }
            gamestate.phase = 1;
            gamestate.count = 0;
        } else if gamestate.phase == 1 {
            gamestate.count += 1;
            if gamestate.count == 60 {
                let (_, mut text, _) = query.get_single_mut().unwrap();
                text.0 = "READY?".to_string();
                gamestate.phase = 2;
                gamestate.count = 0;
            }
        } else if gamestate.phase == 2 {
            gamestate.count += 1;
            if gamestate.count == 90 {
                let (_, mut text, _) = query.get_single_mut().unwrap();
                text.0 = "FIGHT!".to_string();
                gamestate.phase = 3;
                gamestate.count = 0;
            }
        } else if gamestate.phase == 3 {
            gamestate.count += 1;
            if gamestate.count == 30 {
                gamestate.phase = 4;
                gamestate.count = 0;
            }
        } else if gamestate.phase == 4 {
            gamestate.count += 1;
            let (mut bar, _, mut text_color) = query.get_single_mut().unwrap();
            bar.0 = Color::srgba(0.0, 0.0, 0.0, 0.8 - gamestate.count as f32/60.0);
            text_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.8 - gamestate.count as f32/60.0);
            if gamestate.count == 48 {
                commands.insert_resource(Fighting);
                gamestate.phase = 5;
                gamestate.count = 0;
            }
        } else if gamestate.phase == 6 {
            if gamestate.count == 0 {
                let (mut bar, mut text, mut text_color) = query.get_single_mut().unwrap();
                bar.0 = Color::srgba(0.0, 0.0, 0.0, 0.8);
                text.0 = if gamestate.win_types[gamestate.round as usize - 1] {
                    "KO!".to_string()
                } else {
                    "TIME UP!".to_string()
                };
                text_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.8);
                // winner and loser animation
                if gamestate.winners[gamestate.round as usize - 1] == 0 {
                    commands.remove_resource::<Fighting>();
                    gamestate.phase = 7;
                    gamestate.count = 0;
                } else {
                    for (id, mut player, _) in player_query.iter_mut() {
                        if gamestate.winners[gamestate.round as usize - 1] == id.0 + 1 {
                            player.set_animation(WINNER_POSE, 0, 10);
                        } else {
                            player.set_animation(LOSER_POSE, 0, 10);
                        }
                    }
                    gamestate.count = 1;
                }
            }
        } else if gamestate.phase == 7 {
            gamestate.count += 1;
            if gamestate.count == 60 {
                gamestate.phase = 8;
                gamestate.count = 0;
            }
        } else if gamestate.phase == 8 {
            let (_, mut text, _) = query.get_single_mut().unwrap();
            let winner_id = gamestate.winners[gamestate.round as usize - 1];
            if winner_id == 0 {
                text.0 = "DRAW".to_string();
            } else {
                text.0 = format!("Player {} WIN", winner_id);
            }
            gamestate.phase = 9;
            gamestate.count = 0;
        } else if gamestate.phase == 9 {
            gamestate.count += 1;
            if gamestate.count == 60 {
                gamestate.phase = 10;
                gamestate.count = 0;
            }
        } else if gamestate.phase == 10 {
            gamestate.count += 1;
            let mut curtain = curtain_query.get_single_mut().unwrap();
            curtain.0 = Color::srgba(0.0, 0.0, 0.0, gamestate.count as f32/60.0);
            if gamestate.count == 60 {
                gamestate.round += 1;
                if gamestate.round == 4 {
                    // change app state to show result
                    next_state.set(AppState::Result);
                } else {
                    gamestate.phase = 11;
                    gamestate.count = 0;

                    // remove status bar
                    let (mut bar, _, mut text_color) = query.get_single_mut().unwrap();
                    bar.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
                    text_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
                    // reset player
                    for (id, mut player, mut transform) in player_query.iter_mut() {
                        player.reset(id);
                        transform.translation.x = if id.0 == 0 { -500.0 } else { 500.0 };
                        transform.translation.y = 270.0 - config.window_size.y / 2.0;
                    }
                    // reset health bar
                    for (mut health_bar, mesh_handler, health_id) in health_query.iter_mut() {
                        health_bar.0 = 1.0;
                        if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                            if let Some(VertexAttributeValues::Float32x3(ref mut positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
                                positions[3][0] = health_bar.1 * health_bar.0;
                                positions[2][0] = health_bar.1 * health_bar.0 + if health_id.0 == 0 { 50.0 } else { -50.0 };
                            }
                        }
                    }
                    // reset timer
                    let (mut text, mut color, mut timer) = timer_query.get_single_mut().unwrap();
                    timer.0 = 60.0;
                    text.0 = "60.00".to_string();
                    color.0 = Color::WHITE;
                }
            }
        } else if gamestate.phase == 11 {
            gamestate.count += 1;
            let mut curtain = curtain_query.get_single_mut().unwrap();
            curtain.0 = Color::srgba(0.0, 0.0, 0.0, 1.0 - gamestate.count as f32/60.0);
            if gamestate.count == 60 {
                gamestate.phase = 0;
                gamestate.count = 0;
            }
        }
    }
}

fn move_background(
    mut query: Query<&mut Transform, With<SkyBackground>>,
) {
    // move sky background
    for mut transform in query.iter_mut() {
        transform.translation.x -= 0.25;
        if transform.translation.x < -4750.0 {
            transform.translation.x = 4750.0;
        }
    }
}

#[cfg(debug_assertions)]
fn check_pause(
    mut state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<&Text>,
) {
    for (interaction, children) in query.iter() {
        match interaction {
            Interaction::Pressed => {
                let text = text_query.get(children[0]).unwrap();
                match text.0.as_str() {
                    "Pause" => {
                        state.set(AppState::Pause);
                        break;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        state.set(AppState::Pause);
    }
}

fn exit(mut commands: Commands, query: Query<Entity, With<InGame>>) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<Fighting>();
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(debug_assertions)]
        app
            // add debug plugin for rapier2d
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(PausePlugin)
            .add_systems(Update, check_pause);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(ControllerPlugin);
        app
            .add_plugins(PlayerPlugin)
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(300.0))
            .insert_resource(GameState::default())
            .add_systems(OnEnter(AppState::Ingame), setup)
            .add_systems(OnExit(AppState::Ingame), exit)
            .add_systems(Update, update_timer.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, check_gameset.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)))
            .add_systems(Update, move_background.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, main_game_system.run_if(in_state(AppState::Ingame)));
    }
}