use egui::FontDefinitions;
use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::window::Window;

use crate::gui::Gui;

pub struct GuiRenderPass {
    render_pass: egui_wgpu_backend::RenderPass,
    platform: Platform,
}

impl GuiRenderPass {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let render_pass = RenderPass::new(device, config.format, 1);

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
        }
    }

    pub fn handle_events<T>(&mut self, event: &winit::event::Event<T>) -> bool {
        self.platform.handle_event(event);
        self.platform.captures_event(event)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        gui: &mut Gui,
    ) -> Result<(), BackendError> {
        self.platform.begin_frame();

        gui.ui(&self.platform.context(), config);

        let output = self.platform.end_frame(Some(window));
        let paint_jobs = self.platform.context().tessellate(output.shapes);

        let screen_descriptor = ScreenDescriptor {
            physical_width: config.width,
            physical_height: config.height,
            scale_factor: window.scale_factor() as f32,
        };

        self.render_pass
            .add_textures(device, queue, &output.textures_delta)?;

        self.render_pass.remove_textures(output.textures_delta)?;

        self.render_pass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        self.render_pass
            .execute(encoder, view, &paint_jobs, &screen_descriptor, None)?;

        Ok(())
    }
}
