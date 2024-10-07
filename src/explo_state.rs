use bevy::prelude::*;

use crate::Player;
use crate::GameState;

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
    let angle_of_rotation = dot_product.acos() * 6.0;

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
