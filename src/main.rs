extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::GameControllerSubsystem;
use sdl2::controller::GameController;
use sdl2::controller::Button;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::controller::Axis;
use std::i16;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug)]
struct Vector2<T> {
	x: T,
	y: T
}

struct Spaceship {
	position: Vector2<f32>
}

#[derive(Debug)]
struct Projectile {
	position: Vector2<f32>,
	velocity: Vector2<f32>
}

struct GameState {
	player: Spaceship,
	left_joystick: Vector2<f32>,
	right_joystick: Vector2<f32>,
	friendly_projectiles: Vec<Option<Projectile>>
}

const DEADZONE: f32 = 0.15;
const PLAYER_WIDTH: u32 = 50;

fn open_controller(css: &GameControllerSubsystem, index: u32) -> Option<GameController> {
	match css.open(index) {
		Ok(cont) => {
			println!("Successfully opened controller {}", index);
			Some(cont)
		}
		Err(_e) => {
			println!("Unable to open controller {}", index);
			None
		}
	}
}

fn check_deadzone(mut stick: Vector2<f32>) -> Vector2<f32> {
	if stick.x > -DEADZONE && stick.x < DEADZONE && stick.y > -DEADZONE && stick.y < DEADZONE {
		stick.x = 0.0;
		stick.y = 0.0;
	}
	stick
}

fn main() {	
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();

	//Create the actual window
	const SCREEN_WIDTH: u32 = 1280;
	const SCREEN_HEIGHT: u32 = 720;
	let window = video_subsystem.window("Galaxy Monkey", SCREEN_WIDTH, SCREEN_HEIGHT).position_centered().build().unwrap();

	//Create primary drawing interface
	let mut canvas = window.into_canvas().build().unwrap();

	//Create the event_pump
	let mut event_pump = sdl_context.event_pump().unwrap();

	//Init the controller subsystem
	let controller_ss = sdl_context.game_controller().unwrap();

	//Init the timer subsystem
	let mut timer_ss = sdl_context.timer().unwrap();

	let mut _controller = open_controller(&controller_ss, 0);

	//Initialize the game state
	let mut game_state = {
		let left_joystick = Vector2 {
			x: 0.0,
			y: 0.0
		};

		let right_joystick = Vector2 {
			x: 0.0,
			y: 0.0
		};

		let player = {
			let position = Vector2 {
				x: 200.0,
				y: 200.0
			};
			Spaceship {
				position
			}
		};

		let friendly_projectiles = Vec::new();

		GameState {
			player,
			left_joystick,
			right_joystick,
			friendly_projectiles
		}
	};

	let mut old_ticks = 0;

	'running: loop {
		//Get milliseconds since last frame
		let ticks = timer_ss.ticks();
		let time_delta = ticks - old_ticks;

		//Process events
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} => {
					break 'running;
				}
				Event::JoyDeviceAdded {which: i, ..} => {
					if i == 0 {
						_controller = open_controller(&controller_ss, i);	
					}					
				}
				Event::ControllerAxisMotion {axis: ax, value: v, ..} => {
					match ax {
						Axis::LeftX => {
							game_state.left_joystick.x = v as f32 / i16::MAX as f32;						
						}
						Axis::LeftY => {
							game_state.left_joystick.y = v as f32 / i16::MAX as f32;
						}
						Axis::RightX => {
							game_state.right_joystick.x = v as f32 / i16::MAX as f32;
						}
						Axis::RightY => {
							game_state.right_joystick.y = v as f32 / i16::MAX as f32;
						}
						_ => {}
					}
					game_state.left_joystick = check_deadzone(game_state.left_joystick);
					game_state.right_joystick = check_deadzone(game_state.right_joystick);
				}
				Event::ControllerButtonDown {button: Button::Back, ..} => {
					break 'running;
				}
				Event::KeyDown {keycode: Some(key), ..} => {
					println!("You just pressed {}", key);
				}
				_ => {}
			}
		}

		//If the right stick is not neutral, fire a projectile
		if game_state.right_joystick.x != 0.0 || game_state.right_joystick.y != 0.0 {
			//Construct this new projectile
			let projectile = {
				let xpos = game_state.player.position.x + (PLAYER_WIDTH / 2) as f32;
				let ypos = game_state.player.position.y + (PLAYER_WIDTH / 2) as f32;
				let position = Vector2 {
					x: xpos,
					y: ypos
				};

				const PROJECTILE_SPEED: f32 = 3.0;
				let xvel = game_state.right_joystick.x * PROJECTILE_SPEED;
				let yvel = game_state.right_joystick.y * PROJECTILE_SPEED;
				let velocity = Vector2 {
					x: xvel,
					y: yvel
				};

				Projectile {
					position,
					velocity
				}
			};

			//Check the friendly projectile Vec for an empty slot, push otherwise
			let mut index: Option<usize> = None;
			for (i, p) in game_state.friendly_projectiles.iter().enumerate() {
				match p {
					None => {
						index = Some(i);
					}
					_ => {}
				}
			}

			match index {
				Some(i) => {
					game_state.friendly_projectiles[i] = Some(projectile);
				}
				None => {
					game_state.friendly_projectiles.push(Some(projectile));
				}
			}
		}

		//Update the player
		const PLAYER_SPEED: f32 = 3.0;
		game_state.player.position.x += game_state.left_joystick.x * PLAYER_SPEED;
		game_state.player.position.y += game_state.left_joystick.y * PLAYER_SPEED;

		//Update all projectiles
		for projectile in game_state.friendly_projectiles.iter_mut() {
			match projectile {
				Some(p) => {
					p.position.x += p.velocity.x;
					p.position.y += p.velocity.y;
				}
				None => {}
			}
		}

		//Clear the canvas		
		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();

		//Draw the spaceship
		canvas.set_draw_color(Color::RGB(150, 150, 150));
		canvas.fill_rect(Rect::new(game_state.player.position.x as i32, game_state.player.position.y as i32, PLAYER_WIDTH, PLAYER_WIDTH)).unwrap();

		//Draw all projectiles
		//canvas.set_draw_color(Color::RGB(255, 255, 255));
		for projectile in game_state.friendly_projectiles.iter() {
			match projectile {
				Some(p) => {
					let point = Point::new(p.position.x as i32, p.position.y as i32);
					canvas.draw_point(point);
				}
				None => {}
			}
		}

		canvas.present();

		//Update old_ticks
		old_ticks = ticks;

		if time_delta < 8 {
			sleep(Duration::from_millis((8 - time_delta) as u64));
		}		
	}
}
