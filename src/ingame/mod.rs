use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
};
use bevy_rapier2d::prelude::*;
use pose::{LOSER_POSE, WINNER_POSE};

pub mod agent;
#[cfg(not(target_arch = "wasm32"))]
mod controller;
#[cfg(feature="pause")]
mod pause;
mod player;
mod pose;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(feature="pause")]
use crate::PATH_BOLD_FONT;
use crate::{
    AppState, GameConfig, SoundEffect, PATH_BOLD_MONOSPACE_FONT,
    PATH_EXTRA_BOLD_FONT, PATH_IMAGE_PREFIX, PATH_SOUND_PREFIX, TITLE_FONT_SIZE, DEFAULT_FONT_SIZE
};

use agent::*;
#[cfg(not(target_arch = "wasm32"))]
use controller::*;
#[cfg(feature="pause")]
use pause::*;
use player::*;
#[cfg(target_arch = "wasm32")]
use wasm::*;

const FPS: f32 = 60.0;

#[derive(Resource, Default)]
pub struct GameState {
    pub winners: [u8; 3],
    pub win_types: [bool; 3],
    pub round: u8,
    pub phase: u8,
    pub count: u8,
    pub timer: Timer,
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
struct Fighting(u8);

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

#[derive(Component)]
pub struct SkillName(u8);

#[derive(Component)]
pub struct SkillEntity {
    id: u8,
}

#[derive(Component)]
struct DamageDisplay {
    pub is_red: bool,
    pub alpha: f32,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn rand() -> f32 {
    rand::random::<f32>()
}
#[cfg(target_arch = "wasm32")]
pub fn rand() -> f32 {
    web_sys::js_sys::Math::random() as f32
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<GameState>,
    config: Res<GameConfig>,
) {
    info!("setup");
    #[cfg(feature="pause")]
    info!("pause feature enabled");
    #[cfg(feature="pause")]
    commands
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
            InGame,
        ))
        .with_child((
            Text::new("Pause"),
            TextFont {
                font: asset_server.load(PATH_BOLD_FONT),
                font_size: DEFAULT_FONT_SIZE,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::BLACK),
        ));

