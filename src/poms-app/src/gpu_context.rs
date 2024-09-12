use std::sync::Arc;

use winit::window::Window;

/// Represents all the necessary low-level `wgpu` stuff used by the application.
pub(crate) struct GpuContext {
    /// The window used by the application.
    /// `std::sync::Arc` is used to allow sharing the reference to the window with user interface, see `EguiWrapper` for more details.
    pub(crate) window: Arc<Window>,
    /// A platform-specific surface onto which the application renders.
    pub(crate) surface: wgpu::Surface<'static>,
    /// Connection to the underlying GPU, used to create resources.
    pub(crate) device: wgpu::Device,
    /// A queue used to submit commands to the GPU.
    pub(crate) queue: wgpu::Queue,
    /// Configuration of the surface.
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl GpuContext {
    pub(crate) async fn initialize(window: Arc<Window>) -> Self {
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

        // An adapter is a handle to a physical device on the system.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable adapter");

        // From the adapter, we can request a logical device and a queue.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to create device");

        // Check which texture formats are supported by the surface (usually it is okay to just use the first one).
        let supported_format = *surface
            .get_capabilities(&adapter)
            .formats
            .first()
            .expect("No supported format");

        let size = window.inner_size();

        // Configure the surface with the supported format.
        // Either use vsync or not, depending on the feature flags. See README.md for more details.
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

    /// Resizes the surface to the given size. This method should be called when the window is resized in the main event loop.
    pub(crate) fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Creates a new command encoder from `GpuContext::device` and returns it. This is just a shorthand method to avoid boilerplate code.
    pub(crate) fn get_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder"),
            })
    }
}
