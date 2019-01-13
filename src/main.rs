extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::GameControllerSubsystem;
use sdl2::controller::GameController;
use sdl2::rect::Rect;
use std::i16;
use std::thread::sleep;
use std::time::Duration;

struct Vector2<T> {
	x: T,
	y: T
}

struct Spaceship {
	position: Vector2<f32>
}

struct GameState {
	player: Spaceship,
	left_joystick: Vector2<f32>
}

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
	let controller_ss = match sdl_context.game_controller() {
		Ok(gcss) => {
			gcss
		}
		Err(e) => {
			panic!("{}", e);
		}
	};

	//Init the timer subsystem
	let mut timer_ss = match sdl_context.timer() {
		Ok(tss) => {
			tss
		}
		Err(e) => {
			panic!("{}", e);
		}
	};

	let mut controller = open_controller(&controller_ss, 0);

	let left_stick_state = Vector2 {
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

	let mut game_state = GameState {
		player,
		left_joystick: left_stick_state
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
					println!("This event");
					if i == 0 {
						controller = open_controller(&controller_ss, i);	
					}					
				}
				Event::JoyAxisMotion {axis_idx: axid, value: v, ..} => {
					const DEADZONE: f32 = 0.1;
					if axid == 0 {
						game_state.left_joystick.x = v as f32 / i16::MAX as f32;

						//Deadzone check
						if game_state.left_joystick.x > -DEADZONE && game_state.left_joystick.x < DEADZONE {
							game_state.left_joystick.x = 0.0;
						}
					} else if axid == 1 {
						game_state.left_joystick.y = v as f32 / i16::MAX as f32;

						//Deadzone check
						if game_state.left_joystick.y > -DEADZONE && game_state.left_joystick.y < DEADZONE {
							game_state.left_joystick.y = 0.0;
						}
					}
					println!("({}, {})", game_state.left_joystick.x, game_state.left_joystick.y);
				}
				Event::ControllerButtonDown {button: b, ..} => {
					println!("You just pressed {}", b.string());
				}
				Event::KeyDown {keycode: Some(key), ..} => {
					println!("You just pressed {}", key);
				}
				_ => {}
			}
		}

		//Update the player
		const PLAYER_SPEED: u32 = 3;
		game_state.player.position.x += game_state.left_joystick.x * PLAYER_SPEED as f32;
		game_state.player.position.y += game_state.left_joystick.y * PLAYER_SPEED as f32;

		//Clear the canvas		
		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();

		//Draw the spaceship
		canvas.set_draw_color(Color::RGB(150, 150, 150));
		canvas.fill_rect(Rect::new(game_state.player.position.x as i32, game_state.player.position.y as i32, 50, 50)).unwrap();

		canvas.present();

		//Update old_ticks
		old_ticks = ticks;

		if time_delta < 8 {
			sleep(Duration::from_millis((8 - time_delta) as u64));
		}		
	}
}
