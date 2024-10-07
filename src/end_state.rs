use bevy::prelude::*;

use crate::GameState;
use crate::GameOver;
use crate::GameData;

pub fn setup(
	mut commands: Commands,
	_asset_server: Res<AssetServer>,
	game_data: Res<GameData>,
) {
    let container_node = NodeBundle {
        style: Style {
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    };

    let container = commands.spawn(container_node).id();

    let top_text_style = TextStyle {
        font_size: 50.0, 
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        ..default()
    };

    let bottom_text_style = TextStyle {
        font_size: 30.0, 
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Regular.ttf"), 
        ..default()
    };

    let top_text_node = TextBundle::from_section(
        "Game Over", 
        top_text_style,
    );

    let bottom_text_node = TextBundle::from_section(
        format!("final score : {}", game_data.score), 
        bottom_text_style,
    );

    let top_text = commands.spawn((top_text_node, GameOver)).id();
    let bottom_text = commands.spawn((bottom_text_node, GameOver)).id();

    commands.entity(container).push_children(&[top_text, bottom_text]);
}

pub fn player_interact(
	keys: Res<ButtonInput<KeyCode>>,
	mut game_state: ResMut<NextState<GameState>>
) {
	if keys.just_pressed(KeyCode::Space) {
		info!("start space");
		game_state.set(GameState::Start);
	}
}
