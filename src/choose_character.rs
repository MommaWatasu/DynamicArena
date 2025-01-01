use bevy::{prelude::*, ui::widget::NodeImageMode};

use crate::{
    AppState,
    GameConfig,
    TITLE_FONT_SIZE,
    PATH_BOLD_FONT,
    PATH_EXTRA_BOLD_JP_FONT,
    PATH_IMAGE_PREFIX,
    PATH_JP_FONT,
};

#[derive(Component)]
struct ChooseCharacter;

pub struct ChooseCharacterPlugin;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("choose_character: setup");
    commands
        .spawn((Node {
            width: Val::Percent(90.0),
            height: Val::Percent(90.0),
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            ..default()
        },
        ChooseCharacter
    ))
        .with_children(|parent| {
            parent.spawn((
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
            parent.spawn((
                Node{
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    flex_direction: FlexDirection::Row,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    ..default()
                },
                BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.1, 0.8))),
                BorderRadius::all(Val::Px(20.0)),
            ))
                .with_children(|builder| {
                    let character_names = vec!["character1", "character2", "character3"];
                    for i in 0..3 {
                        let image_handle = asset_server.load(format!("{}character{}.png", PATH_IMAGE_PREFIX, i+1).as_str());
                        let font_bold = asset_server.load(PATH_BOLD_FONT);
                        let font_regular = asset_server.load(PATH_JP_FONT);
                        create_character_box(builder, font_bold, font_regular, character_names[i], image_handle);
                    }
                });
            });
}

fn create_character_box(
    builder: &mut ChildBuilder,
    font_bold: Handle<Font>,
    font_regular: Handle<Font>,
    character_name: &str,
    image_handle: Handle<Image>,
) {
    builder.spawn((
        Node {
            width: Val::Percent(30.0),
            height: Val::Percent(95.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceEvenly,
            align_content: AlignContent::SpaceEvenly,
            ..default()
        },
        BorderRadius::all(Val::Px(20.0)),
        BackgroundColor(Color::Srgba(Srgba::new(0.6, 0.8, 0.9, 0.8))),
    )).with_children(|builder| {
        builder.spawn((
            Text::new(character_name),
            TextFont {
                font: font_bold,
                font_size: 40.0,
                ..Default::default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::BLACK),
        ));
        builder.spawn((
            Text::new("移動速度は速いが、攻撃力は高くない"),
            TextFont {
                font: font_regular,
                font_size: 20.0,
                ..Default::default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
        ));
        builder.spawn((
            ImageNode {
                image: image_handle,
                ..Default::default()
            },
            Node {
                aspect_ratio: Some((1/3) as f32),
                align_self: AlignSelf::Center,
                ..default()
            },
        ));
    });
}

fn update() {}

impl Plugin for ChooseCharacterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::ChooseCharacter), setup)
            .add_systems(Update, update.run_if(in_state(AppState::ChooseCharacter)));
    }
}