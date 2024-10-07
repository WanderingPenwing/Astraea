use bevy::prelude::*;
use std::f64::consts::PI;
//use bevy::input::mouse::MouseMotion;
use crate::GameState;
use crate::StartMenu;
use crate::Player;

#[derive(Component)]
struct AudioPlayer;

pub fn audio_setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((AudioBundle {
        source: asset_server.load("Banjo.ogg"),
        settings: PlaybackSettings::LOOP,
    }, AudioPlayer));
    info!("audio started");
}

pub fn setup(
	mut commands: Commands,
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
        // font: asset_server.load("fonts/FiraSans-Bold.ttf"), // Load font if needed
        ..default()
    };

    let bottom_text_style = TextStyle {
        font_size: 30.0, 
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Regular.ttf"), // Load font if needed
        ..default()
    };

    let top_text_node = TextBundle::from_section(
        "Astraea", 
        top_text_style,
    );

    let bottom_text_node = TextBundle::from_section(
        "Press Space to Begin", 
        bottom_text_style,
    );

    let top_text = commands.spawn((top_text_node, StartMenu)).id();
    let bottom_text = commands.spawn((bottom_text_node, StartMenu)).id();

    commands.entity(container).push_children(&[top_text, bottom_text]);
}

pub fn player_interact(
	keys: Res<ButtonInput<KeyCode>>,
	mut game_state: ResMut<NextState<GameState>>,
	mut player_query: Query<(&mut Player, &mut Transform)>,
) {
	if keys.just_pressed(KeyCode::Space) {
		info!("start space");
		game_state.set(GameState::Game);
	}

	if let Ok((_player, mut transform)) = player_query.get_single_mut() {
		let mut rotation = Quat::IDENTITY;
		rotation *= Quat::from_rotation_y((PI / 6000.0) as f32);
	    rotation *= Quat::from_rotation_x((-PI / 2000.0) as f32);
	    transform.rotation *= rotation; 
	}
}
