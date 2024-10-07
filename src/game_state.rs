use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use std::f64::consts::PI;
use rand::seq::SliceRandom;
use rand::RngCore;

use crate::Player;
use crate::GameState;
use crate::MainGame;
use crate::Sky;
use crate::Constellation;
use crate::ConstellationLine;
use crate::PlayerState;

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

#[derive(Component)]
pub struct DebugLine;

pub fn setup(
	mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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


	let line_material = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(2.0, 0.5, 0.5),
        ..default()
    });

    let vertices = vec![
    	Vec3::new(0.0, 0.0, 0.0),
    	Vec3::new(1.0, 0.0, 0.0)
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
   	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    
	commands.spawn((
	    PbrBundle {
	        mesh: meshes.add(mesh),
	        material: line_material.clone(),
	        transform: Transform::default(),
	        ..default()
	    },
	 	DebugLine,
	 	MainGame
	));
}


pub fn player_interact(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform)>, 
    sky: Res<Sky>, 
    text_query: Query<&mut Text, With<AnswerButton>>,
    button_query: Query<(&mut BackgroundColor, &mut BorderColor), With<Button>>, 
	constellation_line_query : Query<(Entity, &ConstellationLine)>,
	commands: Commands,
    game_state: ResMut<NextState<GameState>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
    
        if keys.just_pressed(KeyCode::Space) || player.target_cons_name.is_none() {
            choose_constellation(&mut player, sky, text_query, button_query, constellation_line_query, commands, game_state);
			return
        }

        let mut rotation = Quat::IDENTITY;
        
        if keys.pressed(KeyCode::KeyA) {
            rotation *= Quat::from_rotation_y((PI / 60.0) as f32); 
        }

        if keys.pressed(KeyCode::KeyD) {
            rotation *= Quat::from_rotation_y((-PI / 60.0) as f32); 
        }

        if keys.pressed(KeyCode::KeyI) && player.state == PlayerState::Playing {
        	if let Some(target_cons) = player.target_cons_name.clone() {
        		player.state = PlayerState::Hinted;
    			spawn_cons_lines(commands, meshes, materials, sky, target_cons);
    			return
    		}
 		}

        if rotation != Quat::IDENTITY {
            transform.rotation *= rotation; 
            player.target_rotation = None;
        }

        if keys.pressed(KeyCode::KeyR) {
 			if let Some(target_constellation_name) = player.target_cons_name.clone() {
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

        if let Some(target_rotation) = player.target_rotation {
            let current_rotation = transform.rotation;

            transform.rotation = current_rotation.slerp(target_rotation, 0.1);

            if transform.rotation.angle_between(target_rotation) < 0.01 {
                player.target_rotation = None; 
            }
        }
   }
}

pub fn player_mouse_move (
    buttons: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<(&mut Player, &Camera, &mut GlobalTransform)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    // debug_line_query: Query<&mut Transform, (With<DebugLine>, Without<Player>)>,
) {
	let Ok((mut player, camera, global_transform)) = player_query.get_single_mut() else {
	    return;
	};
	let local_transform = &global_transform.compute_transform();

	// Check if left mouse button is pressed
	if !buttons.pressed(MouseButton::Left) {
	    player.dragging_pos = None;
	    return;
	}

	let window = window_query.single();

	let Some(new_cursor) = window.cursor_position() else {
	    return;
	};

	let Some(old_cursor) = player.dragging_pos else {
	    player.dragging_pos = Some(new_cursor);
	    return;
	};

	// Check if cursor has moved significantly
	if old_cursor.distance(new_cursor) < 3.0 {
	    return;
	}

	// Raycasting from the camera based on the cursor positions
	let Some(old_ray) = camera.viewport_to_world(&global_transform, old_cursor) else {
	    return;
	};

	let Some(new_ray) = camera.viewport_to_world(&global_transform, new_cursor) else {
	    return;
	};

	let delta_rotation = rotate_to_align(new_ray, old_ray); // oposite direction and never stop

	//debug_vector(debug_line_query, new_ray.get_point(SKY_RADIUS), axis*10.0);

	player.target_rotation = Some(delta_rotation * local_transform.rotation );
	player.dragging_pos = Some(new_cursor);
}


