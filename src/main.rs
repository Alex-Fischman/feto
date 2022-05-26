mod object;
mod render;
mod spells;
mod vector;

use object::{Object, Shape};
use render::Vertex;
use std::collections::HashMap;
use vector::Vector;
use winit::event::{MouseButton, VirtualKeyCode};

const TICKRATE: f32 = 100.0;
const GRAVITY: f32 = 10.0;
const JUMP: f32 = 3.0;
const MOVE: f32 = 1.0;
const BASE_SPELL_SPEED: f32 = 5.0;
const BASE_SPELL_RANGE: f32 = 2.0;
const CEILING_BOUNCE: f32 = -0.01;
const PLAYER_HEIGHT: f32 = 0.2;
const GROUND_CHECK: f32 = 0.0001;

type Id = usize;
type System<Data> = HashMap<Id, Data>;

struct World {
	player_id: Id,
	ground_ids: Vec<Id>,
	total_ids: Id,

	objects: System<Object>,
	colors: System<render::Color>,
	spells: System<spells::Spell>,

	elements: Vec<spells::Element>,
	buttons: HashMap<Button, ButtonState>,
	mouse: Vector,
}

#[derive(PartialEq, Eq, Hash)]
enum Button {
	Key(VirtualKeyCode),
	Mouse(MouseButton),
}

#[derive(Clone, Copy, Debug)]
enum ButtonState {
	NotHeld,
	HeldUnreadPress,
	HeldReadPress,
}

impl World {
	fn is_button_held(&self, button: Button) -> bool {
		match self.buttons.get(&button) {
			Some(ButtonState::HeldUnreadPress) | Some(ButtonState::HeldReadPress) => true,
			_ => false,
		}
	}

	fn is_button_pressed(&mut self, button: Button) -> bool {
		match self.buttons.get(&button) {
			Some(ButtonState::HeldUnreadPress) => {
				self.buttons.insert(button, ButtonState::HeldReadPress);
				true
			}
			_ => false,
		}
	}

	fn update_button(&mut self, button: Button, state: winit::event::ElementState) {
		let old = *self.buttons.get(&button).unwrap_or(&ButtonState::NotHeld);
		self.buttons.insert(
			button,
			match state {
				winit::event::ElementState::Pressed => match old {
					ButtonState::NotHeld => ButtonState::HeldUnreadPress,
					ButtonState::HeldUnreadPress => ButtonState::HeldUnreadPress,
					ButtonState::HeldReadPress => ButtonState::HeldReadPress,
				},
				winit::event::ElementState::Released => ButtonState::NotHeld,
			},
		);
	}

