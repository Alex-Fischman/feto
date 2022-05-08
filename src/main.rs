mod object;
mod render;
mod vector;

use object::Object;
use vector::Vector;

use std::collections::HashMap;
use winit::event::VirtualKeyCode;

const TICKRATE: f32 = 100.0;
const GRAVITY: f32 = 10.0;
const JUMP: f32 = 3.0;
const MOVE: f32 = 1.0;
const GROUND_CHECK: f32 = 0.0001;
const CEILING_BOUNCE: f32 = -0.01;

struct World {
	player: Object,
	ground: Vec<Object>,
	keys: HashMap<VirtualKeyCode, bool>,
}

impl World {
	fn update(&mut self) {
		let delta_time = 1.0 / TICKRATE;

		let vx = MOVE
			* if *self.keys.get(&VirtualKeyCode::A).unwrap_or(&false) {
				-1.0
			} else if *self.keys.get(&VirtualKeyCode::D).unwrap_or(&false) {
				1.0
			} else {
				0.0
			};

		self.player.pos.y -= GROUND_CHECK;
		let on_ground =
			self.ground.iter().any(|collider| object::collide(&self.player, collider));
		self.player.pos.y += GROUND_CHECK * 2.0;
		let on_ceiling =
			self.ground.iter().any(|collider| object::collide(&self.player, collider));
		self.player.pos.y -= GROUND_CHECK;

		let vy = if *self.keys.get(&VirtualKeyCode::Space).unwrap_or(&false) && on_ground {
			JUMP
		} else if on_ground {
			0.0
		} else if on_ceiling {
			CEILING_BOUNCE
		} else {
			self.player.vel.y - GRAVITY * delta_time
		};

		self.player.vel = Vector::new(vx, vy);
		self.player.move_and_collide(&self.ground, delta_time);
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
		player: Object::new(0.0, 0.5, [0.0, 0.0, 1.0], object::Shape::Aabb(0.1, 0.1)),
		ground: vec![
			Object::new(0.0, 1.5, [0.0; 3], object::Shape::Aabb(3.0, 1.0)),
			Object::new(0.0, -1.5, [0.0; 3], object::Shape::Aabb(3.0, 1.0)),
			Object::new(1.5, 0.0, [0.0; 3], object::Shape::Aabb(1.0, 3.0)),
			Object::new(-1.5, 0.0, [0.0; 3], object::Shape::Aabb(1.0, 3.0)),
			Object::new(0.5, 0.2, [0.0; 3], object::Shape::Aabb(0.4, 0.1)),
			Object::new(0.0, -0.2, [0.0; 3], object::Shape::Aabb(0.4, 0.1)),
			Object::new(-0.5, -0.6, [0.0; 3], object::Shape::Aabb(0.4, 0.1)),
		],
		keys: HashMap::new(),
	};

	event_loop.run(move |event, _, control_flow| {
		use winit::{event::Event, event::WindowEvent, event_loop::ControlFlow};
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
				WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					Some(key) => {
						world.keys.insert(
							key,
							match input.state {
								winit::event::ElementState::Pressed => true,
								winit::event::ElementState::Released => false,
							},
						);
					}
					None => {}
				},
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
						for object in &world.ground {
							state.render(object, &mut encoder, &view);
						}
						state.render(&world.player, &mut encoder, &view);
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
