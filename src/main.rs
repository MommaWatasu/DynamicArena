use bevy::prelude::*;

mod mainmenu;

const GAMETITLE: &str = "Bevy Game Template";
const WINDOW_SIZE: Vec2 = Vec2::new(800.0, 600.0);
const BACGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
const PATH_FONT: &str = "fonts/DotGothic16-Regular.ttf";
const PATH_IMAGE: &str = "images/";
const PATH_SOUND_BGM: &str = "sounds/bgm.ogg";

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum AppState {
    #[default]
    Mainmenu,
    Ingame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(ClearColor(BACGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_plugins(mainmenu::MainmenuPlugin)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    println!("main: setup");
    // camera
    commands.spawn(Camera2d::default());
    // bgm
    let bgm_sound = asset_server.load(PATH_SOUND_BGM);

    commands.spawn(
        AudioPlayer::new(
            bgm_sound
        )
    );
}