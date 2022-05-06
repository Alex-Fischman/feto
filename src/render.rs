pub struct State {
	pub instance: wgpu::Instance,
	pub surface: wgpu::Surface,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration,
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
}

pub fn clear_pass(view: &wgpu::TextureView) -> wgpu::RenderPassColorAttachment {
	wgpu::RenderPassColorAttachment {
		view,
		resolve_target: None,
		ops: wgpu::Operations {
			load: wgpu::LoadOp::Clear(wgpu::Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 }),
			store: true,
		},
	}
}

pub fn draw_pass(view: &wgpu::TextureView) -> wgpu::RenderPassColorAttachment {
	wgpu::RenderPassColorAttachment {
		view,
		resolve_target: None,
		ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
	}
}