fn rotate_to_align(ray_1: Ray3d, ray_2: Ray3d) -> Quat {
    // Step 1: Get the direction vectors from the rays
    let pos_1 = ray_1.get_point(1.0);
    let pos_2 = ray_2.get_point(1.0);
    
    // Compute direction vectors
    let dir_1 = (pos_1 - Vec3::ZERO).normalize(); // Change Vec3::ZERO to the origin or a relevant point
    let dir_2 = (pos_2 - Vec3::ZERO).normalize();

    // Step 2: Compute the axis of rotation (cross product)
    let axis_of_rotation = dir_1.cross(dir_2).normalize();

    // Check if vectors are parallel
    if axis_of_rotation.length_squared() < f32::EPSILON {
        return Quat::IDENTITY;
    }

    // Step 3: Compute the angle of rotation (dot product and arccosine)
    let dot_product = dir_1.dot(dir_2).clamp(-1.0, 1.0); // Clamp the value to prevent NaN from acos
    let angle_of_rotation = dot_product.acos() * 6.0; // Ensure the angle is in radians

    // Handle any potential invalid angle values
    if angle_of_rotation.is_nan() || angle_of_rotation.is_infinite() {
        return Quat::IDENTITY;
    }

    // Step 4: Create a quaternion representing the rotation
    Quat::from_axis_angle(axis_of_rotation, angle_of_rotation)
}

// fn debug_vector(
// 	mut debug_line_query: Query<&mut Transform, (With<DebugLine>, Without<Player>)>,
// 	pos: Vec3, vec: Vec3
// ) {
// 	let Ok(mut debug_transform) = debug_line_query.get_single_mut() else {
// 		info!("no debug line");
// 		return;
// 	};
// 	
//     let vec_norm = vec.length();
// 
//     debug_transform.scale = Vec3::new(vec_norm, 1.0, 1.0);
// 
//     debug_transform.translation = pos;
// 
//     if vec_norm > f32::EPSILON {
//         let rotation = Quat::from_rotation_arc(Vec3::X, vec.normalize());
//         debug_transform.rotation = rotation;
//     }
// }


pub fn ui_labels(
    mut param_set: ParamSet<(
        Query<&mut Text, With<HealthLabel>>,
        Query<&mut Text, With<ScoreLabel>>,
        Query<&mut Text, With<HintLabel>>,
    )>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((player, _)) = player_query.get_single_mut() {
        if let Ok(mut health_text) = param_set.p0().get_single_mut() {
            health_text.sections[0].value = "# ".repeat(player.health);
        }

        if let Ok(mut score_text) = param_set.p1().get_single_mut() {
            score_text.sections[0].value = format!("{}", player.score);
        }

		if let Ok(mut hint_text) = param_set.p2().get_single_mut() {
			if player.state == PlayerState::Answered {
	            hint_text.sections[0].value = "press space to continue".into();
	        } else {
	        	hint_text.sections[0].value = "press i to get an hint".into();
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
    mut player_query: Query<&mut Player>,
	commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    sky: Res<Sky>
) {
	if let Ok(mut player) = player_query.get_single_mut() {
		if player.state == PlayerState::Answered {
			return
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

		if let Some(selected_cons) = pressed_button {
		    if let Some(target_cons) = player.target_cons_name.clone() {
		    	if player.state == PlayerState::Playing {
		    		spawn_cons_lines(commands, meshes, materials, sky, target_cons.clone());
		    	}
		    	
		    	if target_cons == selected_cons {
		    		if player.state == PlayerState::Hinted {
		    			player.score += 20;
		    		} else {
		    			player.score += 100;
		    		}
		    	} else {
		    		player.health -= 1;
		    	}

		    	player.state = PlayerState::Answered;

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
    mut game_state: ResMut<NextState<GameState>>
) {
	if player.health == 0 {
		game_state.set(GameState::End);
	}
	if sky.content.len() >= 4 {
        let mut rng = rand::thread_rng();
        let mut selected_constellations = sky.content.clone();
        selected_constellations.shuffle(&mut rng);
        let constellations = &selected_constellations[0..4];

        let target_index = rng.next_u32().rem_euclid(4) as usize;
        let target_constellation = &constellations[target_index];

        player.target_rotation = Some(constellation_center(target_constellation.clone()));
        player.target_cons_name = Some(target_constellation.name.clone());

        info!("Target constellation: {}", target_constellation.name);

        for (i, mut text) in text_query.iter_mut().enumerate() {
            text.sections[0].value = constellations[i].name.clone();
        }

        for (mut bg_color, mut border_color) in &mut button_query {
            *bg_color = NORMAL_BUTTON.into();
            *border_color = Color::BLACK.into();
        }

        for (entity, _line) in constellation_line_query.iter() {
            commands.entity(entity).despawn();
        }

        player.state = PlayerState::Playing;
    } else {
        info!("Not enough constellations in the sky (need 4)");
    }
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


