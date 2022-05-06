mod vector;

use vector::Vector;

struct Object {
	c: Vector,
	w: f32,
	h: f32,
	v: Vector,
}

impl Object {
	fn new(x: f32, y: f32, w: f32, h: f32) -> Object {
		Object { c: Vector { x, y }, w, h, v: Vector { x: 0.0, y: 0.0 } }
	}

	fn update(&mut self, delta_time: std::time::Duration) {
		let delta_time = delta_time.as_micros() as f32 / 1000.0 / 1000.0;
		self.c = self.c + self.v * delta_time;
	}

	fn render(
		&self,
		device: &wgpu::Device,
		encoder: &mut wgpu::CommandEncoder,
		view: &wgpu::TextureView,
		config: &wgpu::SurfaceConfiguration,
	) {
		let shader = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));

		use wgpu::util::DeviceExt;
		let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytes(&[
				Vertex {
					position: [self.c.x - self.w, self.c.y - self.h, 0.0],
					clip: [-1.0, -1.0],
				},
				Vertex {
					position: [self.c.x + self.w, self.c.y - self.h, 0.0],
					clip: [1.0, -1.0],
				},
				Vertex {
					position: [self.c.x - self.w, self.c.y + self.h, 0.0],
					clip: [-1.0, 1.0],
				},
				Vertex {
					position: [self.c.x + self.w, self.c.y + self.h, 0.0],
					clip: [1.0, 1.0],
				},
			]),
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
		});
		let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytes(&[0, 1, 2, 3, 2, 1]),
			usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
		});
		let uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Uniform Buffer"),
			contents: bytes(&[config.height as f32 / config.width as f32]),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});
		let uniform_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: None,
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}],
			});
		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &uniform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: uniforms.as_entire_binding(),
			}],
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[&uniform_bind_group_layout],
				push_constant_ranges: &[],
			})),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vertex",
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fragment",
				targets: &[wgpu::ColorTargetState {
					format: config.format,
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
		});

		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[wgpu::RenderPassColorAttachment {
				view,
				resolve_target: None,
				ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
			}],
			depth_stencil_attachment: None,
		});

		render_pass.set_pipeline(&pipeline);
		render_pass.set_bind_group(0, &uniform_bind_group, &[]);
		render_pass.set_vertex_buffer(0, vertices.slice(..));
		render_pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
		render_pass.draw_indexed(0..6, 0, 0..1);
	}
}

fn collide(a: Object, b: Object) -> bool {
	a.c.x - a.w / 2.0 < b.c.x + b.w / 2.0
		&& a.c.x + a.w / 2.0 > b.c.x - b.w / 2.0
		&& a.c.y - a.h / 2.0 < b.c.y + b.h / 2.0
		&& a.c.y + a.h / 2.0 > b.c.y - b.h / 2.0
}

#[repr(C)]
#[derive(Clone, Debug)]
struct Vertex {
	position: [f32; 3],
	clip: [f32; 2],
}

async fn run() {
	env_logger::init();
	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();

	let size = window.inner_size();
	let instance = wgpu::Instance::new(wgpu::Backends::all());
	let surface = unsafe { instance.create_surface(&window) };
	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::LowPower,
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		})
		.await
		.unwrap();
	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::default(),
				label: None,
			},
			None,
		)
		.await
		.unwrap();
	let mut config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: surface.get_preferred_format(&adapter).unwrap(),
		width: size.width,
		height: size.height,
		present_mode: wgpu::PresentMode::Fifo,
	};
	surface.configure(&device, &config);

	let mut then = std::time::Instant::now();
	let mut rect = Object::new(0.0, 0.5, 0.1, 0.1);

	event_loop.run(move |event, _, control_flow| {
		use winit::{event::Event, event::WindowEvent, event_loop::ControlFlow};
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
				WindowEvent::KeyboardInput { input, .. } => {
					use winit::event::{ElementState, VirtualKeyCode};
					rect.v.y = match (input.virtual_keycode, input.state) {
						(Some(VirtualKeyCode::W), ElementState::Pressed) => 1.0,
						(Some(VirtualKeyCode::S), ElementState::Pressed) => -1.0,
						_ => 0.0,
					}
				}
				WindowEvent::Resized(new_inner_size) => {
					config.width = new_inner_size.width;
					config.height = new_inner_size.height;
					surface.configure(&device, &config);
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					config.width = new_inner_size.width;
					config.height = new_inner_size.height;
					surface.configure(&device, &config);
				}
				WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
				_ => {}
			},
			Event::MainEventsCleared => {
				let now = std::time::Instant::now();
				let delta_time = now - then;
				then = now;
				rect.update(delta_time);
				match surface.get_current_texture() {
					Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
					Err(e) => eprintln!("{:?}", e),
					Ok(output) => {
						let view =
							output.texture.create_view(&wgpu::TextureViewDescriptor::default());
						let mut encoder =
							device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
								label: None,
							});
						{
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
						}
						rect.render(&device, &mut encoder, &view, &config);
						rect.c.x -= 1.0;
						rect.render(&device, &mut encoder, &view, &config);
						rect.c.x += 1.0;
						queue.submit(std::iter::once(encoder.finish()));
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
