#[allow(unused_imports)]
use bevy::{
    prelude::*,
    window::{
        Monitor,
        PrimaryWindow,
        WindowMode,
    }
};

mod character_def;
mod choose_character;
#[cfg(not(target_arch = "wasm32"))]
mod connect_controller;
mod ingame;
mod mainmenu;
mod settings;
mod result;

const GAMETITLE: &str = "DynamicArena";
#[cfg(not(target_arch = "wasm32"))]
const TITLE_FONT_SIZE: f32 = 100.0;
#[cfg(target_arch = "wasm32")]
const TITLE_FONT_SIZE: f32 = 20.0;
//const PATH_FONT: &str = "fonts/Orbitron/Orbitron-Regular.ttf";
const PATH_BOLD_FONT: &str = "fonts/Orbitron/Orbitron-Bold.ttf";
const PATH_EXTRA_BOLD_FONT: &str = "fonts/Orbitron/Orbitron-ExtraBold.ttf";
//const PATH_JP_FONT: &str = "fonts/M_PLUS_1p/MPLUS1p-Regular.ttf";
const PATH_BOLD_JP_FONT: &str = "fonts/M_PLUS_1p/MPLUS1p-Bold.ttf";
const PATH_EXTRA_BOLD_JP_FONT: &str = "fonts/M_PLUS_1p/MPLUS1p-ExtraBold.ttf";
const PATH_BOLD_MONOSPACE_FONT: &str = "fonts/Roboto_Condensed/RobotoCondensed-Bold.ttf";
const PATH_IMAGE_PREFIX: &str = "images/";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum GameMode {
    SinglePlayer = 1,
    MultiPlayer = 2,
}

#[derive(Resource)]
struct GameConfig {
    window_size: Vec2,
    mode: GameMode,
    characters_id: [isize; 2],
    sound_volume: f32,
    #[cfg(not(target_arch = "wasm32"))]
    gamepads: [Entity; 2]
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            window_size: Vec2::new(800.0, 600.0),
            mode: GameMode::SinglePlayer,
            characters_id: [-1, -1],
            sound_volume: 1.0,
            #[cfg(not(target_arch = "wasm32"))]
            gamepads: [Entity::from_raw(0), Entity::from_raw(0)]
        }
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum AppState {
    #[default]
    Initialize,
    Mainmenu,
    Settings,
    #[cfg(not(target_arch = "wasm32"))]
    ConnectController,
    ChooseCharacter,
    Ingame,
    Result,
    #[cfg(debug_assertions)]
    Pause
}

fn main() {
    let mut app = App::new();
    #[cfg(not(target_arch = "wasm32"))]
    app
        .add_plugins(connect_controller::ConnectControllerPlugin);
    app
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(GameConfig::default())
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(GlobalVolume::new(0.5))
        
        .add_systems(Startup, setup)
        .add_plugins(mainmenu::MainmenuPlugin)
        .add_plugins(settings::SettingsPlugin)
        .add_plugins(choose_character::ChooseCharacterPlugin)
        .add_plugins(ingame::GamePlugin)
        .add_plugins(result::ResultPlugin)
        .run();
}

#[cfg(not(target_arch = "wasm32"))]
fn setup(
    mut commands: Commands,
    monitors: Query<&Monitor>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut config: ResMut<GameConfig>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    info!("main: setup");
    // assume that there is only one monitor
    let primary_monitor: bool = false;
    for monitor in monitors.iter() {
        if primary_monitor {
            panic!("This Game doesn't support dual monitor!")
        }
        let name = monitor.name.clone().unwrap_or_else(|| "<no name>".into());
        let size = format!("{}x{}px", monitor.physical_height, monitor.physical_width);
        config.window_size = Vec2::new(monitor.physical_width as f32, monitor.physical_height as f32);
        let hz = monitor
            .refresh_rate_millihertz
            .map(|x| format!("{}Hz", x as f32 / 1000.0))
            .unwrap_or_else(|| "<unknown>".into());
        let position = format!(
            "x={} y={}",
            monitor.physical_position.x, monitor.physical_position.y
        );
        let scale = format!("{:.2}", monitor.scale_factor);
        // show monitor info
        info!(
            "Monitor: {} ({}), {} at {}, scale: {}",
            name, size, hz, position, scale
        );
    }
    // set window config
    if let Ok(mut window) = windows.get_single_mut() {
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Primary);
        window.resolution = config.window_size.into();
    }
    // camera
    commands.spawn(Camera2d::default());
    next_state.set(AppState::Mainmenu);
}

#[cfg(target_arch = "wasm32")]
fn setup(
    mut commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut config: ResMut<GameConfig>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    info!("main: setup(wasm)");
    if let Ok(mut window) = windows.get_single_mut() {
        if let Some(win) = web_sys::window() {
            if let Ok(screen) = win.screen() {
                // set window size
                let width = screen.width().unwrap_or(800) as f32;
                let height = screen.height().unwrap_or(600) as f32;
                window.resolution.set(width, height);
                config.window_size = Vec2::new(width, height);
                info!("Set resolution to: {}x{}", width, height);
            }
        }
    }
    // camera
    commands.spawn(Camera2d::default());
    next_state.set(AppState::Mainmenu);
}