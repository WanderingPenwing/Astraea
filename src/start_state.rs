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
    let main_container_node = NodeBundle {
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

    let main_container = commands.spawn(main_container_node).id();

    let title_text_style = TextStyle {
        font_size: 50.0, 
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Bold.ttf"), // Load font if needed
        ..default()
    };

    let start_text_style = TextStyle {
        font_size: 30.0, 
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Regular.ttf"), // Load font if needed
        ..default()
    };

    let explo_text_style = TextStyle {
        font_size: 30.0, 
        color: Color::srgb(0.4,0.4,0.4),
        // font: asset_server.load("fonts/FiraSans-Regular.ttf"), // Load font if needed
        ..default()
    };

    let title_text_node = TextBundle::from_section(
        "Astraea", 
        title_text_style,
    );

    let start_text_node = TextBundle::from_section(
        "Press Space to Begin", 
        start_text_style,
    );

    let explo_text_node = TextBundle::from_section(
        "Press E to Explore", 
        explo_text_style,
    );

    let title_text = commands.spawn((title_text_node, StartMenu)).id();
    let start_text = commands.spawn((start_text_node, StartMenu)).id();
    let explo_text = commands.spawn((explo_text_node, StartMenu)).id();

    commands.entity(main_container).push_children(&[title_text, start_text, explo_text]);
}

pub fn player_interact(
	keys: Res<ButtonInput<KeyCode>>,
	mut game_state: ResMut<NextState<GameState>>,
	mut player_query: Query<(&mut Player, &mut Transform)>,
) {
	if keys.just_pressed(KeyCode::Space) {
		game_state.set(GameState::Game);
	}

	if keys.just_pressed(KeyCode::KeyE) {
		game_state.set(GameState::Explo);
	}

	if let Ok((_player, mut transform)) = player_query.get_single_mut() {
		let mut rotation = Quat::IDENTITY;
		rotation *= Quat::from_rotation_y((PI / 6000.0) as f32);
	    rotation *= Quat::from_rotation_x((-PI / 2000.0) as f32);
	    transform.rotation *= rotation; 
	}
}
