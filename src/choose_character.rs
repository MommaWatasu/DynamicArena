use bevy::prelude::*;

use crate::{
    character_def::*, AppState, GameConfig, DEFAULT_FONT_SIZE, PATH_BOLD_FONT, PATH_BOLD_JP_FONT, PATH_EXTRA_BOLD_JP_FONT, PATH_IMAGE_PREFIX, TITLE_FONT_SIZE
};

#[derive(Component)]
struct ChooseCharacter;

#[derive(Component)]
struct CharacterID(isize);

pub struct ChooseCharacterPlugin;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("setup");
    commands.spawn((
        #[cfg(not(target_arch = "wasm32"))]
        ImageNode::new(asset_server.load(format!("{}background_mainmenu.png", PATH_IMAGE_PREFIX))),
        #[cfg(target_arch = "wasm32")]
        ImageNode::new(asset_server.load(format!("{}web/background_mainmenu.png", PATH_IMAGE_PREFIX))),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ChooseCharacter
    ))
        .with_children(|builder| {
            builder.spawn(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(10.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                }
            )
                .with_children(|builder| {
                    builder.spawn((
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
                    ))
                    .with_child((
                        Text::new("<Back"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: DEFAULT_FONT_SIZE,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::BLACK),
                    ));
                    builder.spawn((
                        Button,
                        Node {
                            justify_self: JustifySelf::End,
                            align_self: AlignSelf::Start,
                            #[cfg(not(target_arch = "wasm32"))]
                            border: UiRect::all(Val::Px(5.0)),
                            #[cfg(target_arch = "wasm32")]
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderRadius::MAX,
                        BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                    ))
                    .with_child((
                        Text::new("Next>"),
                        TextFont {
                            font: asset_server.load(PATH_BOLD_FONT),
                            font_size: DEFAULT_FONT_SIZE,
                            ..Default::default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                    ));
                });
            builder.spawn(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    ..default()
                },
            )
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("キャラクターを選んでください"),
                        TextFont {
                            font: asset_server.load(PATH_EXTRA_BOLD_JP_FONT),
                            font_size: TITLE_FONT_SIZE,
                            ..Default::default()
                        },
                        TextColor(Color::BLACK),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                    builder.spawn((
                        Node {
                            width: Val::Percent(90.0),
                            height: Val::Percent(90.0),
                            flex_direction: FlexDirection::Row,
                            align_self: AlignSelf::Center,
                            justify_self: JustifySelf::Center,
                            align_items: AlignItems::Center,
                            justify_items: JustifyItems::Center,
                            justify_content: JustifyContent::SpaceEvenly,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                        BorderRadius::all(Val::Px(20.0)),
                    ))
                        .with_children(|builder| {
                            for i in 0..3 {
                                create_character_box(builder, &asset_server, i as isize);
                            }
                        });
                });
        });
}

