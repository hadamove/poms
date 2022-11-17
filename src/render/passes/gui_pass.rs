use egui::FontDefinitions;
use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::event::Event;

use crate::{gpu::GpuState, gui::Gui};

pub struct GuiRenderPass {
    render_pass: egui_wgpu_backend::RenderPass,
    platform: Platform,
}

impl GuiRenderPass {
    pub fn new(gpu: &GpuState) -> Self {
        let render_pass = RenderPass::new(&gpu.device, gpu.config.format, 1);

        let platform = Platform::new(PlatformDescriptor {
            physical_width: gpu.config.width,
            physical_height: gpu.config.height,
            scale_factor: gpu.scale_factor,
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        Self {
            render_pass,
            platform,
        }
    }

    pub fn handle_events<T>(&mut self, winit_event: &Event<T>) -> bool {
        self.platform.handle_event(winit_event);
        self.platform.captures_event(winit_event)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        gpu: &GpuState,
        gui: &mut Gui,
    ) -> Result<(), BackendError> {
        self.platform.begin_frame();

        gui.ui(&self.platform.context());

        let output = self.platform.end_frame(None);
        let paint_jobs = self.platform.context().tessellate(output.shapes);

        let screen_descriptor = ScreenDescriptor {
            physical_width: gpu.config.width,
            physical_height: gpu.config.height,
            scale_factor: gpu.scale_factor as f32,
        };

        self.render_pass
            .add_textures(&gpu.device, &gpu.queue, &output.textures_delta)?;

        self.render_pass.remove_textures(output.textures_delta)?;

        self.render_pass
            .update_buffers(&gpu.device, &gpu.queue, &paint_jobs, &screen_descriptor);

        self.render_pass
            .execute(encoder, view, &paint_jobs, &screen_descriptor, None)?;

        Ok(())
    }
}
