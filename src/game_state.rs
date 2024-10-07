use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::RngCore;

use crate::Player;
use crate::GameState;
use crate::MainGame;
use crate::Sky;
use crate::Constellation;
use crate::ConstellationLine;
use crate::PlayerState;
use crate::GameData;

use crate::celestial_to_cartesian;
use crate::spawn_cons_lines;

use crate::NORMAL_BUTTON;
use crate::RIGHT_BUTTON;
use crate::WRONG_BUTTON;

#[derive(Component)]
pub struct AnswerButton;

#[derive(Component)]
pub struct HealthLabel;

#[derive(Component)]
pub struct ScoreLabel;

#[derive(Component)]
pub struct HintLabel;

pub fn setup(
	mut commands: Commands, 
	mut game_data: ResMut<GameData>,
	sky: Res<Sky>,
) {
    let container_node = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0), 
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center, 
            align_items: AlignItems::FlexEnd,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        ..default()
    };

    let button_style = Style {
        width: Val::Px(150.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(10.0)), 
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, 
        border: UiRect::all(Val::Px(5.0)),
        ..default()
    };

    let container = commands.spawn(container_node).id();

    for _i in 1..=4 {
        let button_node = ButtonBundle {
            style: button_style.clone(),
            border_color: BorderColor(Color::BLACK),
            background_color: NORMAL_BUTTON.into(),
            ..default()
        };

        let button_text_node = TextBundle::from_section(
            "".to_string(),
            TextStyle {
                //font: asset_server.load("fonts/FiraSans-Bold.ttf"), // Load font
                font_size: 15.0,
                color: Color::srgb(0.9, 0.9, 0.9),
                ..default()
            },
        );

        let button = commands.spawn((button_node, MainGame)).id();
        let button_text = commands.spawn((button_text_node, AnswerButton, MainGame)).id();

        commands.entity(button).push_children(&[button_text]);
        commands.entity(container).push_children(&[button]);
    }

    let label_style = Style {
        position_type: PositionType::Absolute,
        width: Val::Auto, 
        height: Val::Auto, 
        margin: UiRect::all(Val::Px(10.0)), 
        ..default()
    };

    // Top left label
    let top_left_label_node = TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..label_style.clone()
        },
        text: Text::from_section(
            "* * *", 
            TextStyle {
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        ..default()
    };

    // Top right label
    let top_right_label_node = TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            ..label_style.clone()
        },
        text: Text::from_section(
            "0",
            TextStyle {
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        ..default()
    };

    let centered_container_node = NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Px(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    };

    // Hint label
    let hint_label_node = TextBundle::from_section(
        "hint",
        TextStyle {
            // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 20.0,
            color: Color::srgb(0.7, 0.7, 0.7),
            ..default()
        },
    );
    
    commands.spawn((top_left_label_node, MainGame, HealthLabel));
    commands.spawn((top_right_label_node, MainGame, ScoreLabel));

    let centered_container = commands.spawn(centered_container_node).id();
    let hint_label = commands.spawn((hint_label_node, MainGame, HintLabel)).id();

    commands.entity(centered_container).push_children(&[hint_label]);

	*game_data = GameData::default();
	game_data.content = sky.as_string();
}


pub fn player_interact(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Player>, 
    mut game_data: ResMut<GameData>, 
    sky: Res<Sky>, 
    text_query: Query<&mut Text, With<AnswerButton>>,
    button_query: Query<(&mut BackgroundColor, &mut BorderColor), With<Button>>, 
	constellation_line_query : Query<(Entity, &ConstellationLine)>,
	commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(mut player) = player_query.get_single_mut() else {
		return
    };
  
    if keys.just_pressed(KeyCode::Space) || game_data.target_cons_name.is_none() {
        choose_constellation(&mut player, sky, text_query, button_query, constellation_line_query, commands, game_state, game_data);
		return
    }
    
    if keys.just_pressed(KeyCode::Escape) {
   		game_state.set(GameState::Start);
   	}

    if keys.pressed(KeyCode::KeyI) && game_data.state == PlayerState::Playing {
    	if let Some(target_cons) = game_data.target_cons_name.clone() {
    		game_data.state = PlayerState::Hinted;
			spawn_cons_lines(commands, meshes, materials, sky, target_cons);
			return
  		}
	}

    if keys.pressed(KeyCode::KeyR) {
		if let Some(target_constellation_name) = game_data.target_cons_name.clone() {
			let mut target_constellation = sky.content[0].clone();
			for constellation in sky.content.clone() {
				if constellation.name == target_constellation_name {
					target_constellation = constellation
				}
			}
			player.target_rotation = Some(constellation_center(target_constellation));
		}
    }

    if keys.pressed(KeyCode::KeyW) {
        let target_constellation_name: String = "Ursa Minor".into();
          	
		let mut target_constellation = sky.content[0].clone();
		for constellation in sky.content.clone() {
			if constellation.name == target_constellation_name {
				target_constellation = constellation
			}
		}
		
		player.target_rotation = Some(constellation_center(target_constellation));
	}
}

