use bevy::prelude::*;
use core::Player;
use core::GameState;

fn setup(
	mut commands: Commands,
	_asset_server: Res<AssetServer>,
	mut player_query: Query<&mut Player>
) {
	if let Ok(player) = player_query.get_single_mut() {
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
	        "Game Over", // Text for the top section
	        top_text_style,
	    );

	    // TextBundle for the bottom text
	    let bottom_text_node = TextBundle::from_section(
	        format!("final score : {}", player.score), // Text for the bottom section
	        bottom_text_style,
	    );

	    // Spawn the text nodes and add them as children to the container
	    let top_text = commands.spawn((top_text_node, GameOver)).id();
	    let bottom_text = commands.spawn((bottom_text_node, GameOver)).id();

	    commands.entity(container).push_children(&[top_text, bottom_text]);
	}
}

fn buttons(
	keys: Res<ButtonInput<KeyCode>>,
	mut game_state: ResMut<NextState<GameState>>
) {
	if keys.just_pressed(KeyCode::Space) {
		info!("start space");
		game_state.set(GameState::Start);
	}
}
