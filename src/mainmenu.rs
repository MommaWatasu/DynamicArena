use bevy::prelude::*;

use crate::{
    GAMETITLE,
    //PATH_REGULAR_FONT,
    PATH_BOLD_FONT,
    PATH_EXTRA_BOLD_FONT,
    PATH_IMAGE_PREFIX,
    AppState,
    WindowConfig
};

const GAMETITLE_FONT_SIZE: f32 = 100.0;
const BUTTON_FONT_SIZE: f32 = 50.0;
const GAMETITLE_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
const PATH_SOUND_BGM: &str = "sounds/bgm.ogg";

#[derive(Component)]
struct Mainmenu;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_conf: Res<WindowConfig>,
) {
    println!("mainmenu: setup");

    // show background image
    commands.spawn((
        Sprite{
            image: asset_server.load(format!("{}background.png", PATH_IMAGE_PREFIX)),
            custom_size: Some(window_conf.size.into()),
            ..Default::default()
        },
        Mainmenu
    ));

    // bgm
    commands.spawn((
        AudioPlayer::new(asset_server.load(PATH_SOUND_BGM)),
        PlaybackSettings::LOOP.with_spatial(true),
        GlobalTransform::default(),
        Mainmenu,
    ));

    commands
        .spawn((Node {
            width: Val::Percent(80.0),
            height: Val::Percent(80.0),
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            ..default()
        }, BackgroundColor(Color::srgb(0.1, 0.5, 0.1)),))
        .with_children(|parent| {
            // game title
            parent.spawn(
                    (Text::new(GAMETITLE),
                    TextFont {
                        font: asset_server.load(PATH_EXTRA_BOLD_FONT),
                        font_size: GAMETITLE_FONT_SIZE,
                        ..Default::default()
                    },
                    TextColor(GAMETITLE_COLOR),
                    TextLayout::new_with_justify(JustifyText::Center),
                    // Set the style of the Node itself.
                    Node {
                        align_self: AlignSelf::Center,
                        justify_self: JustifySelf::Center,
                        margin: UiRect{
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Px(0.0),
                            bottom: Val::Percent(20.0),
                        },
                        ..default()
                    },
                    Mainmenu
                ));
            parent.spawn((
                    Button,
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(10.0),
                        border: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::all(Val::Percent(1.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    Mainmenu
                ))
                .with_child((
                    Text::new("Start"),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: BUTTON_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(10.0),
                        border: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::all(Val::Percent(1.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    Mainmenu
                ))
                .with_child((
                    Text::new("Settings"),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: BUTTON_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(10.0),
                        border: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::all(Val::Percent(1.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    Mainmenu
                ))
                .with_child((
                    Text::new("Exit"),
                    TextFont {
                        font: asset_server.load(PATH_BOLD_FONT),
                        font_size: BUTTON_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
        });

}

fn update(
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    text_query: Query<&Text>,
) {
    for (interaction, children) in &mut interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                if children.len() > 0 {
                    let text = text_query.get(children[0]).unwrap();
                    match text.0.as_str() {
                        "Start" => {
                            next_state.set(AppState::ChoosePlayer);
                        }
                        "Settings" => {
                            next_state.set(AppState::Settings);
                        }
                        "Exit" => {
                            app_exit_events.send(AppExit::Success);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

pub struct MainmenuPlugin;

impl Plugin for MainmenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Mainmenu), setup)
            .add_systems(Update, update.run_if(in_state(AppState::Mainmenu)));
    }
}