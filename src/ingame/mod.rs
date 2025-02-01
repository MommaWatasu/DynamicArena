use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[cfg(debug_assertions)]
mod pause;
mod player;
mod pose;

use crate::{
    PATH_IMAGE_PREFIX,
    PATH_BOLD_FONT,
    AppState,
    GameConfig
};

#[cfg(debug_assertions)]
use pause::*;
use player::*;

#[derive(Component)]
struct InGame;

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
            spawn_player(0, builder, &mut meshes, &mut materials, 270.0-config.window_size.y / 2.0);
        });
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
            .add_systems(OnExit(AppState::Ingame), exit);
    }
}