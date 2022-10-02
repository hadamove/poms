use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::window::Window;

use super::my_app::MyApp;

pub struct Gui {
    render_pass: egui_wgpu_backend::RenderPass,
    platform: Platform,

    pub my_app: MyApp,
}

impl Gui {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let render_pass = RenderPass::new(device, config.format, 1);

        let my_app = MyApp::default();

        let platform = Platform::new(PlatformDescriptor {
            physical_width: config.width,
            physical_height: config.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        Self {
            render_pass,
            platform,
            my_app,
        }
    }

    pub fn handle_events<T>(&mut self, event: &winit::event::Event<T>) -> bool {
        self.platform.handle_event(event);
        self.platform.captures_event(event)
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        self.platform.begin_frame();

        // Draw the demo application.
        self.my_app.ui(&self.platform.context());

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let output = self.platform.end_frame(Some(window));

        let paint_jobs = self.platform.context().tessellate(output.shapes);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: config.width,
            physical_height: config.height,
            scale_factor: window.scale_factor() as f32,
        };
        self.render_pass
            .add_textures(device, queue, &output.textures_delta)
            .unwrap();

        self.render_pass
            .remove_textures(output.textures_delta)
            .unwrap();

        self.render_pass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        self.render_pass
            .execute(encoder, view, &paint_jobs, &screen_descriptor, None)
            .unwrap();
    }
}
