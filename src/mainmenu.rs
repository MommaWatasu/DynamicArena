use bevy::{
    prelude::*,
};

use crate::{
    GAMETITLE,
    WINDOW_SIZE,
    PATH_FONT,
    AppState
};

const GAMETITLE_FONT_SIZE: f32 = 32.0;
const GAMETITLE_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
const CLICK_START_TEXT: &str = "Click to Start";
const CLICK_START_FONT_SIZE: f32 = 24.0;
const CLICK_START_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const TEXT_PADDING: f32 = 20.0;
const BOARD_SIZE: Vec2 = Vec2::new(280.0, 210.0);
const BOARD_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

#[derive(Component)]
struct Mainmenu;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("mainmenu: setup");
    // game title
    let top = Val::Px(WINDOW_SIZE.y / 2.0 - GAMETITLE_FONT_SIZE / 2.0 - TEXT_PADDING);
    commands.spawn(
        (Text::new(GAMETITLE),
        TextFont {
            font: asset_server.load(PATH_FONT),
            font_size: GAMETITLE_FONT_SIZE,
            ..Default::default()
        },
        TextColor(GAMETITLE_COLOR),
        TextLayout {
            justify: JustifyText::Center,
            ..Default::default()
        },
    ));
}

fn update() {}

pub struct MainmenuPlugin;

impl Plugin for MainmenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Mainmenu), setup)
            .add_systems(Update, update.run_if(in_state(AppState::Mainmenu)));
    }
}