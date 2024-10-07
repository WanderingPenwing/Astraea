use bevy::prelude::*;
use bevy::math::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

mod end_state;
mod start_state;
mod game_state;
mod explo_state;

use game_state::GameData;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const WRONG_BUTTON: Color = Color::srgb(0.50, 0.15, 0.15);
const RIGHT_BUTTON: Color = Color::srgb(0.15, 0.50, 0.15);

const EASYNESS: f32 = 1.5;
const MAX_STAR_SIZE: f32 = 0.63;
const STAR_SCALE: f32 = 0.02;
const SKY_RADIUS: f32 = 4.0;
const CONS_VIEW_RADIUS: f32 = 0.8;

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

#[derive(Resource, Default, Clone)]
struct Sky {
    content: Vec<Constellation>,
}

impl Sky {
	fn as_string(&self) -> Vec<String> {
		let mut cons_names : Vec<String> = vec![];
		for cons in self.content.clone() {
			cons_names.push(cons.name.clone());
		}
		cons_names
	}

	fn get_constellation(&self, name: &str) -> Constellation {
		for cons in self.content.clone() {
			if cons.name == name {
				return cons;
			}
		}
		self.content[0].clone()
	} 
}

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
struct ConstellationModel {
	name: String,
	center: Vec3,
}

#[derive(Component)]
struct StartMenu;

#[derive(Component)]
struct MainGame;

#[derive(Component)]
struct GameOver;

#[derive(Component, Default)]
struct Player {
	target_rotation: Option<Quat>,
	dragging_pos: Option<Vec2>,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Start,
    Explo,
    Game,
    End,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Sky::default())
        .insert_resource(GameData::default())
        .init_state::<GameState>()
        .add_systems(Startup, star_setup)
        .add_systems(Startup, cons_setup)
        .add_systems(OnEnter(GameState::Start), start_state::setup)
        .add_systems(OnEnter(GameState::Start), start_state::audio_setup)
        .add_systems(Update, start_state::player_interact.run_if(in_state(GameState::Start)))
        .add_systems(OnExit(GameState::Start), despawn_screen::<StartMenu>)
        .add_systems(OnEnter(GameState::Game), game_state::setup)
        .add_systems(Update, game_state::player_interact.run_if(in_state(GameState::Game)))
        .add_systems(Update, explo_state::player_mouse_move.run_if(in_state(GameState::Game).or_else(in_state(GameState::Explo))))
		.add_systems(Update, explo_state::rotate_camera.run_if(in_state(GameState::Game).or_else(in_state(GameState::Explo)))) 
		.add_systems(Update, game_state::ui_buttons.run_if(in_state(GameState::Game)))
        .add_systems(Update, explo_state::constellation_opacity.run_if(in_state(GameState::Game).or_else(in_state(GameState::Explo))))
        .add_systems(Update, game_state::ui_labels.run_if(in_state(GameState::Game)))
        .add_systems(OnExit(GameState::Game), despawn_screen::<MainGame>)
        .add_systems(OnEnter(GameState::End), end_state::setup)
        .add_systems(Update, end_state::player_interact.run_if(in_state(GameState::End)))
        .add_systems(OnExit(GameState::End), despawn_screen::<GameOver>)
        .add_systems(OnEnter(GameState::Explo), explo_state::setup)
        .add_systems(Update, explo_state::player_interact.run_if(in_state(GameState::Explo))) 
        .add_systems(OnExit(GameState::Explo), despawn_screen::<MainGame>)
        .run();
}

fn spawn_cons_lines(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    target_constellation: Constellation,
) {
    let line_material = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(0.5, 0.5, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    
    let mut vertices : Vec<Vec3> = vec![];

    let mut avg_pos : Vec3 = Vec3::ZERO;
    let num_lines = target_constellation.lines.len();

    for line in target_constellation.lines {
    	for star_index in line {
    		let star = target_constellation.stars[star_index as usize].clone();
    		let star_pos = celestial_to_cartesian(star.rah, star.dec);
    		vertices.push(star_pos);
    		avg_pos += star_pos;
    	}
    }

    avg_pos /= 2.0 * num_lines as f32;

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    
    commands.spawn((
	    PbrBundle {
	        mesh: meshes.add(mesh),
	        material: line_material.clone(),
	        transform: Transform::default(),
	        ..default()
	    },
	 	ConstellationModel {
	 		name: target_constellation.name,
	 		center: avg_pos,
	 	},
	 	MainGame
	));
}

fn star_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ClearColor(Color::BLACK));

	let stars = get_stars().unwrap();

	let star_mesh = meshes.add(Sphere::new(1.0).mesh().ico(3).unwrap());
	
	let star_material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(1.0, 1.0, 1.0),
            ..default()
        });
    
	for star in stars {
		let star_pos = star_position(star.clone()) * SKY_RADIUS;
		let star_mag = star.v.parse::<f32>().unwrap();
        let mut star_size = STAR_SCALE * 2.512f32.powf(-star_mag*0.5);

        if star.constellation.is_some() {
        	star_size *= EASYNESS;
        }
        star_size = star_size.min(MAX_STAR_SIZE*STAR_SCALE);
        
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

    commands.spawn((
       	Camera3dBundle {
   	        transform: Transform::from_xyz(0.0, 0.0, 0.0),
   	        ..default()
   	    },
   	    Player::default(),
   	));
}

fn get_stars() -> std::io::Result<Vec<StarData>> {
    let data = include_str!("../data/stars.json");

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
	let data = include_str!("../data/constellations.json");

    let sky_data: Vec<Constellation> = serde_json::from_str(&data).unwrap();

    Ok(sky_data)
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

