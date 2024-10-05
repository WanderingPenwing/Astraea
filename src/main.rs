//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy::render::*;
use bevy::math::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

	let star_material = materials.add(Color::WHITE);

	let star = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)));
	
    // circular base
    commands.spawn(MaterialMeshBundle {
        mesh: star,
        material: star_material.clone(),
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        ..default()
    });
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.8, 1.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
