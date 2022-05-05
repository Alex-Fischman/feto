#[repr(C)]
#[derive(Clone, Debug)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 3],
}

const VERTICES: [Vertex; 5] = [
	Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] },
	Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] },
	Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] },
	Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] },
	Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] },
];

const INDICES: [u16; 9] = [0, 1, 4, 1, 2, 4, 2, 3, 4];

struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: winit::dpi::PhysicalSize<u32>,
	pipeline: wgpu::RenderPipeline,
	vertices: wgpu::Buffer,
	indices: wgpu::Buffer,
}

impl State {
	async fn new(window: &winit::window::Window) -> State {
		let size = window.inner_size();
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(window) };
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
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_preferred_format(&adapter).unwrap(),
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &config);
		let shader = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[],
			push_constant_ranges: &[],
		});
		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vertex",
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
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
		use wgpu::util::DeviceExt;
		let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytes(&VERTICES),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytes(&INDICES),
			usage: wgpu::BufferUsages::INDEX,
		});
		State { surface, device, queue, config, size, pipeline, vertices, indices }
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.size = new_size;
		self.config.width = new_size.width;
		self.config.height = new_size.height;
		self.surface.configure(&self.device, &self.config);
	}

	fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
		false
	}

	fn update(&mut self) {}

	fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder =
			self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[wgpu::RenderPassColorAttachment {
				view: &view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
					store: true,
				},
			}],
			depth_stencil_attachment: None,
		});
		render_pass.set_pipeline(&self.pipeline);
		render_pass.set_vertex_buffer(0, self.vertices.slice(..));
		render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
		drop(render_pass);
		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();
		Ok(())
	}
}

async fn run() {
	env_logger::init();
	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
	let mut state = State::new(&window).await;
	event_loop.run(move |event, _, control_flow| {
		use winit::{event::Event, event::WindowEvent, event_loop::ControlFlow};
		*control_flow = ControlFlow::Wait;
		match event {
			Event::WindowEvent { event, window_id } if window_id == window.id() => {
				if !state.input(&event) {
					match event {
						WindowEvent::Resized(size) => state.resize(size),
						WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
							state.resize(*new_inner_size)
						}
						WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
						_ => {}
					}
				}
			}
			Event::RedrawRequested(window_id) if window_id == window.id() => {
				state.update();
				match state.render() {
					Ok(_) => {}
					Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
					Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
					Err(e) => eprintln!("{:?}", e),
				}
			}
			Event::MainEventsCleared => window.request_redraw(),
			_ => (),
		}
	});
}

fn main() {
	pollster::block_on(run());
}

fn bytes<T: Sized>(t: &T) -> &[u8] {
	unsafe { std::slice::from_raw_parts((t as *const T) as *const u8, std::mem::size_of::<T>()) }
}
