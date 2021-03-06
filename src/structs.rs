#[derive(Debug)]
pub struct Vector2<T> {
	pub x: T,
	pub y: T
}

#[derive(Debug)]
pub struct Spaceship {
	pub position: Vector2<f32>
}

#[derive(Debug)]
pub struct Projectile {
	pub position: Vector2<f32>,
	pub velocity: Vector2<f32>
}

pub struct GameState {
	pub player: Spaceship,
	pub state: State,
	pub left_joystick: Vector2<f32>,
	pub right_joystick: Vector2<f32>,
	pub friendly_projectiles: Vec<Option<Projectile>>,
	pub enemies: Vec<Option<Spaceship>>,
	pub round_number: u32
}

pub enum State {
	Playing,
	StartMenu
}