	fn update(&mut self) {
		let delta_time = 1.0 / TICKRATE;
		use winit::event::VirtualKeyCode::*;

		let vx = MOVE
			* if self.is_button_held(Button::Key(A)) {
				-1.0
			} else if self.is_button_held(Button::Key(D)) {
				1.0
			} else {
				0.0
			};

		let Object { pos: player_pos, vel: player_vel, .. } =
			self.objects.get(&self.player_id).unwrap();
		let player_pos = player_pos.clone();
		let player_vel = player_vel.clone();
		let ground_objects: Vec<Object> =
			self.ground_ids.iter().map(|id| self.objects.get(id).unwrap().clone()).collect();

		let on_ceiling = ground_objects.iter().any(|g| {
			object::collide(
				&Object::new(
					player_pos.x,
					player_pos.y,
					Shape::Line(Vector::new(0.0, PLAYER_HEIGHT / 2.0 + GROUND_CHECK)),
				),
				g,
			)
		});
		let on_ground = ground_objects.iter().any(|g| {
			object::collide(
				&Object::new(
					player_pos.x,
					player_pos.y,
					Shape::Line(Vector::new(0.0, -(PLAYER_HEIGHT / 2.0 + GROUND_CHECK))),
				),
				g,
			)
		});

		*self.colors.get_mut(&self.player_id).unwrap() = if on_ground {
			[0.0, 1.0, 0.0]
		} else if on_ceiling {
			[1.0, 0.0, 0.0]
		} else {
			[0.0, 0.0, 1.0]
		};

		let vy = if self.is_button_held(Button::Key(Space)) && on_ground {
			JUMP
		} else if on_ground {
			0.0
		} else if on_ceiling {
			CEILING_BOUNCE
		} else {
			player_vel.y - GRAVITY * delta_time
		};

		self.objects.get_mut(&self.player_id).unwrap().vel = Vector::new(vx, vy);
		self.objects
			.get_mut(&self.player_id)
			.unwrap()
			.move_and_collide(&ground_objects, delta_time);

		for key in [Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0] {
			if self.is_button_pressed(Button::Key(key)) {
				let element = match key {
					Key1 => spells::Element::Earth,
					Key2 => spells::Element::Water,
					Key3 => spells::Element::Air,
					Key4 => spells::Element::Fire,
					Key5 => spells::Element::Acid,
					Key6 => spells::Element::Pressure,
					Key7 => spells::Element::Shock,
					Key8 => spells::Element::Radiance,
					Key9 => spells::Element::Life,
					Key0 => spells::Element::Void,
					_ => unreachable!(),
				};
				self.elements.push(element);
			}
		}

		if self.is_button_pressed(Button::Mouse(MouseButton::Left)) {
			if let Some(dir) = (self.mouse - player_pos).normalized() {
				if !self.elements.is_empty() {
					let spell_id = self.total_ids;
					self.total_ids += 1;
					let spell_stats = spells::Spell::new(&self.elements);
					let spell_object = Object {
						pos: player_pos,
						vel: dir * BASE_SPELL_SPEED * spell_stats.speed,
						shape: Shape::Aabb(Vector::new(0.1, 0.1)),
					};
					self.objects.insert(spell_id, spell_object);
					self.spells.insert(spell_id, spell_stats);
					self.elements.clear();
				}
			}
		}

		let mut hits = vec![];
		for (id, stats) in self.spells.iter_mut() {
			let objects: Vec<Object> = self
				.objects
				.iter()
				.filter(|t| *t.0 != self.player_id && t.0 != id)
				.map(|t| t.1.clone())
				.collect();
			let spell = self.objects.get_mut(&id).unwrap();
			let start_pos = spell.pos.clone();
			if spell.move_and_collide(&objects, delta_time)
				|| stats.dist_traveled > stats.range * BASE_SPELL_RANGE
			{
				hits.push(*id);
			} else {
				stats.dist_traveled += (spell.pos - start_pos).length();
				spell.vel.y -= GRAVITY * delta_time;
			}
		}
		for id in hits {
			self.spells.remove(&id);
			self.objects.remove(&id);
			// let spell = self.spells.get(&id).unwrap();
			// match spell.element {
			// 	_ => todo!(),
			// }
		}
	}
}

