//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
//use bevy::render::*;
use bevy::math::*;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use rand::seq::SliceRandom;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Resource, Default)]
struct Sky {
    content: Vec<Constellation>, // or use a specific array size, e.g., [String; 10]
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Constellation {
	#[serde(rename = "Name")]
    name: String,           // Name of the constellation
    #[serde(rename = "RAh")]
    rah: f64,               // Right Ascension of the constellation in hours
    #[serde(rename = "DEd")]
    dec: f64,               // Declination of the constellation in degrees
    stars: Vec<StarPos>,    // List of stars in the constellation
    lines: Vec<[u32; 2]>,   // Star connection lines as pairs of star IDs
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StarPos {
	id: usize,
	#[serde(rename = "bfID")]
	bfid: String,
	#[serde(rename = "RAh")]
	rah: f64,              
	#[serde(rename = "DEd")]
	dec: f64,               
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Sky::default())
        .add_systems(Startup, star_setup)
        .add_systems(Startup, cons_setup)
        .add_systems(Startup, ui_setup)
        //.add_systems(Update, button_interaction)
        .add_systems(Update, player_rotate)
        .run();
}


#[derive(Component)]
struct Star;

#[derive(Component)]
struct Player {
	target_rotation: Option<Quat>,
}

fn star_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.insert_resource(ClearColor(Color::BLACK));

	let stars = get_stars().unwrap();

    let star_scale = 0.02;
    let sky_radius = 4.0;

    //let mesh = meshes.add(Cuboid::new(star_size, star_size, star_size));
	let star_mesh = meshes.add(Sphere::new(1.0).mesh().ico(3).unwrap());
    //let material = materials.add(Color::srgb(1.0, 1.0, 1.0));
	let star_material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(1.0, 1.0, 1.0),
            ..default()
        });
    
	for star in stars {
		let star_pos = star_position(star.clone()) * sky_radius;
		let star_mag = star.v.parse::<f32>().unwrap();
        let mut star_size = star_scale * 2.512f32.powf(-star_mag*0.5);

        if star.constellation.is_some() {
        	star_size *= 1.5;
        }
        star_size = star_size.min(0.63*star_scale);
        
		commands.spawn((
			PbrBundle {
	            mesh: star_mesh.clone(),
	            material: star_material.clone(),
	            transform: Transform::from_xyz(star_pos.x, star_pos.y, star_pos.z)
	            	.with_scale(Vec3::splat(star_size)),
	            ..default()
	        },
            Star,
     	));
    }

    // camera
    commands.spawn((
    	Camera3dBundle {
	        transform: Transform::from_xyz(0.0, 0.0, 0.0),
	        ..default()
	    },
	    Player {
	    	target_rotation: Some(Quat::from_rotation_y(-1.5))
	    },
	));
}

fn cons_setup(mut sky: ResMut<Sky>) {
	info!("setup");

	sky.content = get_cons().unwrap();
}

fn get_stars() -> std::io::Result<Vec<StarData>> {
    let mut file = File::open("data/stars.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    info!("###");

    let stars: Vec<StarData> = serde_json::from_str(&data).unwrap();

    Ok(stars)
}

fn get_cons() -> std::io::Result<Vec<Constellation>> {
	let mut file = File::open("data/constellations.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    info!("###");

    let sky_data: Vec<Constellation> = serde_json::from_str(&data).unwrap();

    Ok(sky_data)
}

fn star_position(star_data: StarData) -> Vec3 {
    // Convert declination to decimal degrees
	let text_ra = star_data.ra;
	let text_dec = star_data.dec;
    
	let ra_seconds: f64 = 3600.0 * text_ra[0..2].parse::<f64>().unwrap()
        + 60.0 * text_ra[4..6].parse::<f64>().unwrap()
        + text_ra[8..12].parse::<f64>().unwrap();
        
    // Parse Dec
    let formated_dec = text_dec
                .replace("°", " ")
                .replace("′", " ")
                .replace("″", " ");
    let dec_parts: Vec<&str> = formated_dec.split_whitespace().collect();

    let dec_deg: f64 = dec_parts[0].parse::<f64>().unwrap()
    	+ dec_parts[1].parse::<f64>().unwrap() / 60.0
        + dec_parts[2].parse::<f64>().unwrap() / 3600.0;

    celestial_to_cartesian(ra_seconds/3600.0, dec_deg)
}

fn celestial_to_cartesian(rah: f64, ded: f64) -> Vec3 {
    let y_rot = 2.0 * PI * rah / 24.0;
    let x_rot = 2.0 * PI * ded / 360.0;

    let x : f32 = (y_rot.sin() * x_rot.cos()) as f32;
    let y : f32 = x_rot.sin() as f32;
    let z : f32 = (y_rot.cos() * x_rot.cos()) as f32;

    Vec3::new(x, y, z)
}

fn player_rotate(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>, // Query to get Player and Transform
    sky: Res<Sky>, // Res to access the Sky resource
    //mut commands: Commands,
) {
    for (mut player, mut transform) in query.iter_mut() {
        // If the space key was just pressed
        if keys.just_pressed(KeyCode::Space) {
            info!("space pressed");

            // Select a random constellation from the Sky's content
            if let Some(constellation) = sky.content.choose(&mut rand::thread_rng()) {
                // Create a target rotation quaternion from the constellation's direction
                let target_rotation = Quat::from_rotation_arc(Vec3::Z, celestial_to_cartesian(constellation.rah, constellation.dec));

                // Store the target rotation in the player component
                player.target_rotation = Some(target_rotation);
            }
        }

        // If there is a target rotation, smoothly rotate the player towards it
        if let Some(target_rotation) = player.target_rotation {
            // Get the current rotation of the player
            let current_rotation = transform.rotation;

            // Slerp between the current rotation and the target rotation
            transform.rotation = current_rotation.slerp(target_rotation, 0.1); // 0.1 is the interpolation factor

            // Optionally, you could clear the target rotation when close enough
            if transform.rotation.angle_between(target_rotation) < 0.01 {
                player.target_rotation = None; // Clear once the rotation is close enough
            }
        }
   }
}

fn ui_setup(
	mut commands: Commands, 
	//asset_server: Res<AssetServer>, 
	//mut materials: ResMut<Assets<ColorMaterial>>
) {
	let text = "Hello world!";
    
    commands.spawn(
        TextBundle::from_section(
            text,
            TextStyle {
                font_size: 100.0,
                color: Color::WHITE,
                ..default()
            },
        ) // Set the alignment of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        })
    );
}

// System to handle button interaction
// fn button_interaction(
//     mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
// ) {
//     for (interaction, mut color) in &mut interaction_query {
//         match *interaction {
//             Interaction::Pressed => {
//                 *color = Color::srgb(0.35, 0.75, 0.35).into(); // Change color on click
//                 info!("Button clicked!");
//             }
//             Interaction::Hovered => {
//                 *color = Color::srgb(0.25, 0.25, 0.25).into(); // Change color on hover
//             }
//             Interaction::None => {
//                 *color = Color::srgb(0.15, 0.15, 0.15).into(); // Default color
//             }
//         }
//     }
// }


