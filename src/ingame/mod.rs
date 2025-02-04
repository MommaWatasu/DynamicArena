use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::{Indices, PrimitiveTopology}};
use bevy_rapier2d::prelude::*;

#[cfg(debug_assertions)]
mod pause;
mod player;
mod pose;

use crate::{
    AppState, GameConfig, PATH_BOLD_FONT, PATH_BOLD_MONOSPACE_FONT, PATH_EXTRA_BOLD_FONT, PATH_IMAGE_PREFIX
};

#[cfg(debug_assertions)]
use pause::*;
use player::*;

#[derive(Component)]
struct InGame;

#[derive(Component)]
struct GameTimer(f32);

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<GameConfig>,
) {
    info!("ingame: setup");
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
            PlayerID(0),
            HealthBar(100.0),
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
            PlayerID(1),
            HealthBar(100.0),
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
    commands
        .spawn((
            Sprite{
                image: asset_server.load(format!("{}background.png", PATH_IMAGE_PREFIX)),
                custom_size: Some(config.window_size),
                ..Default::default()
            },
            InGame
        ))
        .with_children(|builder| {
            builder.spawn((
                Ground,
                Transform::from_translation(Vec3::new(0.0, 100.0-config.window_size.y / 2.0, 0.0),),
                RigidBody::Fixed,
                Collider::cuboid(config.window_size.x / 2.0, 10.0),
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
                ActiveEvents::COLLISION_EVENTS,
            ));
            spawn_player(0, config.characters_id[0], builder, &mut meshes, &mut materials, 270.0-config.window_size.y / 2.0);
            spawn_player(1, config.characters_id[1], builder, &mut meshes, &mut materials, 270.0-config.window_size.y / 2.0);
        });
}

fn update_timer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut TextColor, &mut GameTimer)>,
) {
    let (mut text, mut color, mut timer) = query.single_mut();
    if timer.0 == 0.0 {
        return;
    }
    timer.0 -= time.delta_secs();
    if timer.0 < 0.0 {
        timer.0 = 0.0;
        gameset(&mut commands, &asset_server);
    } else if timer.0 < 5.0 {
        color.0 = Color::srgb(1.0, 0.0, 0.0);
    }
    text.0 = format!("{:.2}", timer.0);
}

fn gameset(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    info!("ingame: gameset");
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        InGame
    ))
        .with_child((
                Text::new("GAME SET!"),
                TextFont {
                    font: asset_server.load(PATH_EXTRA_BOLD_FONT),
                    font_size: 100.0,
                    ..Default::default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::BLACK),
        ));
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
    info!("ingame: exit");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
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
        app
            .add_plugins(PlayerPlugin)
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(300.0))
            .add_systems(OnEnter(AppState::Ingame), setup)
            .add_systems(OnExit(AppState::Ingame), exit)
            .add_systems(Update, update_timer.run_if(in_state(AppState::Ingame)));
    }
}