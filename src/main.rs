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
		Object { pos: Vector { x, y }, vel: Vector { x: 0.0, y: 0.0 }, color, shape }
	}

	fn update(&mut self, delta_time: std::time::Duration) {
		let delta_time = delta_time.as_micros() as f32 / 1000.0 / 1000.0;
		self.pos += self.vel * delta_time;
	}
}

enum Shape {
	Aabb(f32, f32),
}

impl render::Renderable<Color> for Object {
	fn shader(&self) -> wgpu::ShaderModuleDescriptor {
		wgpu::include_wgsl!("shader.wgsl")
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

async fn run() {
	env_logger::init();

	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
	let mut state = render::State::new(&window).await;
	let mut then = std::time::Instant::now();
	let mut keys = std::collections::HashMap::new();

	let mut rect = Object::new(0.0, 0.5, [0.0, 0.0, 1.0], Shape::Aabb(0.1, 0.1));
	let ground = Object::new(0.0, 0.0, [0.0; 3], Shape::Aabb(0.4, 0.4));

	event_loop.run(move |event, _, control_flow| {
		use winit::{event::Event, event::WindowEvent, event_loop::ControlFlow};
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
				WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					Some(key) => {
						keys.insert(key, input.state == winit::event::ElementState::Pressed);
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

				rect.vel.x = if let Some(true) = keys.get(&winit::event::VirtualKeyCode::A) {
					-1.0
				} else if let Some(true) = keys.get(&winit::event::VirtualKeyCode::D) {
					1.0
				} else {
					0.0
				};
				rect.vel.y = if let Some(true) = keys.get(&winit::event::VirtualKeyCode::S) {
					-1.0
				} else if let Some(true) = keys.get(&winit::event::VirtualKeyCode::W) {
					1.0
				} else {
					0.0
				};
				if rect.vel.length() != 0.0 {
					rect.vel *= 1.0 / rect.vel.length();
				}
				rect.update(delta_time);
				rect.color =
					if collide(&rect, &ground) { [0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0] };

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
						state.render(&ground, &mut encoder, &view);
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