    // timer box
    commands
        .spawn((
            InGame,
            Node {
                #[cfg(not(target_arch = "wasm32"))]
                width: Val::Px(300.0),
                #[cfg(target_arch = "wasm32")]
                width: Val::Px(150.0),
                #[cfg(not(target_arch = "wasm32"))]
                height: Val::Px(100.0),
                #[cfg(target_arch = "wasm32")]
                height: Val::Px(50.0),
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
                    font_size: DEFAULT_FONT_SIZE,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::WHITE),
            ));
        });
    // damage display
    commands.spawn((
        InGame,
        Node {
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Start,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            height: Val::Percent(20.0),
            ..default()
        },
    ))
        .with_children(|builder|{
            builder.spawn((
                Node {
                    justify_self: JustifySelf::Start,
                    align_self: AlignSelf::End,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(20.0),
                    height: Val::Px(100.0),
                    margin: UiRect::horizontal(Val::Px(100.0)),
                    ..default()
                },
            ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("Damage"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_MONOSPACE_FONT),
                            font_size: TITLE_FONT_SIZE,
                            ..default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::srgba(0.0, 0.0, 5.0, 0.0)),
                        DamageDisplay {
                            is_red: false,
                            alpha: 0.0,
                        },
                        PlayerID(0),
                    ));
                });
            builder.spawn((
                Node {
                    justify_self: JustifySelf::End,
                    align_self: AlignSelf::End,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(20.0),
                    height: Val::Px(100.0),
                    margin: UiRect::horizontal(Val::Px(100.0)),
                    ..default()
                },
            ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("Damage"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_MONOSPACE_FONT),
                            font_size: TITLE_FONT_SIZE,
                            ..default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::srgba(0.0, 0.0, 5.0, 0.0)),
                        DamageDisplay {
                            is_red: true,
                            alpha: 0.0,
                        },
                        PlayerID(1),
                    ));
                });
        });

    // health bar for player 1
    commands.spawn((
        InGame,
        PlayerID(0),
        #[cfg(not(target_arch = "wasm32"))]
        HealthBar(1.0, 250.0 - config.window_size.x / 2.0),
        #[cfg(target_arch = "wasm32")]
        HealthBar(1.0, 125.0 - config.window_size.x / 2.0),
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [300.0 - config.window_size.x / 2.0, 0.0, 1.0],
                        [250.0 - config.window_size.x / 2.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 10.0, 0.0, 1.0],
                        [0.0, 10.0, 0.0, 1.0],
                        [0.0, 10.0, 0.0, 0.5],
                        [0.0, 10.0, 0.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [150.0 - config.window_size.x / 2.0, 0.0, 1.0],
                        [125.0 - config.window_size.x / 2.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 1.0, 0.0, 1.0],
                        [0.0, 1.0, 0.0, 1.0],
                        [0.0, 1.0, 0.0, 0.5],
                        [0.0, 1.0, 0.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(-150.0, config.window_size.y / 2.0 - 30.0, 1.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(-75.0, config.window_size.y / 2.0 - 25.0, 1.0)),
    ));
    // health bar for player 2
    commands.spawn((
        InGame,
        PlayerID(1),
        #[cfg(not(target_arch = "wasm32"))]
        HealthBar(1.0, config.window_size.x / 2.0 - 250.0),
        #[cfg(target_arch = "wasm32")]
        HealthBar(1.0, config.window_size.x / 2.0 - 125.0),
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [config.window_size.x / 2.0 - 300.0, 0.0, 1.0],
                        [config.window_size.x / 2.0 - 250.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 10.0, 0.0, 1.0],
                        [0.0, 10.0, 0.0, 1.0],
                        [0.0, 10.0, 0.0, 0.5],
                        [0.0, 10.0, 0.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [config.window_size.x / 2.0 - 150.0, 0.0, 1.0],
                        [config.window_size.x / 2.0 - 125.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 1.0, 0.0, 1.0],
                        [0.0, 1.0, 0.0, 1.0],
                        [0.0, 1.0, 0.0, 0.5],
                        [0.0, 1.0, 0.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(150.0, config.window_size.y / 2.0 - 30.0, 1.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(75.0, config.window_size.y / 2.0 - 25.0, 1.0)),
    ));

    // energy bar for player 1
    commands.spawn((
        InGame,
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [400.0 - config.window_size.x / 2.0, 0.0, 1.0],
                        [350.0 - config.window_size.x / 2.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 10.0, 0.5],
                        [0.0, 0.0, 10.0, 0.5],
                        [0.0, 0.0, 10.0, 0.5],
                        [0.0, 0.0, 10.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [200.0 - config.window_size.x / 2.0, 0.0, 1.0],
                        [175.0 - config.window_size.x / 2.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(-150.0, config.window_size.y / 2.0 - 60.0, 1.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(-75.0, config.window_size.y / 2.0 - 50.0, 1.0)),
    ));
    commands.spawn((
        InGame,
        PlayerID(0),
        #[cfg(not(target_arch = "wasm32"))]
        EnergyBar(0.0, 350.0 - config.window_size.x / 2.0),
        #[cfg(target_arch = "wasm32")]
        EnergyBar(0.0, 175.0 - config.window_size.x / 2.0),
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [10.0, 0.0, 1.0],
                        [10.0, 20.0, 1.0],
                        [50.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 10.0, 1.0],
                        [0.0, 0.0, 10.0, 1.0],
                        [0.0, 0.0, 10.0, 1.0],
                        [0.0, 0.0, 10.0, 1.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [-25.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 1.0],
                        [0.0, 0.0, 1.0, 1.0],
                        [0.0, 0.0, 1.0, 1.0],
                        [0.0, 0.0, 1.0, 1.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(-150.0, config.window_size.y / 2.0 - 60.0, 2.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(-75.0, config.window_size.y / 2.0 - 50.0, 2.0)),
    ));
    // energy bar for player 2
    commands.spawn((
        InGame,
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [config.window_size.x / 2.0 - 400.0, 0.0, 1.0],
                        [config.window_size.x / 2.0 - 350.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [config.window_size.x / 2.0 - 200.0, 0.0, 1.0],
                        [config.window_size.x / 2.0 - 175.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(150.0, config.window_size.y / 2.0 - 60.0, 1.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(75.0, config.window_size.y / 2.0 - 50.0, 1.0)),
    ));
    commands.spawn((
        InGame,
        PlayerID(1),
        #[cfg(not(target_arch = "wasm32"))]
        EnergyBar(0.0, config.window_size.x / 2.0 - 350.0),
        #[cfg(target_arch = "wasm32")]
        EnergyBar(0.0, config.window_size.x / 2.0 - 175.0),
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [-10.0, 0.0, 1.0],
                        [-10.0, 20.0, 1.0],
                        [-50.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 10.0, 1.0],
                        [0.0, 0.0, 10.0, 1.0],
                        [0.0, 0.0, 10.0, 1.0],
                        [0.0, 0.0, 10.0, 1.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [-25.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 1.0],
                        [0.0, 0.0, 1.0, 1.0],
                        [0.0, 0.0, 1.0, 1.0],
                        [0.0, 0.0, 1.0, 1.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(150.0, config.window_size.y / 2.0 - 60.0, 2.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(75.0, config.window_size.y / 2.0 - 50.0, 2.0)),
    ));

    // fire charge bar for player 1
    commands.spawn((
        InGame,
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [500.0 - config.window_size.x / 2.0, 0.0, 1.0],
                        [450.0 - config.window_size.x / 2.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [200.0 - config.window_size.x / 2.0, 0.0, 1.0],
                        [175.0 - config.window_size.x / 2.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(-150.0, config.window_size.y / 2.0 - 90.0, 1.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(-75.0, config.window_size.y / 2.0 - 50.0, 1.0)),
    ));
    commands.spawn((
        InGame,
        PlayerID(0),
        #[cfg(not(target_arch = "wasm32"))]
        FireBar(1.0, 450.0 - config.window_size.x / 2.0),
        #[cfg(target_arch = "wasm32")]
        FireBar(1.0, 175.0 - config.window_size.x / 2.0),
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [10.0, 0.0, 1.0],
                        [10.0, 20.0, 1.0],
                        [500.0 - config.window_size.x / 2.0, 0.0, 1.0],
                        [450.0 - config.window_size.x / 2.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [20.0, 0.0, 0.0, 1.0],
                        [20.0, 0.0, 0.0, 1.0],
                        [20.0, 0.0, 0.0, 1.0],
                        [20.0, 0.0, 0.0, 1.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [-25.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [1.0, 0.0, 0.0, 1.0],
                        [1.0, 0.0, 0.0, 1.0],
                        [1.0, 0.0, 0.0, 1.0],
                        [1.0, 0.0, 0.0, 1.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(-150.0, config.window_size.y / 2.0 - 90.0, 2.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(-75.0, config.window_size.y / 2.0 - 50.0, 2.0)),
    ));
    
    // fire charge bar for player 2
    commands.spawn((
        InGame,
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [config.window_size.x / 2.0 - 500.0, 0.0, 1.0],
                        [config.window_size.x / 2.0 - 450.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [config.window_size.x / 2.0 - 200.0, 0.0, 1.0],
                        [config.window_size.x / 2.0 - 175.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                        [0.0, 0.0, 1.0, 0.5],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(150.0, config.window_size.y / 2.0 - 90.0, 1.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(75.0, config.window_size.y / 2.0 - 50.0, 1.0)),
    ));
    commands.spawn((
        InGame,
        PlayerID(1),
        #[cfg(not(target_arch = "wasm32"))]
        FireBar(1.0, config.window_size.x / 2.0 - 450.0),
        #[cfg(target_arch = "wasm32")]
        FireBar(1.0, config.window_size.x / 2.0 - 175.0),
        #[cfg(not(target_arch = "wasm32"))]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [-10.0, 0.0, 1.0],
                        [-10.0, 20.0, 1.0],
                        [config.window_size.x / 2.0 - 500.0, 0.0, 1.0],
                        [config.window_size.x / 2.0 - 450.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [20.0, 0.0, 0.0, 1.0],
                        [20.0, 0.0, 0.0, 1.0],
                        [20.0, 0.0, 0.0, 1.0],
                        [20.0, 0.0, 0.0, 1.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        #[cfg(target_arch = "wasm32")]
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                        [-25.0, 0.0, 1.0],
                        [0.0, 20.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [1.0, 0.0, 0.0, 1.0],
                        [1.0, 0.0, 0.0, 1.0],
                        [1.0, 0.0, 0.0, 1.0],
                        [1.0, 0.0, 0.0, 1.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(150.0, config.window_size.y / 2.0 - 90.0, 2.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(75.0, config.window_size.y / 2.0 - 50.0, 2.0)),
    ));

    // skill name display
    commands.spawn((
        InGame,
        Visibility::Hidden,
        Node {
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            ..default()
        },
        SkillName(0),
        ImageNode::new(asset_server.load(format!("{}skill_name1.png", PATH_IMAGE_PREFIX))),
    ));
    commands.spawn((
        InGame,
        Visibility::Hidden,
        Node {
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            ..default()
        },
        SkillName(1),
        ImageNode::new(asset_server.load(format!("{}skill_name2.png", PATH_IMAGE_PREFIX))),
    ));
    commands.spawn((
        InGame,
        Visibility::Hidden,
        Node {
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            ..default()
        },
        SkillName(2),
        ImageNode::new(asset_server.load(format!("{}skill_name3.png", PATH_IMAGE_PREFIX))),
    ));

    // curtain for skill
    commands.spawn((
        Mesh2d(
            meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [
                            -config.window_size.x / 2.0,
                            config.window_size.y / 2.0,
                            10.0,
                        ],
                        [config.window_size.x / 2.0, config.window_size.y / 2.0, 10.0],
                        [
                            -config.window_size.x / 2.0,
                            -config.window_size.y / 2.0,
                            10.0,
                        ],
                        [
                            config.window_size.x / 2.0,
                            -config.window_size.y / 2.0,
                            10.0,
                        ],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        [0.0, 0.0, 0.0, 0.0],
                        [0.0, 0.0, 0.0, 0.0],
                        [0.0, 0.0, 0.0, 0.0],
                        [0.0, 0.0, 0.0, 0.0],
                    ],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 1, 2, 3])),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        SkillEntity { id: 1 },
        Transform::from_translation(Vec3::new(0.0, 0.0, 19.0)),
    ));

    // thunder for skill of character 0
    commands.spawn((
        InGame,
        SkillEntity { id: 0 },
        Visibility::Hidden,
        Sprite {
            image: asset_server.load(format!("{}thunder.png", PATH_IMAGE_PREFIX)),
            custom_size: Some(Vec2::new(250.0, 1000.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 100.0, 20.0)),
    ));

    // sky background
    commands.spawn((
        #[cfg(not(target_arch = "wasm32"))]
        Sprite {
            image: asset_server.load(format!("{}sky_upscaled.png", PATH_IMAGE_PREFIX)),
            ..default()
        },
        #[cfg(target_arch = "wasm32")]
        Sprite {
            image: asset_server.load(format!("{}web/sky_original.png", PATH_IMAGE_PREFIX)),
            ..default()
        },
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(0.0, 100.0, -2.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(0.0, 50.0, -2.0)),
        SkyBackground,
        InGame,
    ));
    commands.spawn((
        #[cfg(not(target_arch = "wasm32"))]
        Sprite {
            image: asset_server.load(format!("{}sky_upscaled.png", PATH_IMAGE_PREFIX)),
            flip_x: true,
            ..default()
        },
        #[cfg(target_arch = "wasm32")]
        Sprite {
            image: asset_server.load(format!("{}web/sky_original.png", PATH_IMAGE_PREFIX)),
            flip_x: true,
            ..default()
        },
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(4800.0, 100.0, -2.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(1200.0, 50.0, -2.0)),
        SkyBackground,
        InGame,
    ));

    // background
    commands.spawn((
        Sprite {
            #[cfg(not(target_arch = "wasm32"))]
            image: asset_server.load(format!("{}background.png", PATH_IMAGE_PREFIX)),
            #[cfg(target_arch = "wasm32")]
            image: asset_server.load(format!("{}web/background.png", PATH_IMAGE_PREFIX)),
            ..default()
        },
        BackGround,
        #[cfg(not(target_arch = "wasm32"))]
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        #[cfg(target_arch = "wasm32")]
        Transform::from_translation(Vec3::new(0.0, 70.0, -1.0)),
        InGame,
    ));
    if cfg!(target_arch = "wasm32") {
        spawn_player(
            0,
            config.characters_id[0],
            &mut commands,
            &mut meshes,
            &mut materials,
            135.0 - config.window_size.y / 2.0,
        );
        spawn_player(
            1,
            config.characters_id[1],
            &mut commands,
            &mut meshes,
            &mut materials,
            135.0 - config.window_size.y / 2.0,
        );
    } else {
        spawn_player(
            0,
            config.characters_id[0],
            &mut commands,
            &mut meshes,
            &mut materials,
            270.0 - config.window_size.y / 2.0,
        );
        spawn_player(
            1,
            config.characters_id[1],
            &mut commands,
            &mut meshes,
            &mut materials,
            270.0 - config.window_size.y / 2.0,
        );
    }

    // create controller circle
    #[cfg(target_arch = "wasm32")]
    commands
        .spawn((
            InGame,
            Mesh2d(meshes.add(Circle::new(CONTROLLER_CIRCLE_RADIUS))),
            MeshMaterial2d(materials.add(Color::srgba(1.0, 1.0, 1.0, 0.4))),
            Transform::from_translation(Vec3::new(
                -config.window_size.x / 2.0 + 100.0,
                -config.window_size.y / 4.0,
                20.0,
            )),
        ))
        .with_children(|builder| {
            builder.spawn((
                ControllerCircle,
                Mesh2d(meshes.add(Circle::new(CONTROLLER_CIRCLE_RADIUS / 3.0))),
                MeshMaterial2d(materials.add(Color::srgba(1.0, 1.0, 1.0, 1.0))),
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ));
        });

    game_state.phase = 0;
    game_state.count = 0;
    game_state.round = 1;
    game_state.timer = Timer::from_seconds(1.0 / FPS, TimerMode::Repeating);
}

fn update_timer(
    time: Res<Time>,
    fighting: ResMut<Fighting>,
    mut gamestate: ResMut<GameState>,
    mut timer_query: Query<(&mut Text, &mut TextColor, &mut GameTimer)>,
    health_bar_query: Query<(&HealthBar, &PlayerID)>,
) {
    if fighting.0 != 0 {
        return;
    }
    let (mut text, mut color, mut timer) = timer_query.single_mut();
    if timer.0 == 0.0 {
        return;
    }
    timer.0 -= time.delta_secs();
    if timer.0 < 0.0 {
        timer.0 = 0.0;
        let bars: Vec<_> = health_bar_query.iter().collect();
        let winner_id = if bars[0].0 .0 < bars[1].0 .0 {
            bars[1].1 .0 + 1
        } else if bars[0].0 .0 == bars[1].0 .0 {
            0
        } else {
            bars[0].1 .0 + 1
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
    fighting: ResMut<Fighting>,
    player_query: Query<(&Player, &PlayerID)>
) {
    if fighting.0 != 0 {
        return;
    }
    for (player, player_id) in player_query.iter() {
        if player.health == 0 {
            let round = gamestate.round as usize - 1;
            gamestate.winners[round] = if player_id.0 == 0 { 2 } else { 1 };
            gamestate.win_types[round] = true;
            if gamestate.phase == 5 {
                gamestate.count = 0;
                gamestate.phase = 6;
            }
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
    mut status_bar_query: Query<(&mut BackgroundColor, &mut Text, &mut TextColor), With<StatusBar>>,
    mut curtain_query: Query<&mut BackgroundColor, (With<Curtain>, Without<StatusBar>)>,
    mut background_query: Query<&mut Transform, (With<BackGround>, Without<Player>, Without<Foot>)>,
    mut player_query: Query<
        (&PlayerID, &mut Player, &mut Transform),
        (Without<BackGround>, Without<Foot>),
    >,
    mut foot_query: Query<&mut Transform, (With<Foot>, Without<Player>, Without<BackGround>)>,
    mut health_query: Query<(&mut HealthBar, &mut Mesh2d, &PlayerID), Without<FireBar>>,
    mut fire_query: Query<(&mut FireBar, &mut Mesh2d, &PlayerID), Without<HealthBar>>,
    mut timer_query: Query<(&mut Text, &mut TextColor, &mut GameTimer), Without<StatusBar>>,
    sound_query: Query<Entity, With<SoundEffect>>,
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
                        font_size: TITLE_FONT_SIZE,
                        ..default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                ));
                commands.spawn((
                    AudioPlayer::new(asset_server.load(format!(
                        "{}/round{}.ogg",
                        PATH_SOUND_PREFIX, gamestate.round
                    ))),
                    SoundEffect,
                ));
            } else {
                let (mut bar, mut text, mut text_color) = status_bar_query.single_mut();
                bar.0 = Color::srgba(0.0, 0.0, 0.0, 0.8);
                text.0 = format!("ROUND {}", gamestate.round);
                commands.spawn((
                    AudioPlayer::new(asset_server.load(format!(
                        "{}/round{}.ogg",
                        PATH_SOUND_PREFIX, gamestate.round
                    ))),
                    SoundEffect,
                ));
                text_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.8);
            }
            gamestate.phase = 1;
            gamestate.count = 0;
        } else if gamestate.phase == 1 {
            gamestate.count += 1;
            if gamestate.count == 60 {
                let (_, mut text, _) = status_bar_query.single_mut();
                text.0 = "READY?".to_string();
                // TODO: I have to think about how to handle spawned Audio Player entity
                commands.spawn((
                    AudioPlayer::new(asset_server.load(format!("{}/ready.ogg", PATH_SOUND_PREFIX))),
                    SoundEffect,
                ));
                gamestate.phase = 2;
                gamestate.count = 0;
            }
        } else if gamestate.phase == 2 {
            gamestate.count += 1;
            if gamestate.count == 90 {
                let (_, mut text, _) = status_bar_query.single_mut();
                text.0 = "FIGHT!".to_string();
                commands.spawn((
                    AudioPlayer::new(asset_server.load(format!("{}/fight.ogg", PATH_SOUND_PREFIX))),
                    SoundEffect,
                ));
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
            let (mut bar, _, mut text_color) = status_bar_query.single_mut();
            bar.0 = Color::srgba(0.0, 0.0, 0.0, 0.8 - gamestate.count as f32 / 60.0);
            text_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.8 - gamestate.count as f32 / 60.0);
            if gamestate.count == 48 {
                commands.insert_resource(Fighting(0));
                gamestate.phase = 5;
                gamestate.count = 0;
            }
        } else if gamestate.phase == 6 {
            if gamestate.count == 0 {
                let (mut bar, mut text, mut text_color) = status_bar_query.single_mut();
                bar.0 = Color::srgba(0.0, 0.0, 0.0, 0.8);
                text.0 = if gamestate.win_types[gamestate.round as usize - 1] {
                    "KO!".to_string()
                } else {
                    "TIME UP!".to_string()
                };
                if gamestate.win_types[gamestate.round as usize - 1] {
                    commands.spawn((
                        AudioPlayer::new(
                            asset_server.load(format!("{}/KO.ogg", PATH_SOUND_PREFIX)),
                        ),
                        SoundEffect,
                    ));
                } else {
                    commands.spawn((
                        AudioPlayer::new(
                            asset_server.load(format!("{}/timeup.ogg", PATH_SOUND_PREFIX)),
                        ),
                        SoundEffect,
                    ));
                }
                text_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.8);
                // winner and loser animation
                if !gamestate.win_types[gamestate.round as usize - 1] {
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
            let (_, mut text, _) = status_bar_query.single_mut();
            let winner_id = gamestate.winners[gamestate.round as usize - 1];
            if winner_id == 0 {
                text.0 = "DRAW".to_string();
                commands.spawn((
                    AudioPlayer::new(asset_server.load(format!("{}/draw.ogg", PATH_SOUND_PREFIX))),
                    SoundEffect,
                ));
            } else {
                text.0 = format!("Player {} WIN", winner_id);
                commands.spawn((
                    AudioPlayer::new(
                        asset_server
                            .load(format!("{}/player{}_win.ogg", PATH_SOUND_PREFIX, winner_id)),
                    ),
                    SoundEffect,
                ));
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
            let mut curtain = curtain_query.single_mut();
            curtain.0 = Color::srgba(0.0, 0.0, 0.0, gamestate.count as f32 / 60.0);
            if gamestate.count == 60 {
                gamestate.round += 1;
                if gamestate.round == 4 {
                    // change app state to show result
                    next_state.set(AppState::Result);
                } else {
                    gamestate.phase = 11;
                    gamestate.count = 0;

                    // remove status bar
                    let (mut bar, _, mut text_color) = status_bar_query.single_mut();
                    bar.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
                    text_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
                    // reset background
                    background_query.single_mut().translation.x = 0.0;
                    // reset player
                    for (id, mut player, mut transform) in player_query.iter_mut() {
                        player.reset(id);
                        if cfg!(target_arch = "wasm32") {
                            transform.translation.x = if id.0 == 0 { -250.0 } else { 250.0 };
                            transform.translation.y = 135.0 - config.window_size.y / 2.0;
                        } else {
                            transform.translation.x = if id.0 == 0 { -500.0 } else { 500.0 };
                            transform.translation.y = 270.0 - config.window_size.y / 2.0;
                        }
                    }
                    for mut foot_transform in foot_query.iter_mut() {
                        foot_transform.translation.x = 0.0;
                        if cfg!(target_arch = "wasm32") {
                            foot_transform.translation.y = -20.0;
                        } else {
                            foot_transform.translation.y = -40.0;
                        }
                    }

                    // reset health bar
                    for (mut health_bar, mesh_handler, health_id) in health_query.iter_mut() {
                        health_bar.0 = 1.0;
                        if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                            if let Some(VertexAttributeValues::Float32x3(ref mut positions)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
                            {
                                positions[3][0] = health_bar.1 * health_bar.0;
                                if cfg!(target_arch = "wasm32") {
                                    positions[2][0] = health_bar.1 * health_bar.0
                                        + if health_id.0 == 0 { 25.0 } else { -25.0 };
                                } else {
                                    positions[2][0] = health_bar.1 * health_bar.0
                                        + if health_id.0 == 0 { 50.0 } else { -50.0 };
                                }
                            }
                        }
                    }
                    // reset fire bar
                    for (mut fire_bar, mesh_handler, fire_id) in fire_query.iter_mut() {
                        fire_bar.0 = 1.0;
                        if let Some(mesh) = meshes.get_mut(mesh_handler.id()) {
                            if let Some(VertexAttributeValues::Float32x3(ref mut positions)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
                            {
                                positions[3][0] = fire_bar.1 * fire_bar.0;
                                if cfg!(target_arch = "wasm32") {
                                    positions[2][0] = fire_bar.1 * fire_bar.0
                                        + if fire_id.0 == 0 { 25.0 } else { -25.0 };
                                } else {
                                    positions[2][0] = fire_bar.1 * fire_bar.0
                                        + if fire_id.0 == 0 { 50.0 } else { -50.0 };
                                }
                            }
                            if let Some(VertexAttributeValues::Float32x4(ref mut colors)) =
                                mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                            {
                                for i in 0..4 {
                                    colors[i][0] = 20.0;
                                }
                            }
                        }
                    }
                    // reset timer
                    let (mut text, mut color, mut timer) = timer_query.single_mut();
                    timer.0 = 60.0;
                    text.0 = "60.00".to_string();
                    color.0 = Color::WHITE;
                    // reset audio player(unused sound effect entity)
                    for entity in sound_query.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        } else if gamestate.phase == 11 {
            gamestate.count += 1;
            let mut curtain = curtain_query.single_mut();
            curtain.0 = Color::srgba(0.0, 0.0, 0.0, 1.0 - gamestate.count as f32 / 60.0);
            if gamestate.count == 60 {
                gamestate.phase = 0;
                gamestate.count = 0;
            }
        }
    }
}

fn move_background(mut query: Query<&mut Transform, With<SkyBackground>>) {
    // move sky background
    for mut transform in query.iter_mut() {
        transform.translation.x -= 0.25;
        if cfg!(target_arch = "wasm32") {
            if transform.translation.x < -1150.0 {
                transform.translation.x = 1150.0;
            }
        } else {
            if transform.translation.x < -4750.0 {
                transform.translation.x = 4750.0;
            }
        }
    }
}

#[cfg(feature="pause")]
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
        // add debug plugin for rapier2d
        app.add_plugins(RapierDebugRenderPlugin::default());
        #[cfg(feature="pause")]
        app
            .add_plugins(PausePlugin)
            .add_systems(Update, check_pause);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(ControllerPlugin);

        #[cfg(target_arch = "wasm32")]
        app.insert_resource(TouchState {
            start_position: Vec2::ZERO,
            id: u64::MAX,
        })
        .insert_resource(DoubleJumpCheck::new())
        .add_systems(
            Update,
            touch_input.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
        );

        app.add_plugins(PlayerPlugin)
            .add_plugins(AgentPlugin)
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(300.0))
            .insert_resource(GameState::default())
            .add_systems(OnEnter(AppState::Ingame), setup)
            .add_systems(OnExit(AppState::Ingame), exit)
            .add_systems(
                Update,
                update_timer.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
            )
            .add_systems(
                Update,
                check_gameset.run_if(in_state(AppState::Ingame).and(resource_exists::<Fighting>)),
            )
            .add_systems(Update, move_background.run_if(in_state(AppState::Ingame)))
            .add_systems(Update, main_game_system.run_if(in_state(AppState::Ingame)));
    }
}
