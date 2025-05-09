use bevy::prelude::*;

use crate::{AppState, PATH_BOLD_FONT};

#[derive(Component)]
struct Pause;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("setup");
    commands
        .spawn((
            Node {
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            Pause,
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(50.0),
                        border: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BackgroundColor(Color::WHITE),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Button,
                        Text::new("Resume"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: 50.0,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                    ));
                    builder.spawn((
                        Button,
                        Text::new("Exit"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: 50.0,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                    ));
                });
        });
}

fn check_resume(
    mut state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<(&Interaction, &Text), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, text) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => match text.0.as_str() {
                "Resume" => {
                    state.set(AppState::Ingame);
                }
                "Exit" => {
                    state.set(AppState::Mainmenu);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn exit(mut commands: Commands, query: Query<Entity, With<Pause>>) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Pause), setup)
            .add_systems(OnExit(AppState::Pause), exit)
            .add_systems(Update, check_resume.run_if(in_state(AppState::Pause)));
    }
}
