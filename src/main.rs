#[allow(unused_imports)]
use bevy::{
    prelude::*,
    window::{
        Monitor,
        PrimaryWindow,
        WindowMode,
    }
};

mod mainmenu;

const GAMETITLE: &str = "DynamicArena";
const BACGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
const PATH_REGULAR_FONT: &str = "fonts/static/Orbitron-Regular.ttf";
const PATH_BOLD_FONT: &str = "fonts/static/Orbitron-Bold.ttf";
const PATH_EXTRA_BOLD_FONT: &str = "fonts/static/Orbitron-ExtraBold.ttf";
const PATH_IMAGE_PREFIX: &str = "images/";

#[derive(Resource)]
struct WindowConfig {
    size: Vec2
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum AppState {
    #[default]
    Initialize,
    Mainmenu,
    Settings,
    ChoosePlayer,
    Ingame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(ClearColor(BACGROUND_COLOR))
        .insert_resource(WindowConfig {
            size: Vec2::new(800.0, 600.0),
        })
        
        .add_systems(Startup, setup)
        .add_plugins(mainmenu::MainmenuPlugin)
        .run();
}

#[cfg(not(target_arch = "wasm32"))]
fn setup(
    mut commands: Commands,
    monitors: Query<&Monitor>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut window_conf: ResMut<WindowConfig>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    println!("main: setup");
    // assume that there is only one monitor
    let primary_monitor: bool = false;
    for monitor in monitors.iter() {
        if primary_monitor {
            panic!("This Game doesn't support dual monitor!")
        }
        let name = monitor.name.clone().unwrap_or_else(|| "<no name>".into());
        let size = format!("{}x{}px", monitor.physical_height, monitor.physical_width);
        window_conf.size = Vec2::new(monitor.physical_width as f32, monitor.physical_height as f32);
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
        println!(
            "Monitor: {} ({}), {} at {}, scale: {}",
            name, size, hz, position, scale
        );
    }
    // set window config
    if let Ok(mut window) = windows.get_single_mut() {
        window.mode = WindowMode::Fullscreen(MonitorSelection::Primary);
        window.resolution = window_conf.size.into();
    }
    // camera
    commands.spawn(Camera2d::default());
    next_state.set(AppState::Mainmenu);
}

#[cfg(target_arch = "wasm32")]
fn setup(
    mut commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut window_conf: ResMut<WindowConfig>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    println!("main: setup(wasm)");
    if let Ok(mut window) = windows.get_single_mut() {
        if let Some(win) = web_sys::window() {
            if let Ok(screen) = win.screen() {
                // set window size
                let width = screen.width().unwrap_or(800) as f32;
                let height = screen.height().unwrap_or(600) as f32;
                window.resolution.set(width, height);
                window_conf.size = Vec2::new(width, height);
                info!("Set resolution to: {}x{}", width, height);
            }
        }
    }
    // camera
    commands.spawn(Camera2d::default());
    next_state.set(AppState::Mainmenu);
}