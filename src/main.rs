//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
//use bevy::render::*;
use bevy::math::*;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct StarData {
	#[serde(rename = "Dec")]
    dec: String,
    #[serde(rename = "HR")]
    hr: String,
    #[serde(rename = "K")]
    k: Option<String>,
    #[serde(rename = "RA")]
    ra: String,
    #[serde(rename = "V")]
    v: String,
    #[serde(rename = "C")]
    constellation: Option<String>,  // Optional field
    #[serde(rename = "F")]
    f: Option<String>,              // Optional field
    #[serde(rename = "B")]
    bayer_designation: Option<String>, // Optional field
    #[serde(rename = "N")]
    name: Option<String>,            // Optional field
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct Star;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane

	let stars = get_stars().unwrap();

	for star in stars {
        info!("{:?}", star);
    }

	
    let star_size = 0.02;
    commands.spawn((
        PbrBundle {//Plane3d::default().mesh().size(1., 1.)
            mesh: meshes.add(Cuboid::new(star_size, star_size, star_size)),
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
            transform: Transform::from_xyz(1.0, 0.0, 0.0),
            ..default()
        },
        Star,
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-1.5)),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-1.5)),
        ..default()
    });
}

fn get_stars() -> std::io::Result<Vec<StarData>> {
    let mut file = File::open("data/stars.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    info!("###");

    let stars: Vec<StarData> = serde_json::from_str(&data).unwrap();

    Ok(stars)
}

// fn star_position_to_spherical(ra_hours: f32, dec_deg: f32, dec_min: f32, dec_sec: f32) -> Vec3 {
//     // Convert declination to decimal degrees
//     let dec_decimal = declination_to_decimal(dec_deg, dec_min, dec_sec);
//     
//     // Convert Right Ascension from hours to degrees
//     let ra_degrees = right_ascension_to_degrees(ra_hours);
// 
//     // Convert to spherical coordinates
//     let theta = ra_degrees.to_radians(); // RA as theta (azimuthal angle)
//     let phi = (90.0 - dec_decimal).to_radians(); // Declination to phi (polar angle)
// 
//     // Assuming a unit sphere, the radius (r) is 1. Calculate Cartesian coordinates.
//     let x = phi.sin() * theta.cos();
//     let y = phi.sin() * theta.sin();
//     let z = phi.cos();
// 
//     Vec3::new(x, y, z)
// }


