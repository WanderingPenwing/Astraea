use bevy::prelude::*;
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::mouse::MouseWheel;

use std::f32::consts::{E, PI};

use crate::Player;
use crate::GameState;
use crate::ConstellationModel;
use crate::Sky;
use crate::MainGame;

use crate::spawn_cons_lines;

use crate::CONS_VIEW_RADIUS;
use crate::MOUSE_SPEED;

#[derive(Component)]
pub struct InfoLabel;

pub fn setup (
	sky : Res<Sky>,
	mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
	for constellation in sky.content.iter() {
		spawn_cons_lines(&mut commands, &mut meshes, &mut materials, constellation.clone());
	}
	
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
	
	let info_label_node = TextBundle::from_section(
        "info",
        TextStyle {
            // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 20.0,
            color: Color::srgb(0.7, 0.7, 0.7),
            ..default()
        },
    );

    let centered_container = commands.spawn(centered_container_node).id();
    let info_label = commands.spawn((info_label_node, MainGame, InfoLabel)).id();

    commands.entity(centered_container).push_children(&[info_label]);
}

pub fn player_mouse_move (
    buttons: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<(&mut Player, &Camera, &mut GlobalTransform)>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    ui_query: Query<&Interaction, With<Button>>,
) {
    for interaction in ui_query.iter() {
        if *interaction == Interaction::Pressed {
        	// Button clicked
            return;
        }
    }
    
	let Ok((mut player, camera, global_transform)) = player_query.get_single_mut() else {
	    return;
	};
	let local_transform = &global_transform.compute_transform();

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

	if old_cursor.distance(new_cursor) < 3.0 {
	    return;
	}

	let Some(old_ray) = camera.viewport_to_world(&global_transform, old_cursor) else {
	    return;
	};

	let Some(new_ray) = camera.viewport_to_world(&global_transform, new_cursor) else {
	    return;
	};

	let delta_rotation = rotate_to_align(new_ray, old_ray); 

	player.target_rotation = Some(delta_rotation * local_transform.rotation );
	player.dragging_pos = Some(new_cursor);
}

pub fn zoom(
	mut evr_scroll: EventReader<MouseWheel>,
	mut projection_query: Query<&mut Projection, With<Player>>,
) {
	let Ok(mut projection) = projection_query.get_single_mut() else {
		return;
	};

	let Projection::Perspective(ref mut perspective) = *projection else {
		return;
	};
	
	for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
				perspective.fov = (0.6*PI).min((0.02*PI).max(perspective.fov * 0.9_f32.powf(ev.y)));
            }
            MouseScrollUnit::Pixel => {
                println!("Scroll (pixel units): vertical: {}, horizontal: {}", ev.y, ev.x);
            }
        }
    }
}

fn rotate_to_align(ray_1: Ray3d, ray_2: Ray3d) -> Quat {
    let pos_1 = ray_1.get_point(1.0);
    let pos_2 = ray_2.get_point(1.0);
    
    let dir_1 = pos_1.normalize();
    let dir_2 = pos_2.normalize();

    let axis_of_rotation = dir_1.cross(dir_2).normalize();

    if axis_of_rotation.length_squared() < f32::EPSILON {
        return Quat::IDENTITY;
    }

    let dot_product = dir_1.dot(dir_2).clamp(-1.0, 1.0);
    let angle_of_rotation = dot_product.acos() * MOUSE_SPEED;

    if angle_of_rotation.is_nan() || angle_of_rotation.is_infinite() {
        return Quat::IDENTITY;
    }

    Quat::from_axis_angle(axis_of_rotation, angle_of_rotation)
}

pub fn rotate_camera(
	mut player_query : Query<(&mut Player, &mut Transform)>
) {
	let Ok((mut player, mut transform)) = player_query.get_single_mut() else {
        return;
    };
	
	let Some(target_rotation) = player.target_rotation else {
        return;
    };
    
    let current_rotation = transform.rotation;
    
    transform.rotation = current_rotation.slerp(target_rotation, 0.1);

    if transform.rotation.angle_between(target_rotation) < 0.01 {
        player.target_rotation = None; 
    }
}

pub fn player_interact(
	keys: Res<ButtonInput<KeyCode>>,
	mut game_state: ResMut<NextState<GameState>>,
	//mut player_query: Query<(&mut Player, &mut Transform)>,
) {
	if keys.just_pressed(KeyCode::Escape) {
		game_state.set(GameState::Start);
	}
}

pub fn constellation_opacity(
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&Player, &Camera, &GlobalTransform)>,
    constellation_query: Query<(&Handle<StandardMaterial>, &ConstellationModel)>, // Query all constellation lines
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut info_label_query: Query<&mut Text, With<InfoLabel>>,
) {
	let (_player, camera, global_transform) = player_query.single();
	let window = window_query.single();
	let Some(cursor_position) = window.cursor_position() else {
		return;
	};

	let Some(mouse_ray) = camera.viewport_to_world(&global_transform, cursor_position) else {
	    return;
	};

	let cursor_global_pos = mouse_ray.get_point(1.0);

	let mut closest_const_name: String = "".into();
	let mut closest_const_pos: Vec3 = Vec3::ZERO;

	
    for (material_handle, constellation_model) in constellation_query.iter() {
        let Some(material) = materials.get_mut(material_handle) else {
        	continue;
        };

        let distance = constellation_model.center.distance(cursor_global_pos);
        let exponent = -(2.0 * distance / CONS_VIEW_RADIUS).powi(2);
        let opa = E.powf(exponent);
        
        material.base_color = Color::srgba(opa, opa, opa, opa); // Set the alpha channel to adjust transparency

        if distance < closest_const_pos.distance(cursor_global_pos) {
        	closest_const_name = constellation_model.name.clone();
        	closest_const_pos = constellation_model.center;
        }
    }

    let Ok(mut info_label) = info_label_query.get_single_mut() else {
    	return;
    };
	
    info_label.sections[0].value = closest_const_name;
}
