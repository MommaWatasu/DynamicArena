use bevy::{color::palettes::basic::GREEN, prelude::*};

#[derive(Component)]
struct Player {
    coord: Vec2,
    speed: Vec2,
    health: u32,
}

#[derive(Component)]
struct Head;

#[derive(Component)]
struct Body;

pub fn spawn_player(
    builder: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    builder.spawn((
        Player {
            coord: Vec2::new(0.0, 0.0),
            speed: Vec2::new(0.0, 0.0),
            health: 100,
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
        Visibility::Visible,
    ))
        .with_children(|builder| {
            builder.spawn((
                Mesh2d(meshes.add(Rectangle::default())),
                MeshMaterial2d(materials.add(Color::from(GREEN))),
                Transform::from_scale(Vec3::splat(128.)),
                BackgroundColor(Color::from(GREEN)),
                Head
            ));
            /*
            builder.spawn((
                Mesh2d(meshes.add(Rectangle::default())),
                MeshMaterial2d(materials.add(Color::from(GREEN))),
                Transform::from_scale(Vec3::splat(128.)),
                Body
            ));
            */
        });
}

fn player_movement() {}

struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, player_movement);
    }
}