use crate::vector::Vector;

pub struct State {
	pub instance: wgpu::Instance,
	pub surface: wgpu::Surface,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration,
}

#[repr(C)]
pub struct Vertex<VertexData> {
	pub pos: Vector,
	pub data: VertexData,
}

pub trait Renderable<VertexData> {
	fn shader(&self) -> wgpu::ShaderModuleDescriptor;
	fn vertex_layout(&self) -> Vec<wgpu::VertexAttribute>;
	fn vertices(&self) -> Vec<Vertex<VertexData>>;
}

impl State {
	pub async fn new(window: &winit::window::Window) -> State {
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
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_preferred_format(&adapter).unwrap(),
			width: window.inner_size().width,
			height: window.inner_size().height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &config);
		State { instance, surface, adapter, device, queue, config }
	}

	pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
		self.config.width = size.width;
		self.config.height = size.height;
		self.surface.configure(&self.device, &self.config);
	}

	pub fn render<VertexData>(
		&self,
		renderable: &dyn Renderable<VertexData>,
		encoder: &mut wgpu::CommandEncoder,
		view: &wgpu::TextureView,
	) {
		let shader = self.device.create_shader_module(&renderable.shader());

		let mut vertices = renderable.vertices();
		let aspect = self.config.height as f32 / self.config.width as f32;
		for vertex in &mut vertices {
			if aspect < 1.0 {
				vertex.pos.x *= aspect;
			} else {
				vertex.pos.y /= aspect;
			}
		}
		use wgpu::util::DeviceExt;
		let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytes(&vertices),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: None,
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vertex",
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: std::mem::size_of::<Vertex<VertexData>>()
						as wgpu::BufferAddress,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &renderable.vertex_layout(),
				}],
			},
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
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fragment",
				targets: &[wgpu::ColorTargetState {
					format: self.config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				}],
			}),
			multiview: None,
		});
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[wgpu::RenderPassColorAttachment {
				view: &view,
				resolve_target: None,
				ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
			}],
			depth_stencil_attachment: None,
		});
		render_pass.set_pipeline(&pipeline);
		render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
		render_pass.draw(0..vertices.len() as u32, 0..1);
	}
}

impl<VertexData> Vertex<VertexData> {
	pub fn new(x: f32, y: f32, data: VertexData) -> Vertex<VertexData> {
		Vertex { pos: Vector { x, y }, data }
	}
}

fn bytes<T: Sized>(slice: &[T]) -> &[u8] {
	unsafe {
		std::slice::from_raw_parts(
			(slice as *const [T]) as *const u8,
			slice.len() * std::mem::size_of::<T>(),
		)
	}
}
