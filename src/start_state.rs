use bevy::prelude::*;
use std::f64::consts::PI;
use crate::Player;
use crate::GameState;
use crate::GameOver;
use crate::StartMenu;
use crate::PlayerState;

pub fn audio_setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioBundle {
        source: asset_server.load("assets/Banjo.ogg"),
        ..default()
    });
}

pub fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Create a container node that places its children (text areas) in a vertical column and centers them
    let container_node = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),  // Full width of the screen
            height: Val::Percent(100.0), // Full height of the screen
            flex_direction: FlexDirection::Column, // Arrange children in a column (vertical)
            justify_content: JustifyContent::Center, // Center vertically
            align_items: AlignItems::Center, // Center horizontally
            ..default()
        },
        ..default()
    };

    // Create the container for the text areas
    let container = commands.spawn(container_node).id();

    // TextStyle for the top text (larger font)
    let top_text_style = TextStyle {
        font_size: 50.0, // Larger font size
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Bold.ttf"), // Load font if needed
        ..default()
    };

    // TextStyle for the bottom text (smaller font)
    let bottom_text_style = TextStyle {
        font_size: 30.0, // Smaller font size
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Regular.ttf"), // Load font if needed
        ..default()
    };

    // TextBundle for the top text
    let top_text_node = TextBundle::from_section(
        "Astraea", // Text for the top section
        top_text_style,
    );

    // TextBundle for the bottom text
    let bottom_text_node = TextBundle::from_section(
        "Press Space to Begin", // Text for the bottom section
        bottom_text_style,
    );

    // Spawn the text nodes and add them as children to the container
    let top_text = commands.spawn((top_text_node, StartMenu)).id();
    let bottom_text = commands.spawn((bottom_text_node, StartMenu)).id();

    commands.entity(container).push_children(&[top_text, bottom_text]);

    commands.spawn((
       	Camera3dBundle {
   	        transform: Transform::from_xyz(0.0, 0.0, 0.0),
   	        ..default()
   	    },
   	    Player {
   	    	target_rotation: None,
   	    	target_cons_name: None,
   	    	score: 0,
   	    	health: 3,
   	    	state: PlayerState::Playing,
   	    },
   	    GameOver,
   	));
}

pub fn player_interact(
	keys: Res<ButtonInput<KeyCode>>,
	mut game_state: ResMut<NextState<GameState>>,
	mut player_query: Query<(&mut Player, &mut Transform)>
) {
	if keys.just_pressed(KeyCode::Space) {
		info!("start space");
		game_state.set(GameState::Game);
	}


	if let Ok((_player, mut transform)) = player_query.get_single_mut() {
		let mut rotation = Quat::IDENTITY;
		rotation *= Quat::from_rotation_y((PI / 6000.0) as f32); // Rotate by 3 degrees (PI/60 radians)
	    rotation *= Quat::from_rotation_x((-PI / 2000.0) as f32); // Rotate by -3 degrees
	    transform.rotation *= rotation; // Apply the rotation
	}
}
