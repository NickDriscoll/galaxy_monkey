extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::GameControllerSubsystem;
use sdl2::controller::GameController;
use sdl2::controller::Button;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::controller::Axis;
use sdl2::ttf;
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use std::i16;
use std::thread::sleep;
use std::time::Duration;
use crate::structs::Vector2;
use crate::structs::Spaceship;
use crate::structs::Projectile;
use crate::structs::GameState;
use crate::structs::State;

mod structs;
//mod subroutines;

const DEADZONE: f32 = 0.20;
const PLAYER_WIDTH: u32 = 50;
const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;

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

fn text_texture<'a>(text: &str, texture_creator: &'a TextureCreator<WindowContext>, font: &Font) -> Texture<'a> {
	let color = Color::RGB(0, 255, 0);
	match font.render(text).solid(color) {
		Ok(surface) => {
			match texture_creator.create_texture_from_surface(surface) {
				Ok(t) => {
					t
				}
				Err(e) => {
					panic!("{}", e);
				}
			}
		}
		Err(e) => {
			panic!("{}", e);
		}
	}
}

fn obtain_result<T, E: std::fmt::Display>(res: Result<T, E>) -> T {
	match res {
		Ok(r) => {
			r
		}
		Err(e) => {
			panic!("{}", e);
		}
	}
}

fn draw_centered_text(canvas: &mut Canvas<Window>, texture: &Texture, y_offset: i32) {
	//Draw the title
	let dst = {
		let query = texture.query();
		let xpos = (SCREEN_WIDTH / 2 - query.width / 2) as i32;
		let ypos = (SCREEN_HEIGHT / 2 - query.height / 2) as i32 + y_offset;
		Rect::new(xpos, ypos, query.width, query.height)
	};
	canvas.copy(texture, None, dst).unwrap();
}

fn delete_marked_entities<T>(optionvec: &mut Vec<Option<T>>, marks: Vec<usize>) {
	for i in marks {
		optionvec[i] = None;
	}
}

fn insert_into_option_vec<T>(optionvec: &mut Vec<Option<T>>, item: T) {
	let mut index = None;
	for (i, p) in optionvec.iter().enumerate() {
		if let None = p {
			index = Some(i);
		}
	}

	match index {
		Some(i) => {
			optionvec[i] = Some(item);
		}
		None => {
			optionvec.push(Some(item));
		}
	}
}

fn clamp<T: std::cmp::PartialOrd>(value: T, lower_bound: T, upper_bound: T) -> T{
	let mut clamped_value = value;
	if clamped_value < lower_bound {
		clamped_value = lower_bound;
	}
	if clamped_value > upper_bound {
		clamped_value = upper_bound;
	}
	clamped_value
}