async fn run() {
	env_logger::init();

	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
	let mut state = render::State::new(&window).await;
	let mut then = std::time::Instant::now();
	let mut leftover_time = 0.0;

	let mut world = World {
		player_id: 0,
		ground_ids: vec![1, 2, 3, 4, 5, 6, 7],
		total_ids: 8,

		objects: HashMap::from([
			(
				0,
				Object::new(
					0.0,
					0.5,
					Shape::Aabb(Vector::new(PLAYER_HEIGHT / 2.0, PLAYER_HEIGHT)),
				),
			),
			(1, Object::new(0.0, 1.5, Shape::Aabb(Vector::new(3.0, 1.0)))),
			(2, Object::new(0.0, -1.5, Shape::Aabb(Vector::new(3.0, 1.0)))),
			(3, Object::new(1.5, 0.0, Shape::Aabb(Vector::new(1.0, 3.0)))),
			(4, Object::new(-1.5, 0.0, Shape::Aabb(Vector::new(1.0, 3.0)))),
			(5, Object::new(0.5, 0.2, Shape::Aabb(Vector::new(0.4, 0.1)))),
			(6, Object::new(0.0, -0.2, Shape::Aabb(Vector::new(0.4, 0.1)))),
			(7, Object::new(-0.5, -0.6, Shape::Aabb(Vector::new(0.4, 0.1)))),
		]),
		colors: HashMap::from([
			(0, [0.0, 0.0, 1.0]),
			(1, [0.0; 3]),
			(2, [0.0; 3]),
			(3, [0.0; 3]),
			(4, [0.0; 3]),
			(5, [0.0; 3]),
			(6, [0.0; 3]),
			(7, [0.0; 3]),
		]),
		spells: HashMap::new(),

		elements: vec![],
		buttons: HashMap::new(),
		mouse: Vector::new(0.0, 0.0),
	};

	event_loop.run(move |event, _, control_flow| {
		use winit::{event::Event, event::WindowEvent, event_loop::ControlFlow};
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
				WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					Some(key) => world.update_button(Button::Key(key), input.state),
					None => {}
				},
				WindowEvent::MouseInput { button, state, .. } => {
					world.update_button(Button::Mouse(button), state)
				}
				WindowEvent::CursorMoved { position, .. } => {
					let pixels = Vector::new(position.x as f32, position.y as f32);
					let aspect = state.config.height as f32 / state.config.width as f32;
					world.mouse = if aspect < 1.0 {
						Vector::new(
							pixels.x / state.config.height as f32 * 2.0 - 1.0 / aspect,
							pixels.y / state.config.height as f32 * 2.0 - 1.0,
						)
					} else {
						Vector::new(
							pixels.x / state.config.width as f32 * 2.0 - 1.0,
							pixels.y / state.config.width as f32 * 2.0 - aspect,
						)
					};
					world.mouse.y *= -1.0;
				}
				WindowEvent::Resized(size) => state.resize(size),
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					state.resize(*new_inner_size)
				}
				WindowEvent::CloseRequested | WindowEvent::Destroyed => {
					*control_flow = ControlFlow::Exit
				}
				_ => {}
			},
			Event::MainEventsCleared => {
				let now = std::time::Instant::now();
				let delta_time = now - then;
				then = now;

				let delta_time = delta_time.as_micros() as f32 / 1000.0 / 1000.0;
				let updates_time = delta_time + leftover_time;
				let updates_to_run = (updates_time * TICKRATE).floor();
				leftover_time = updates_time - updates_to_run / TICKRATE;
				for _ in 0..updates_to_run as usize {
					world.update();
				}

				match state.surface.get_current_texture() {
					Err(e) => {
						eprintln!("{:?}", e);
						*control_flow = ControlFlow::Exit;
					}
					Ok(output) => {
						let view =
							output.texture.create_view(&wgpu::TextureViewDescriptor::default());
						let mut encoder = state.device.create_command_encoder(
							&wgpu::CommandEncoderDescriptor { label: None },
						);
						encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
							label: Some("Clear Screen"),
							color_attachments: &[wgpu::RenderPassColorAttachment {
								view: &view,
								resolve_target: None,
								ops: wgpu::Operations {
									load: wgpu::LoadOp::Clear(wgpu::Color {
										r: 1.0,
										g: 0.0,
										b: 1.0,
										a: 1.0,
									}),
									store: true,
								},
							}],
							depth_stencil_attachment: None,
						});

						for (id, color) in
							world.colors.iter().map(|(id, color)| (id, *color)).chain(
								world.spells.iter().map(|(id, spell)| {
									(
										id,
										match spell.element {
											spells::Element::Earth => [0.4, 0.2, 0.0],
											spells::Element::Void => [0.0, 0.0, 0.0],
											spells::Element::Life => [1.0, 1.0, 1.0],
											spells::Element::Water => [0.0, 0.0, 1.0],
											spells::Element::Air => [0.0, 1.0, 1.0],
											spells::Element::Fire => [1.0, 0.0, 0.0],
											spells::Element::Acid => [0.0, 1.0, 0.0],
											spells::Element::Shock => [1.0, 1.0, 0.0],
											spells::Element::Pressure => [0.4, 0.6, 1.0],
											spells::Element::Radiance => [1.0, 0.0, 1.0],
										},
									)
								}),
							) {
							let object = world.objects.get(id).unwrap();
							let p = object.pos;
							let vertices = match object.shape {
								Shape::Aabb(Vector { x: w, y: h }) => vec![
									Vertex::new(p.x - w / 2.0, p.y - h / 2.0, color),
									Vertex::new(p.x + w / 2.0, p.y - h / 2.0, color),
									Vertex::new(p.x - w / 2.0, p.y + h / 2.0, color),
									Vertex::new(p.x + w / 2.0, p.y + h / 2.0, color),
									Vertex::new(p.x - w / 2.0, p.y + h / 2.0, color),
									Vertex::new(p.x + w / 2.0, p.y - h / 2.0, color),
								],
								Shape::Line(_dir) => todo!(),
							};
							state.render(
								&mut encoder,
								&view,
								&wgpu::include_wgsl!("flat.wgsl"),
								&wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3]
									.to_vec(),
								vertices,
							);
						}
						state.queue.submit(std::iter::once(encoder.finish()));
						output.present();
					}
				}
			}
			_ => (),
		}
	});
}

fn main() {
	pollster::block_on(run());
}
