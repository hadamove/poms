use winit::window::Window;

pub struct Context {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    pub scale_factor: f64,
}

impl Context {
    pub async fn new(window: &Window) -> Self {
        #[cfg(feature = "vulkan")]
        let backends = wgpu::Backends::all();

        #[cfg(not(feature = "vulkan"))]
        let backends = wgpu::Backends::all() & !wgpu::Backends::VULKAN;

        let instance = wgpu::Instance::new(backends);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Could not find a suitable adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Could not create device");

        let supported_format = surface
            .get_supported_formats(&adapter)
            .get(0)
            .expect("No format supported")
            .to_owned();

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
        };
        surface.configure(&device, &config);

        let scale_factor = window.scale_factor();

        Self {
            surface,
            device,
            queue,
            config,

            scale_factor,
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
