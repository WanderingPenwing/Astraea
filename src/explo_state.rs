use bevy::prelude::*;

use crate::Player;

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