pub fn ui_labels(
    mut param_set: ParamSet<(
        Query<&mut Text, With<HealthLabel>>,
        Query<&mut Text, With<ScoreLabel>>,
        Query<&mut Text, With<HintLabel>>,
    )>,
    game_data: Res<GameData>
) {
	if let Ok(mut health_text) = param_set.p0().get_single_mut() {
		health_text.sections[0].value = "# ".repeat(game_data.health);
	}

	if let Ok(mut score_text) = param_set.p1().get_single_mut() {
		score_text.sections[0].value = format!("{}", game_data.score);
	}

	if let Ok(mut hint_text) = param_set.p2().get_single_mut() {
		if game_data.state == PlayerState::Answered {
			hint_text.sections[0].value = "press space to continue".into();
		} else {
			hint_text.sections[0].value = "press i to get an hint".into();
		}
	}
}


pub fn ui_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        With<Button>,
    >,
    mut text_query: Query<&mut Text, With<AnswerButton>>,
    mut game_data: ResMut<GameData>,
	commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    sky: Res<Sky>
) {	
	if game_data.state == PlayerState::Answered {
		return;
	}

	let mut pressed_button: Option<String> = None;

	for (
	    interaction,
	    _color,
	    _border_color,
	    children
	) in &mut interaction_query {
	    if *interaction == Interaction::Pressed {
	    	if let Ok(text) = text_query.get_mut(children[0]) {
	            pressed_button = Some(text.sections[0].value.clone());
	        }
	    }
	}

	let Some(selected_cons) = pressed_button else {
		return;
	};

	let Some(target_cons) = game_data.target_cons_name.clone() else {
		return;
	};
	
   	if game_data.state == PlayerState::Playing {
   		spawn_cons_lines(commands, meshes, materials, sky, target_cons.clone());
   	}
	    	
   	if target_cons == selected_cons {
   		if game_data.state == PlayerState::Hinted {
   			game_data.score += 20;
   		} else {
   			game_data.score += 100;
   		}
   	} else {
   		game_data.health -= 1;
   	}

   	game_data.content.retain(|x| x != &target_cons);

   	game_data.state = PlayerState::Answered;

 	for (
        _interaction,
        mut color,
        mut border_color,
        children
    ) in &mut interaction_query {
    	if let Ok(text) = text_query.get_mut(children[0]) {
    		let button_text = text.sections[0].value.clone();
    		
	        *color = if button_text == target_cons {
	        	RIGHT_BUTTON.into()
	        } else {
	        	WRONG_BUTTON.into()
	        };

	        border_color.0 = if button_text == selected_cons {
	        	Color::WHITE
	        } else {
	        	Color::BLACK
	        };
	    }
    }
}

fn choose_constellation(
	player: &mut Player, 
	sky: Res<Sky>, 
    mut text_query: Query<&mut Text, With<AnswerButton>>,
    mut button_query: Query<(&mut BackgroundColor, &mut BorderColor), With<Button>>, 
	constellation_line_query : Query<(Entity, &ConstellationLine)>,
	mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
) {
	if game_data.health == 0 {
		game_state.set(GameState::End);
	}

	if game_data.content.len() < 4 {
		game_state.set(GameState::End);
	}
	
    let mut rng = rand::thread_rng();
    let mut cons_names = game_data.content.clone();
    cons_names.shuffle(&mut rng);
    let selected_cons_names = &cons_names[0..4];

    let target_index = rng.next_u32().rem_euclid(4) as usize;
    let target_constellation = sky.get_constellation(&selected_cons_names[target_index]);

    player.target_rotation = Some(constellation_center(target_constellation.clone()));
    game_data.target_cons_name = Some(target_constellation.name.clone());

    info!("Target constellation: {}", target_constellation.name);

    for (i, mut text) in text_query.iter_mut().enumerate() {
        text.sections[0].value = selected_cons_names[i].clone();
    }

    for (mut bg_color, mut border_color) in &mut button_query {
        *bg_color = NORMAL_BUTTON.into();
        *border_color = Color::BLACK.into();
    }

    for (entity, _line) in constellation_line_query.iter() {
        commands.entity(entity).despawn();
    }

    game_data.state = PlayerState::Playing;
}

fn constellation_center(target_constellation: Constellation) -> Quat {
	let mut mean_pos = Vec3::ZERO;
	
    for star in target_constellation.stars.clone() {
    	mean_pos += celestial_to_cartesian(star.rah, star.dec)
    }

    Quat::from_rotation_arc(
        Vec3::Z,
        -mean_pos*(1.0/target_constellation.stars.len() as f32),
    )
}


