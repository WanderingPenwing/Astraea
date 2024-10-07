use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::RngCore;

use crate::Player;
use crate::GameState;
use crate::MainGame;
use crate::Sky;
use crate::Constellation;
use crate::ConstellationModel;

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

#[derive(Resource)]
pub struct GameData {
    content: Vec<String>,
	pub score: usize,
	health: usize,
	state: PlayerState,
	target_cons_name: Option<String>,
	target_cons_focused: bool,
}

impl Default for GameData {
    fn default() -> Self {
         GameData {
         	content: vec![],
   	    	score: 0,
   	    	health: 3,
   	    	state: PlayerState::Playing,
   	    	target_cons_name: None,
   	    	target_cons_focused: false,
   	    }
    }
}

#[derive(Default, PartialEq, Debug)]
enum PlayerState {
	#[default]
	Playing,
	Hinted,
	Answered,
}

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
	constellation_line_query : Query<(Entity, &ConstellationModel)>,
    mut game_state: ResMut<NextState<GameState>>,
	mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(mut player) = player_query.get_single_mut() else {
		return
    };

    if player.dragging_pos.is_some() {
    	game_data.target_cons_focused = false;
    }
  
    if keys.just_pressed(KeyCode::Space) || game_data.target_cons_name.is_none() {
        choose_constellation(&mut player, sky, text_query, button_query, constellation_line_query, commands, game_state, game_data);
		return
    }
    
    if keys.just_pressed(KeyCode::Escape) {
   		game_state.set(GameState::Start);
   	}
	
    if keys.pressed(KeyCode::KeyI) {
    	if game_data.state != PlayerState::Playing {
    		info!("Invalid state : {:?}", game_data.state);
    	}
    	let Some(target_cons) = game_data.target_cons_name.clone() else {
			return;
  		};
  		game_data.state = PlayerState::Hinted;
		spawn_cons_lines(&mut commands, &mut meshes, &mut materials, sky.get_constellation(&target_cons));
		return;
	}

    if keys.pressed(KeyCode::KeyW) {
		game_data.target_cons_focused = true;
		let Some(target_constellation_name) = game_data.target_cons_name.clone() else {
			return;
		};
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
    mut label_query: Query<(&mut Text, Option<&HealthLabel>, Option<&ScoreLabel>, Option<&HintLabel>)>,
    game_data: Res<GameData>
) {

	for (mut text, health_label, score_label, hint_label) in label_query.iter_mut() {
		if health_label.is_some() {
			text.sections[0].value = "# ".repeat(game_data.health);
		} else if score_label.is_some() {
			text.sections[0].value = format!("{}", game_data.score);
		} else if hint_label.is_some() {
			if !game_data.target_cons_focused {
				text.sections[0].value = "press z to re-center".into();
			} else if game_data.state == PlayerState::Playing {
				text.sections[0].value = "press i to get an hint".into();
			} else if game_data.state == PlayerState::Answered {
				text.sections[0].value = "press space to continue".into();
			} else {
				text.sections[0].value = "guess the constellation".into();
			}
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
	mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
   		spawn_cons_lines(&mut commands, &mut meshes, &mut materials, sky.get_constellation(&target_cons));
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
	constellation_line_query : Query<(Entity, &ConstellationModel)>,
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
    game_data.target_cons_focused = true;
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


