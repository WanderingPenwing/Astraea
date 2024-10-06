use bevy::prelude::*;
use bevy::math::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use rand::seq::SliceRandom;
use rand::RngCore;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const WRONG_BUTTON: Color = Color::srgb(0.50, 0.15, 0.15);
const RIGHT_BUTTON: Color = Color::srgb(0.15, 0.50, 0.15);

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
    constellation: Option<String>,  
    #[serde(rename = "F")]
    f: Option<String>,
    #[serde(rename = "B")]
    bayer_designation: Option<String>,
    #[serde(rename = "N")]
    name: Option<String>,
}

#[derive(Resource, Default)]
struct Sky {
    content: Vec<Constellation>,
}

#[derive(Resource, Default)]
struct ShowConstellationEvent(bool);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Constellation {
	#[serde(rename = "Name")]
    name: String,
    #[serde(rename = "RAh")]
    rah: f64,
    #[serde(rename = "DEd")]
    dec: f64,
    stars: Vec<StarPos>,
    lines: Vec<[u32; 2]>,
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

#[derive(Component)]
struct Star;

#[derive(Component)]
struct AnswerButton;

#[derive(Component)]
struct ConstellationLine;

#[derive(Component)]
struct Player {
	target_rotation: Option<Quat>,
	target_cons_name: Option<String>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Sky::default())
		.insert_resource(ShowConstellationEvent::default())
        .add_systems(Startup, star_setup)
        .add_systems(Startup, cons_setup)
        .add_systems(Startup, ui_setup)
        .add_systems(Update, player_rotate)
        .add_systems(Update, button_system)
        .run();
}

fn spawn_cons_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sky: Res<Sky>,
   	constellation_name: String,
) {
	info!("show : {}", constellation_name);
    // Create a material for the line
    let line_material = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(1.0, 0.5, 0.5), // Red color for the line
        ..default()
    });

    // Define vertices for the line (two points in 3D space)
    let vertices = vec![
        [-1.0, -1.0, 0.0], // Starting point (origin)
        [-1.0, 1.0, 0.0], // Ending point
        [1.0, -1.0, 0.0], // Starting point (origin)
        [1.0, 1.0, 0.0], // Ending point
        [0.0, -1.0, 1.0], // Starting point (origin)
        [0.0, 1.0, 1.0], // Ending point
        [0.0, -1.0, -1.0], // Starting point (origin)
        [0.0, 1.0, -1.0], // Ending point
    ];

    // Create the mesh and add the vertices
    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    // (Optional) Define indices if you want more complex line patterns
    // mesh.set_indices(Some(Indices::U32(vec![0, 1])));

    // Spawn the mesh with the line material in the scene
    commands.spawn((
	    PbrBundle {
	        mesh: meshes.add(mesh),
	        material: line_material.clone(),
	        transform: Transform::default(), // Position and scale for the line
	        ..default()
	    },
	 	ConstellationLine
	));
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
	    	target_rotation: None,
	    	target_cons_name: None,
	    },
	));
}

fn get_stars() -> std::io::Result<Vec<StarData>> {
    let mut file = File::open("data/stars.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

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

fn cons_setup(mut sky: ResMut<Sky>) {
	sky.content = get_cons().unwrap();
}

fn get_cons() -> std::io::Result<Vec<Constellation>> {
	let mut file = File::open("data/constellations.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let sky_data: Vec<Constellation> = serde_json::from_str(&data).unwrap();

    Ok(sky_data)
}

fn ui_setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Create a container node that places its children (buttons) at the bottom of the screen
    let container_node = NodeBundle {
        style: Style {
            width: Val::Percent(100.0), // Full width of the screen
            height: Val::Percent(100.0), // Full height of the screen
            flex_direction: FlexDirection::Row, // Arrange children in a row (horizontal)
            justify_content: JustifyContent::Center, // Center horizontally
            align_items: AlignItems::FlexEnd, // Place at the bottom of the screen
            padding: UiRect::all(Val::Px(10.0)), // Optional padding
            ..default()
        },
        ..default()
    };

    // Button style (same for all buttons)
    let button_style = Style {
        width: Val::Px(150.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(10.0)), // Add margin between buttons
        justify_content: JustifyContent::Center, // Center text horizontally
        align_items: AlignItems::Center, // Center text vertically
        border: UiRect::all(Val::Px(5.0)),
        ..default()
    };

    // Create the container for the buttons
    let container = commands.spawn(container_node).id();

    // Function to create buttons with different text
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

        // Spawn the button and its text as children of the container
        let button = commands.spawn(button_node).id();
        let button_text = commands.spawn((button_text_node, AnswerButton)).id();

        commands.entity(button).push_children(&[button_text]);
        commands.entity(container).push_children(&[button]);
    }
}

fn player_rotate(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform)>, // Query to get Player and Transform
    sky: Res<Sky>, // Res to access the Sky resource
    mut text_query: Query<&mut Text, With<AnswerButton>>,
    mut button_query: Query<(&mut BackgroundColor, &mut BorderColor), With<Button>>, // Query to reset button colors
	constellation_line_query : Query<(Entity, &ConstellationLine)>,
	mut commands: Commands,
) {
    for (mut player, mut transform) in player_query.iter_mut() {
        // If the space key was just pressed
        if keys.just_pressed(KeyCode::Space) {
            if sky.content.len() >= 4 {
                let mut rng = rand::thread_rng();
                let mut selected_constellations = sky.content.clone();
                selected_constellations.shuffle(&mut rng);
                let constellations = &selected_constellations[0..4];

                let target_index = rng.next_u32().rem_euclid(4) as usize;
                let target_constellation = &constellations[target_index];
                let target_rotation = Quat::from_rotation_arc(
                    Vec3::Z,
                    celestial_to_cartesian(target_constellation.rah, target_constellation.dec),
                );

                player.target_rotation = Some(target_rotation);
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
            } else {
                info!("Not enough constellations in the sky (need 4)");
            }
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

fn button_system(
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
	mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sky: Res<Sky>,
) {
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
		let mut maybe_target_cons: Option<String> = None;

		for player in &mut player_query {
			maybe_target_cons = player.target_cons_name.clone();
	    }
	    
	    if let Some(target_cons) = maybe_target_cons {
	    	spawn_cons_lines(commands, meshes, materials, sky, target_cons.clone());
	    	
	    	if target_cons == selected_cons {
	    		info!("success");
	    	}

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



