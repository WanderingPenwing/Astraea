use bevy::prelude::*;

fn main() {
    App::new()
    	.add_plugins(DefaultPlugins)
    	.add_systems(Startup, setup)
    	.add_systems(Update, move_player)
    	.run();
}

#[derive(Bundle)]
struct PlayerBundle {
	core: Player,
    sprite: SpriteBundle,
}

#[derive(Component)]
struct Player;


fn setup(
	//mut materials: ResMut<Assets<ColorMaterial>>,
	//mut meshes: ResMut<Assets<Mesh>>,
	mut commands: Commands,
	asset_server: Res<AssetServer>
) {
	commands.spawn(Camera2dBundle::default());
	commands.spawn(PlayerBundle {
		core: Player,
		sprite: SpriteBundle {
			texture: asset_server.load("../assets/test.png"),
			..default()
		},
	});
}

const MOVE_SPEED: f32 = 6.0;

fn move_player(
	mut transforms: Query<&mut Transform, With<Player>>,
	keys: Res<ButtonInput<KeyCode>>,
) {
	for mut transform in transforms.iter_mut() {
		let mut direction = Vec3::ZERO;

		if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
		if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
		if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
		if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }
		
		if 0.0 < direction.length() {
			transform.translation += MOVE_SPEED * direction.normalize();
		}
	}
}


