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

const EASYNESS: f32 = 1.5;
const MAX_STAR_SIZE: f32 = 0.63;
const STAR_SCALE: f32 = 0.02;
const SKY_RADIUS: f32 = 4.0;

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
struct HealthLabel;

#[derive(Component)]
struct ScoreLabel;

#[derive(Component)]
struct ConstellationLine;

#[derive(Component)]
struct StartMenu;

#[derive(Component)]
struct MainGame;

#[derive(Component)]
struct GameOver;

#[derive(Component)]
struct Player {
	target_rotation: Option<Quat>,
	target_cons_name: Option<String>,
	score: usize,
	health: usize,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Start,
    Game,
    End,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Sky::default())
        .init_state::<GameState>()
        .add_systems(Startup, star_setup)
        .add_systems(Startup, cons_setup)
        .add_systems(OnEnter(GameState::Start), start_ui_setup)
        .add_systems(Update, start_menu_system.run_if(in_state(GameState::Start)))
        .add_systems(OnExit(GameState::Start), despawn_screen::<StartMenu>)
        .add_systems(OnEnter(GameState::Game), game_ui_setup)
        .add_systems(Update, player_input.run_if(in_state(GameState::Game)))
        .add_systems(Update, game_buttons.run_if(in_state(GameState::Game)))
        .add_systems(Update, label_update.run_if(in_state(GameState::Game)))
        .add_systems(OnExit(GameState::Game), despawn_screen::<MainGame>)
        .add_systems(OnEnter(GameState::End), end_setup)
        .add_systems(Update, end_buttons.run_if(in_state(GameState::End)))
        .add_systems(OnExit(GameState::End), despawn_screen::<GameOver>)
        .run();
}

fn end_setup() {
	info!("ending")
}

fn end_buttons(
	keys: Res<ButtonInput<KeyCode>>,
	mut game_state: ResMut<NextState<GameState>>
) {
	if keys.just_pressed(KeyCode::Space) {
		info!("start space");
		game_state.set(GameState::Start);
	}
}

fn start_ui_setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Create a container node that places its children (text areas) in a vertical column and centers them
    let container_node = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),  // Full width of the screen
            height: Val::Percent(100.0), // Full height of the screen
            flex_direction: FlexDirection::Column, // Arrange children in a column (vertical)
            justify_content: JustifyContent::Center, // Center vertically
            align_items: AlignItems::Center, // Center horizontally
            ..default()
        },
        ..default()
    };

    // Create the container for the text areas
    let container = commands.spawn(container_node).id();

    // TextStyle for the top text (larger font)
    let top_text_style = TextStyle {
        font_size: 50.0, // Larger font size
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Bold.ttf"), // Load font if needed
        ..default()
    };

    // TextStyle for the bottom text (smaller font)
    let bottom_text_style = TextStyle {
        font_size: 30.0, // Smaller font size
        color: Color::WHITE,
        // font: asset_server.load("fonts/FiraSans-Regular.ttf"), // Load font if needed
        ..default()
    };

    // TextBundle for the top text
    let top_text_node = TextBundle::from_section(
        "Astraea", // Text for the top section
        top_text_style,
    );

    // TextBundle for the bottom text
    let bottom_text_node = TextBundle::from_section(
        "Press Space to Begin", // Text for the bottom section
        bottom_text_style,
    );

    // Spawn the text nodes and add them as children to the container
    let top_text = commands.spawn((top_text_node, StartMenu)).id();
    let bottom_text = commands.spawn((bottom_text_node, StartMenu)).id();

    commands.entity(container).push_children(&[top_text, bottom_text]);

    commands.spawn((
       	Camera3dBundle {
   	        transform: Transform::from_xyz(0.0, 0.0, 0.0),
   	        ..default()
   	    },
   	    Player {
   	    	target_rotation: None,
   	    	target_cons_name: None,
   	    	score: 0,
   	    	health: 3,
   	    },
   	    GameOver,
   	));
}


