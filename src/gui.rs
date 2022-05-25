use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use imgui::FontSource;
use winit::{event::Event, window::Window};

pub struct Gui {
    context: RefCell<imgui::Context>,
    platform: RefCell<imgui_winit_support::WinitPlatform>,
    renderer: imgui_wgpu::Renderer,

    last_frame: Instant,
    delta_t: Duration,
}

impl Gui {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let mut context = Gui::init_context(window);

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(
            context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: config.format,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(&mut context, device, queue, renderer_config);
        let last_frame = Instant::now();

        Self {
            context: RefCell::new(context),
            platform: RefCell::new(platform),
            renderer,
            last_frame,
            delta_t: Duration::new(0, 0),
        }
    }

    fn init_context(window: &Window) -> imgui::Context {
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);

        let hidpi_factor = window.scale_factor();
        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        let font_size = (13.0 * hidpi_factor) as f32;

        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);
        context
    }

    pub fn handle_event(&self, window: &Window, event: &Event<()>) {
        self.platform
            .borrow_mut()
            .handle_event(self.context.borrow_mut().io_mut(), window, event)
    }

    pub fn update(&mut self, window: &Window) {
        let now = Instant::now();
        self.delta_t = now - self.last_frame;

        let mut context_mut = self.context.borrow_mut();
        let io = context_mut.io_mut();

        io.update_delta_time(self.delta_t);
        self.last_frame = now;

        self.platform
            .borrow_mut()
            .prepare_frame(io, window)
            .unwrap();
    }

    pub fn build_ui(&self, ui: &imgui::Ui, window: &Window) {
        let frametime_window = imgui::Window::new("Profile");
        frametime_window
            .size([200.0, 50.0], imgui::Condition::FirstUseEver)
            .position([400.0, 200.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                ui.text(format!("Frametime: {:?}", self.delta_t));
            });

        // TODO: add more widgets here

        self.platform.borrow_mut().prepare_render(ui, window);
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.update(window);

        let mut context_mut = self.context.borrow_mut();
        let ui = context_mut.frame();

        self.build_ui(&ui, window);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.renderer
            .render(ui.render(), queue, device, &mut render_pass)
            .expect("Rendering failed");
    }
}