fn main() {
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();

	//Create the actual window
	let window = video_subsystem.window("Galaxy Monkey", SCREEN_WIDTH, SCREEN_HEIGHT).position_centered().build().unwrap();

	//Create primary drawing interface
	let mut canvas = window.into_canvas().build().unwrap();

	//Create the texture creator
	let texture_creator = canvas.texture_creator();

	//Create the event_pump
	let mut event_pump = sdl_context.event_pump().unwrap();

	//Init the controller subsystem
	let controller_ss = sdl_context.game_controller().unwrap();

	//Init the timer subsystem
	let mut timer_ss = sdl_context.timer().unwrap();

	//Create array of controllers
	let mut _controllers: [Option<GameController>; 4] = [None, None, None, None];

	//Init the ttf subsystem
	let ttf_context = obtain_result(ttf::init());

	//Load the font
	let font = obtain_result(ttf_context.load_font("fonts/CursedTimerULiL.ttf", 64));

	//Create title screen texture
	let game_title = text_texture("Galaxy Monkey", &texture_creator, &font);

	//Create press start text
	let press_start_text = text_texture("Press Start", &texture_creator, &font);
	let mut press_start_position: i32 = 150;

	//Round # texture variable
	let mut round_number_texture = text_texture("Round 0", &texture_creator, &font);

	//Timer variable for making "Press Start" flash
	let mut press_start_timer = 0;
	let mut displaying = true;

	//Timer variable for transitioning between rounds
	let mut round_transition_timer = 0;
	let mut going_to_next_round = false;

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
			let x = (SCREEN_WIDTH / 2 - PLAYER_WIDTH) as f32;
			let y = (SCREEN_HEIGHT / 2 - PLAYER_WIDTH) as f32;
			let position = Vector2 {
				x,
				y
			};
			Spaceship {
				position
			}
		};

		let friendly_projectiles = Vec::new();
		let enemies = Vec::new();

		let round_number = 0;

		GameState {
			player,
			state: State::StartMenu,
			left_joystick,
			right_joystick,
			friendly_projectiles,
			enemies,
			round_number
		}
	};

	let mut old_ticks = 0;

	'running: loop {
		//Get milliseconds since last frame
		let ticks = timer_ss.ticks();
		let time_delta = ticks - old_ticks;

		match game_state.state {
			State::StartMenu => {
				for event in event_pump.poll_iter() {
					match event {
						Event::Quit {..} |
						Event::ControllerButtonDown {button: Button::Back, ..} => {
							break 'running;
						}
						Event::ControllerButtonDown {button: Button::Start, ..} |
						Event::KeyDown {..} => {
							game_state.state = State::Playing;
						}
						Event::JoyDeviceAdded {which: i, ..} => {
							_controllers[i as usize] = open_controller(&controller_ss, i);
						}
						Event::MouseWheel {y, ..} => {
							press_start_position -= y * 30;
						}
						_ => {}
					}
				}

				//Clear the screen
				canvas.set_draw_color(Color::RGB(0, 0, 0));
				canvas.clear();

				//Draw the title
				draw_centered_text(&mut canvas, &game_title, -200);

				//Draw press start
				const INTERVAL: u32 = 500;
				if ticks - press_start_timer > INTERVAL {
					displaying = !displaying;
					press_start_timer = ticks;
				}

				if displaying {
					draw_centered_text(&mut canvas, &press_start_text, press_start_position);
				}
			}
			State::Playing => {
				//Process events
				for event in event_pump.poll_iter() {
					match event {
						Event::Quit {..} |
						Event::ControllerButtonDown {button: Button::Back, ..} => {
							break 'running;
						}
						Event::JoyDeviceAdded {which: i, ..} => {
							_controllers[i as usize] = open_controller(&controller_ss, i);
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
						Event::KeyDown {keycode: Some(key), ..} => {
							match key {
								_ => {
									println!("You pressed the unbound key: {}", key);
								}
							}
						}
						_ => {}
					}
				}

				//Check if enemies option-vec is empty
				let mut enemies_is_empty = true;
				for enemy in game_state.enemies.iter() {
					if let Some(_e) = enemy {
						enemies_is_empty = false;
						break;
					}
				}

				//This will probably become the trigger for advancing rounds
				if enemies_is_empty {
					if !going_to_next_round {
						//Start the timer
						round_transition_timer = ticks;

						//Increment round number
						game_state.round_number += 1;

						//Create round # texture
						round_number_texture = text_texture(&format!("Round {}", game_state.round_number), &texture_creator, &font);

						going_to_next_round = true;
					}

					const INTERVAL: u32 = 2500; //Timer duration in millis
					if ticks - round_transition_timer > INTERVAL {
						let new_enemy = {
							let position = Vector2 {
								x: 0.0,
								y: 30.0
							};

							Spaceship {
								position
							}
						};

						//Insert enemy into vec
						insert_into_option_vec(&mut game_state.enemies, new_enemy);
						going_to_next_round = false;
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

						const PROJECTILE_SPEED: f32 = 10.0;
						let angle = f32::atan(game_state.right_joystick.y / game_state.right_joystick.x);

						let xvel = {
							if game_state.right_joystick.x < 0.0 {
								-(PROJECTILE_SPEED * f32::cos(angle))
							} else {
								PROJECTILE_SPEED * f32::cos(angle)
							}
						};

						let yvel = {
							if game_state.right_joystick.x < 0.0 {
								-(PROJECTILE_SPEED * f32::sin(angle))
							} else {
								PROJECTILE_SPEED * f32::sin(angle)
							}
						};

						let velocity = Vector2 {
							x: xvel,
							y: yvel
						};

						Projectile {
							position,
							velocity
						}
					};

					//Insert new projectile into vec
					insert_into_option_vec(&mut game_state.friendly_projectiles, projectile);
				}

				//Update the player
				const PLAYER_SPEED: f32 = 3.0;
				game_state.player.position.x += game_state.left_joystick.x * PLAYER_SPEED;
				game_state.player.position.y += game_state.left_joystick.y * PLAYER_SPEED;

				game_state.player.position.x = clamp(game_state.player.position.x, 0.0, (SCREEN_WIDTH - PLAYER_WIDTH) as f32);
				game_state.player.position.y = clamp(game_state.player.position.y, 0.0, (SCREEN_HEIGHT - PLAYER_WIDTH) as f32);

				//Update all enemies
				let mut enemies_to_destroy = Vec::new();
				for (i, enemy) in game_state.enemies.iter_mut().enumerate() {
					if let Some(e) = enemy {
						if e.position.x > SCREEN_WIDTH as f32 {
							enemies_to_destroy.push(i);
						}

						e.position.x += 1.0;
					}
				}

				//Set all offscreen enemies to None
				delete_marked_entities(&mut game_state.enemies, enemies_to_destroy);

				//Update all projectiles
				let mut projectiles_to_destroy = Vec::new();
				for (i, projectile) in game_state.friendly_projectiles.iter_mut().enumerate() {
					if let Some(p) = projectile {
						if p.position.x < 0.0 || p.position.x > SCREEN_WIDTH as f32 ||
						   p.position.y < 0.0 || p.position.y > SCREEN_HEIGHT as f32 {
							projectiles_to_destroy.push(i);
						}

						p.position.x += p.velocity.x;
						p.position.y += p.velocity.y;
					}
				}

				//Set all offscreen projectiles to None
				delete_marked_entities(&mut game_state.friendly_projectiles, projectiles_to_destroy);

				//Clear the canvas
				canvas.set_draw_color(Color::RGB(0, 0, 0));
				canvas.clear();

				//Draw the spaceship
				canvas.set_draw_color(Color::RGB(150, 150, 150));
				canvas.fill_rect(Rect::new(game_state.player.position.x as i32, game_state.player.position.y as i32, PLAYER_WIDTH, PLAYER_WIDTH)).unwrap();

				//Draw all enemies
				canvas.set_draw_color(Color::RGB(50, 120, 0));
				for enemy in game_state.enemies.iter() {
					if let Some(e) = enemy {
						canvas.fill_rect(Rect::new(e.position.x as i32, e.position.y as i32, PLAYER_WIDTH, PLAYER_WIDTH)).unwrap();
					}
				}

				//Draw all projectiles
				canvas.set_draw_color(Color::RGB(150, 150, 150));
				for projectile in game_state.friendly_projectiles.iter() {
					if let Some(p) = projectile {
						let point = Point::new(p.position.x as i32, p.position.y as i32);
						canvas.draw_point(point).unwrap();
					}
				}

				//Draw the round transition text if necessary
				if going_to_next_round {
					draw_centered_text(&mut canvas, &round_number_texture, 0);
				}
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