fn spawn_cons_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sky: Res<Sky>,
   	target_constellation_name: String,
) {
    // Create a material for the line
    let line_material = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(0.5, 0.5, 1.0), // Red color for the line
        ..default()
    });

	let mut target_constellation = sky.content[0].clone();
    for constellation in sky.content.clone() {
    	if constellation.name == target_constellation_name {
    		target_constellation = constellation;
    	}
    }

    let mut vertices : Vec<Vec3> = vec![];

    for line in target_constellation.lines {
    	for star_index in line {
    		let star = target_constellation.stars[star_index as usize].clone();
    		vertices.push(celestial_to_cartesian(star.rah, star.dec));
    	}
    }

    // Create the mesh and add the vertices
    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    
    commands.spawn((
	    PbrBundle {
	        mesh: meshes.add(mesh),
	        material: line_material.clone(),
	        transform: Transform::default(), // Position and scale for the line
	        ..default()
	    },
	 	ConstellationLine,
	 	MainGame
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

    //let mesh = meshes.add(Cuboid::new(star_size, star_size, star_size));
	let star_mesh = meshes.add(Sphere::new(1.0).mesh().ico(3).unwrap());
    //let material = materials.add(Color::srgb(1.0, 1.0, 1.0));
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

fn game_ui_setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
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
        let button = commands.spawn((button_node, MainGame)).id();
        let button_text = commands.spawn((button_text_node, AnswerButton, MainGame)).id();

        commands.entity(button).push_children(&[button_text]);
        commands.entity(container).push_children(&[button]);
    }

    // Label style for top corners
    let label_style = Style {
        position_type: PositionType::Absolute, // Absolute positioning
        width: Val::Auto, // Auto width to fit text
        height: Val::Auto, // Auto height to fit text
        margin: UiRect::all(Val::Px(10.0)), // Margin around the text
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
            "* * *", // Text content
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
            "0000", // Text content
            TextStyle {
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        ..default()
    };

    // Spawn the top left and top right labels
    commands.spawn((top_left_label_node, MainGame, HealthLabel));
    commands.spawn((top_right_label_node, MainGame, ScoreLabel));
}


fn choose_constellation(
	player: &mut Player, 
	sky: Res<Sky>, // Res to access the Sky resource
    mut text_query: Query<&mut Text, With<AnswerButton>>,
    mut button_query: Query<(&mut BackgroundColor, &mut BorderColor), With<Button>>, // Query to reset button colors
	constellation_line_query : Query<(Entity, &ConstellationLine)>,
	mut commands: Commands,
) {
	if sky.content.len() >= 4 {
        let mut rng = rand::thread_rng();
        let mut selected_constellations = sky.content.clone();
        selected_constellations.shuffle(&mut rng);
        let constellations = &selected_constellations[0..4];

        let target_index = rng.next_u32().rem_euclid(4) as usize;
        let target_constellation = &constellations[target_index];
        let target_rotation = Quat::from_rotation_arc(
            Vec3::Z,
            -celestial_to_cartesian(target_constellation.rah, target_constellation.dec),
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

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform)>, // Query to get Player and Transform
    sky: Res<Sky>, // Res to access the Sky resource
    text_query: Query<&mut Text, With<AnswerButton>>,
    button_query: Query<(&mut BackgroundColor, &mut BorderColor), With<Button>>, // Query to reset button colors
	constellation_line_query : Query<(Entity, &ConstellationLine)>,
	commands: Commands,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        // If the space key was just pressed
        if keys.just_pressed(KeyCode::Space) || player.target_cons_name.is_none() {
            choose_constellation(&mut player, sky, text_query, button_query, constellation_line_query, commands);
			return
        }

        let mut rotation = Quat::IDENTITY;
        
        // Rotate left when the A key is pressed
        if keys.pressed(KeyCode::KeyA) {
            rotation *= Quat::from_rotation_y((PI / 60.0) as f32); // Rotate by 3 degrees (PI/60 radians)
        }

        // Rotate right when the D key is pressed
        if keys.pressed(KeyCode::KeyD) {
            rotation *= Quat::from_rotation_y((-PI / 60.0) as f32); // Rotate by -3 degrees
        }

        // Apply the rotation to the transform
        if rotation != Quat::IDENTITY {
            transform.rotation *= rotation; // Apply the rotation
            player.target_rotation = None;
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

fn start_menu_system(
	keys: Res<ButtonInput<KeyCode>>,
	mut game_state: ResMut<NextState<GameState>>,
	mut player_query: Query<(&mut Player, &mut Transform)>
) {
	if keys.just_pressed(KeyCode::Space) {
		info!("start space");
		game_state.set(GameState::Game);
	}


	if let Ok((_player, mut transform)) = player_query.get_single_mut() {
		let mut rotation = Quat::IDENTITY;
		rotation *= Quat::from_rotation_y((PI / 6000.0) as f32); // Rotate by 3 degrees (PI/60 radians)
	    rotation *= Quat::from_rotation_x((-PI / 2000.0) as f32); // Rotate by -3 degrees
	    transform.rotation *= rotation; // Apply the rotation
	}
}

fn label_update(
    mut param_set: ParamSet<(
        Query<&mut Text, With<HealthLabel>>,
        Query<&mut Text, With<ScoreLabel>>,
    )>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((player, _)) = player_query.get_single_mut() {
        // Update the health label
        if let Ok(mut health_text) = param_set.p0().get_single_mut() {
            health_text.sections[0].value = "# ".repeat(player.health);
        }

        // Update the score label
        if let Ok(mut score_text) = param_set.p1().get_single_mut() {
            score_text.sections[0].value = format!("{}", player.score);
        }
    }
}


fn game_buttons(
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
    sky: Res<Sky>,
    mut game_state: ResMut<NextState<GameState>>
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
		if let Ok(mut player) = player_query.get_single_mut() {
		    if let Some(target_cons) = player.target_cons_name.clone() {
		    	spawn_cons_lines(commands, meshes, materials, sky, target_cons.clone());
		    	
		    	if target_cons == selected_cons {
		    		info!("success");
		    		player.score += 100;
		    	} else {
		    		player.health -= 1;
		    		if player.health == 0 {
		    			info!("dead");
		    			game_state.set(GameState::End);
		    		}
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
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

