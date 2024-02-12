use std::sync::Arc;

use winit::window::Window;

pub struct Context {
    pub window: Arc<Window>,

    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl Context {
    pub async fn initialize(window: Arc<Window>) -> Self {
        #[cfg(feature = "vulkan")]
        let backends = wgpu::Backends::all();

        #[cfg(not(feature = "vulkan"))]
        let backends = wgpu::Backends::all() & !wgpu::Backends::VULKAN;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to create device");

        let supported_format = *surface
            .get_capabilities(&adapter)
            .formats
            .first()
            .expect("No supported format");

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: supported_format,
            width: size.width,
            height: size.height,
            #[cfg(not(feature = "no-vsync"))]
            present_mode: wgpu::PresentMode::Fifo,
            #[cfg(feature = "no-vsync")]
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![supported_format],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        Self {
            window,

            surface,
            device,
            queue,
            config,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn get_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            })
    }
}