fn create_character_box(
    builder: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    character_id: isize,
) {
    let profile = &CHARACTER_PROFILES[character_id as usize];
    builder.spawn((
        Node {
            width: Val::Percent(30.0),
            height: Val::Percent(90.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Button,
        CharacterID(character_id),
        BorderRadius::all(Val::Px(20.0)),
        BackgroundColor(Color::srgba(0.6, 0.8, 0.9, 0.8)),
    )).with_children(|builder| {
        builder.spawn((
            Text::new(profile.name),
            TextFont {
                font: asset_server.load(PATH_BOLD_FONT),
                #[cfg(not(target_arch = "wasm32"))]
                font_size: 40.0,
                #[cfg(target_arch = "wasm32")]
                font_size: 15.0,
                ..Default::default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::BLACK),
        ));
        builder.spawn((
            Text::new(profile.description),
            TextFont {
                font: asset_server.load(PATH_BOLD_JP_FONT),
                #[cfg(not(target_arch = "wasm32"))]
                font_size: 30.0,
                #[cfg(target_arch = "wasm32")]
                font_size: 10.0,
                ..Default::default()
            },
            TextLayout::new_with_justify(JustifyText::Left),
            TextColor(Color::BLACK),
        ));
        builder.spawn((
            Text::new(format!("<スキル> {}\n{}", profile.skill_name, profile.skill_description)),
            TextFont {
                font: asset_server.load(PATH_BOLD_JP_FONT),
                #[cfg(not(target_arch = "wasm32"))]
                font_size: 30.0,
                #[cfg(target_arch = "wasm32")]
                font_size: 10.0,
                ..Default::default()
            },
            TextLayout::new_with_justify(JustifyText::Left),
            TextColor(Color::BLACK),
        ));
        builder.spawn((
            ImageNode::new(asset_server.load(format!("{}character_{}_chart.png", PATH_IMAGE_PREFIX, character_id))),
        ));
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn choose_rand_character(id: isize) -> isize {
    use rand::seq::IteratorRandom;
    let mut available_nums = vec![0, 1, 2];
    available_nums.retain(|&x| x != id);
    available_nums.iter().choose(&mut rand::rng()).unwrap().clone()
}

#[cfg(target_arch = "wasm32")]
fn choose_rand_character(id: isize) -> isize {
    let mut available_nums = vec![0, 1, 2];
    available_nums.retain(|&x| x != id);
    let random_index = (web_sys::js_sys::Math::random() * 2.0).floor() as usize;
    available_nums[random_index]
}

fn update(
    button_query: Query<(&Interaction, &CharacterID), (Changed<Interaction>, With<Button>)>,
    mut transitin_query: Query<(&Children, &mut BorderColor), (With<Button>, Without<CharacterID>)>,
    mut text_query: Query<(&Text, &mut TextColor)>,
    mut config: ResMut<GameConfig>,
) {
    for (interaction, id) in button_query.iter() {
        match interaction {
            &Interaction::Pressed => {
                if config.characters_id[0] == id.0 {
                    config.characters_id = [-1, -1];
                    for (children, mut bc) in transitin_query.iter_mut() {
                        let mut text = text_query.get_mut(children[0]).unwrap();
                        match text.0.as_str() {
                            "Next>" => {
                                bc.0 = Color::srgba(0.0, 0.0, 0.0, 0.8);
                                text.1.0 = Color::srgba(0.0, 0.0, 0.0, 0.8);
                                break;
                            }
                            _ => continue,
                        }
                    }
                } else {
                    config.characters_id[0] = id.0;
                    config.characters_id[1] = choose_rand_character(id.0);
                    for (children, mut bc) in transitin_query.iter_mut() {
                        let mut text = text_query.get_mut(children[0]).unwrap();
                        match text.0.as_str() {
                            "Next>" => {
                                bc.0 = Color::BLACK;
                                text.1.0 = Color::BLACK;
                                break;
                            }
                            _ => continue,
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_buttons(
    mut state: ResMut<NextState<AppState>>,
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<(&Text, &TextColor)>,
) {
    for (interaction, children) in &mut interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                if children.len() > 0 {
                    let text = text_query.get(children[0]).unwrap();
                    match text.0.as_str() {
                        "<Back" => {
                            #[cfg(not(target_arch = "wasm32"))]
                            state.set(AppState::ConnectController);
                            #[cfg(target_arch = "wasm32")]
                            state.set(AppState::Mainmenu);
                            break;
                        }
                        "Next>" => {
                            if text.1.0 == Color::BLACK {
                                state.set(AppState::Ingame);
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn exit(
    mut commands: Commands,
    query: Query<Entity, With<ChooseCharacter>>,
) {
    info!("exit");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

impl Plugin for ChooseCharacterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::ChooseCharacter), setup)
            .add_systems(OnExit(AppState::ChooseCharacter), exit)
            .add_systems(Update, check_buttons.run_if(in_state(AppState::ChooseCharacter)))
            .add_systems(Update, update.run_if(in_state(AppState::ChooseCharacter)));
    }
}