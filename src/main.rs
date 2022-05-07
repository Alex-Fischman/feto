mod render;
mod vector;

use vector::Vector;

type Color = [f32; 3];

struct Object {
	pos: Vector,
	vel: Vector,
	color: Color,
	shape: Shape,
}

impl Object {
	fn new(x: f32, y: f32, color: Color, shape: Shape) -> Object {
		Object { pos: Vector { x, y }, vel: Vector::new(0.0, 0.0), color, shape }
	}

	fn move_and_collide(&mut self, colliders: &[Object], delta_time: f32) -> bool {
		let mut collided = false;
		self.pos += self.vel * delta_time;
		'outer: loop {
			for collider in colliders {
				if collide(self, collider) {
					collided = true;
					match (&self.shape, &collider.shape) {
						(Shape::Aabb(sw, sh), Shape::Aabb(cw, ch)) => {
							let red =
								((self.pos.y - sh / 2.0) - (collider.pos.y + ch / 2.0)).abs();
							let black =
								((self.pos.x - sw / 2.0) - (collider.pos.x + cw / 2.0)).abs();
							let yellow =
								((self.pos.y + sh / 2.0) - (collider.pos.y - ch / 2.0)).abs();
							let purple =
								((self.pos.x + sw / 2.0) - (collider.pos.x - cw / 2.0)).abs();
							self.pos += if red < black && red < yellow && red < purple {
								Vector::new(0.0, red)
							} else if black < red && black < yellow && black < purple {
								Vector::new(black, 0.0)
							} else if yellow < red && yellow < black && yellow < purple {
								Vector::new(0.0, -yellow)
							} else {
								Vector::new(-purple, 0.0)
							};
						}
					}
					continue 'outer;
				}
			}
			break;
		}
		collided
	}
}

enum Shape {
	Aabb(f32, f32),
}

impl render::Renderable<Color> for Object {
	fn shader(&self) -> wgpu::ShaderModuleDescriptor {
		wgpu::include_wgsl!("flat.wgsl")
	}

	fn vertex_layout(&self) -> Vec<wgpu::VertexAttribute> {
		wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3].to_vec()
	}

	fn vertices(&self) -> Vec<render::Vertex<Color>> {
		match self.shape {
			Shape::Aabb(w, h) => vec![
				render::Vertex::new(self.pos.x - w / 2.0, self.pos.y - h / 2.0, self.color),
				render::Vertex::new(self.pos.x + w / 2.0, self.pos.y - h / 2.0, self.color),
				render::Vertex::new(self.pos.x - w / 2.0, self.pos.y + h / 2.0, self.color),
				render::Vertex::new(self.pos.x + w / 2.0, self.pos.y + h / 2.0, self.color),
				render::Vertex::new(self.pos.x - w / 2.0, self.pos.y + h / 2.0, self.color),
				render::Vertex::new(self.pos.x + w / 2.0, self.pos.y - h / 2.0, self.color),
			],
		}
	}
}

fn collide(a: &Object, b: &Object) -> bool {
	match (&a.shape, &b.shape) {
		(Shape::Aabb(aw, ah), Shape::Aabb(bw, bh)) => {
			a.pos.x - aw / 2.0 < b.pos.x + bw / 2.0
				&& a.pos.x + aw / 2.0 > b.pos.x - bw / 2.0
				&& a.pos.y - ah / 2.0 < b.pos.y + bh / 2.0
				&& a.pos.y + ah / 2.0 > b.pos.y - bh / 2.0
		}
	}
}

use std::collections::HashMap;
use winit::event::VirtualKeyCode;
struct Keys(HashMap<VirtualKeyCode, bool>);

impl Keys {
	fn new() -> Keys {
		Keys(HashMap::new())
	}

	fn is_key_held(&self, key: VirtualKeyCode) -> bool {
		match self.0.get(&key) {
			None => false,
			Some(false) => false,
			Some(true) => true,
		}
	}
}

const TICKRATE: f32 = 100.0;
const GRAVITY: f32 = 10.0;
const JUMP: f32 = 2.0;
const GROUND_CHECK: f32 = 0.0001;
const CEILING_BOUNCE: f32 = -0.01;
fn update(rect: &mut Object, ground: &[Object], keys: &mut Keys) {
	let vx = if keys.is_key_held(VirtualKeyCode::A) {
		-1.0
	} else if keys.is_key_held(VirtualKeyCode::D) {
		1.0
	} else {
		0.0
	};

	rect.pos.y -= GROUND_CHECK;
	let on_ground = ground.iter().any(|collider| collide(rect, collider));
	rect.pos.y += GROUND_CHECK * 2.0;
	let on_ceiling = ground.iter().any(|collider| collide(rect, collider));
	rect.pos.y -= GROUND_CHECK;

	let vy = if keys.is_key_held(VirtualKeyCode::Space) && on_ground {
		JUMP
	} else if on_ground {
		0.0
	} else if on_ceiling {
		CEILING_BOUNCE
	} else {
		rect.vel.y - GRAVITY / TICKRATE
	};

	rect.vel = Vector::new(vx, vy);
	rect.move_and_collide(ground, 1.0 / TICKRATE);
}

async fn run() {
	env_logger::init();

	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
	let mut state = render::State::new(&window).await;
	let mut then = std::time::Instant::now();
	let mut leftover_time = 0.0;
	let mut keys = Keys::new();

	let mut rect = Object::new(0.0, 0.5, [0.0, 0.0, 1.0], Shape::Aabb(0.1, 0.1));
	let ground = [
		Object::new(0.0, 1.5, [0.0; 3], Shape::Aabb(3.0, 1.0)),
		Object::new(0.0, -1.5, [0.0; 3], Shape::Aabb(3.0, 1.0)),
		Object::new(1.5, 0.0, [0.0; 3], Shape::Aabb(1.0, 3.0)),
		Object::new(-1.5, 0.0, [0.0; 3], Shape::Aabb(1.0, 3.0)),
		Object::new(0.0, -0.25, [0.0; 3], Shape::Aabb(1.0, 1.0)),
	];

	event_loop.run(move |event, _, control_flow| {
		use winit::{event::Event, event::WindowEvent, event_loop::ControlFlow};
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
				WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					Some(key) => {
						keys.0.insert(
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
					update(&mut rect, &ground, &mut keys);
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
						for object in &ground {
							state.render(object, &mut encoder, &view);
						}
						state.render(&rect, &mut encoder, &view);
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
