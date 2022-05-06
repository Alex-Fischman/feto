mod render;
mod vector;

use vector::Vector;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
	pos: [f32; 2],
	clip: [f32; 2],
}

trait Renderable {
	fn vertices(&mut self) -> [Vertex; 6];
}

struct Object {
	pos: Vector,
	vel: Vector,
	shape: Shape,
}

impl Object {
	fn new_aabb(x: f32, y: f32, width: f32, height: f32) -> Object {
		Object {
			pos: Vector { x, y },
			vel: Vector { x: 0.0, y: 0.0 },
			shape: Shape::Aabb(width, height),
		}
	}

	fn update(&mut self, delta_time: std::time::Duration) {
		let delta_time = delta_time.as_micros() as f32 / 1000.0 / 1000.0;
		self.pos += self.vel * delta_time;
	}
}

enum Shape {
	Aabb(f32, f32),
}

impl Renderable for Object {
	fn vertices(&mut self) -> [Vertex; 6] {
		match self.shape {
			Shape::Aabb(w, h) => [
				Vertex { pos: [self.pos.x - w, self.pos.y - h], clip: [-1.0, -1.0] },
				Vertex { pos: [self.pos.x + w, self.pos.y - h], clip: [1.0, -1.0] },
				Vertex { pos: [self.pos.x - w, self.pos.y + h], clip: [-1.0, 1.0] },
				Vertex { pos: [self.pos.x + w, self.pos.y + h], clip: [1.0, 1.0] },
				Vertex { pos: [self.pos.x - w, self.pos.y + h], clip: [-1.0, 1.0] },
				Vertex { pos: [self.pos.x + w, self.pos.y - h], clip: [1.0, -1.0] },
			],
		}
	}
}

async fn run() {
	env_logger::init();

	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
	let mut state = render::State::new(&window).await;
	let mut then = std::time::Instant::now();
	let mut rect = Object::new_aabb(0.0, 0.5, 0.1, 0.1);

	event_loop.run(move |event, _, control_flow| {
		use winit::{event::Event, event::WindowEvent, event_loop::ControlFlow};
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
				WindowEvent::KeyboardInput { input, .. } => {
					use winit::event::{ElementState, VirtualKeyCode};
					rect.vel.y = match (input.virtual_keycode, input.state) {
						(Some(VirtualKeyCode::W), ElementState::Pressed) => 1.0,
						(Some(VirtualKeyCode::S), ElementState::Pressed) => -1.0,
						_ => 0.0,
					}
				}
				WindowEvent::Resized(size) => state.resize(size),
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					state.resize(*new_inner_size)
				}
				WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
				_ => {}
			},
			Event::MainEventsCleared => {
				let now = std::time::Instant::now();
				let delta_time = now - then;
				then = now;
				rect.update(delta_time);
				match state.surface.get_current_texture() {
					Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
					Err(e) => eprintln!("{:?}", e),
					Ok(output) => {
						let view =
							output.texture.create_view(&wgpu::TextureViewDescriptor::default());
						let mut encoder = state.device.create_command_encoder(
							&wgpu::CommandEncoderDescriptor { label: None },
						);

						encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
							label: Some("Clear Screen"),
							color_attachments: &[render::clear_pass(&view)],
							depth_stencil_attachment: None,
						});

						let mut render = |renderable: &mut dyn Renderable| {
							let shader = state
								.device
								.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));
							use wgpu::util::DeviceExt;

							let mut vertices = renderable.vertices();
							let aspect = state.config.height as f32 / state.config.width as f32;
							for vertex in &mut vertices {
								if aspect < 1.0 {
									vertex.pos[0] *= aspect;
								} else {
									vertex.pos[1] /= aspect;
								}
							}
							let vertices = state.device.create_buffer_init(
								&wgpu::util::BufferInitDescriptor {
									label: Some("Vertex Buffer"),
									contents: bytes(&vertices),
									usage: wgpu::BufferUsages::VERTEX,
								},
							);
							let pipeline = state.device.create_render_pipeline(
								&wgpu::RenderPipelineDescriptor {
									label: None,
									layout: None,
									vertex: wgpu::VertexState {
										module: &shader,
										entry_point: "vertex",
										buffers: &[wgpu::VertexBufferLayout {
											array_stride: std::mem::size_of::<Vertex>()
												as wgpu::BufferAddress,
											step_mode: wgpu::VertexStepMode::Vertex,
											attributes: &wgpu::vertex_attr_array![
												0 => Float32x2,
												1 => Float32x2
											],
										}],
									},
									fragment: Some(wgpu::FragmentState {
										module: &shader,
										entry_point: "fragment",
										targets: &[wgpu::ColorTargetState {
											format: state.config.format,
											blend: Some(wgpu::BlendState::REPLACE),
											write_mask: wgpu::ColorWrites::ALL,
										}],
									}),
									primitive: wgpu::PrimitiveState {
										topology: wgpu::PrimitiveTopology::TriangleList,
										strip_index_format: None,
										front_face: wgpu::FrontFace::Ccw,
										cull_mode: Some(wgpu::Face::Back),
										polygon_mode: wgpu::PolygonMode::Fill,
										unclipped_depth: false,
										conservative: false,
									},
									depth_stencil: None,
									multisample: wgpu::MultisampleState {
										count: 1,
										mask: !0,
										alpha_to_coverage_enabled: false,
									},
									multiview: None,
								},
							);

							let mut render_pass =
								encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
									label: None,
									color_attachments: &[render::draw_pass(&view)],
									depth_stencil_attachment: None,
								});
							render_pass.set_pipeline(&pipeline);
							render_pass.set_vertex_buffer(0, vertices.slice(..));
							render_pass.draw(0..6, 0..1);
						};

						render(&mut rect);
						rect.pos.x -= 1.0;
						render(&mut rect);
						rect.pos.x += 1.0;
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

fn bytes<T: Sized>(slice: &[T]) -> &[u8] {
	unsafe {
		std::slice::from_raw_parts(
			(slice as *const [T]) as *const u8,
			slice.len() * std::mem::size_of::<T>(),
		)
	}
}
