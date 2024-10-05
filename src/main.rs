//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
//use bevy::render::*;
use bevy::math::*;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

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

    let star_size = 0.02;
    let sky_radius = 2.0;

    let mesh = meshes.add(Cuboid::new(star_size, star_size, star_size));
    let material = materials.add(Color::srgb(1.0, 1.0, 1.0));
    

	for star in stars {
		info!("{:?}", star);

		let star_pos = star_position(star) * sky_radius;
        info!("{:?}", star_pos);

		
		commands.spawn((
			PbrBundle {
	            mesh: mesh.clone(),
	            material: material.clone(),
	            transform: Transform::from_xyz(star_pos.x, star_pos.y, star_pos.z),
	            ..default()
	        },
            Star,
     	));
    }

	
    // commands.spawn((
    //     PbrBundle {//Plane3d::default().mesh().size(1., 1.)
    //         mesh: meshes.add(Cuboid::new(star_size, star_size, star_size)),
    //         material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
    //         transform: Transform::from_xyz(1.0, 0.0, 0.0),
    //         ..default()
    //     },
    //     Star,
    // ));

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

fn star_position(star_data: StarData) -> Vec3 {
    // Convert declination to decimal degrees
	let text_ra = star_data.ra;
	let text_dec = star_data.dec;
    
	let ra_seconds: f64 = 3600.0 * text_ra[0..2].parse::<f64>().unwrap()
        + 60.0 * text_ra[4..6].parse::<f64>().unwrap()
        + text_ra[8..12].parse::<f64>().unwrap();

	info!(text_dec);
    // Parse Dec
    let formated_dec = text_dec
                .replace("°", " ")
                .replace("′", " ")
                .replace("″", " ");
    let dec_parts: Vec<&str> = formated_dec.split_whitespace().collect();

    let dec_deg: f64 = dec_parts[0].parse::<f64>().unwrap()
    	+ dec_parts[1].parse::<f64>().unwrap() / 60.0
        + dec_parts[2].parse::<f64>().unwrap() / 3600.0;

	// let dec_sign : f64 = if text_dec.starts_with('-') {
	// 	-1.0
	// } else {
	// 	1.0
	// };

    return celestial_to_cartesian(ra_seconds/3600.0, dec_deg)
}

fn celestial_to_cartesian(rah: f64, ded: f64) -> Vec3 {
    let y_rot = 2.0 * PI * rah / 24.0;
    let x_rot = 2.0 * PI * ded / 360.0;

    let x : f32 = (y_rot.sin() * x_rot.cos()) as f32;
    let y : f32 = x_rot.sin() as f32;
    let z : f32 = (y_rot.cos() * x_rot.cos()) as f32;

    Vec3::new(x, y, z)
}


