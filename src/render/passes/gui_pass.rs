use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};

use crate::{gpu::GpuState, gui::GuiOutput};

pub struct GuiRenderPass {
    render_pass: egui_wgpu_backend::RenderPass,
}

impl GuiRenderPass {
    pub fn new(gpu: &GpuState) -> Self {
        Self {
            render_pass: RenderPass::new(&gpu.device, gpu.config.format, 1),
        }
    }

    pub fn render(
        &mut self,
        gpu: &GpuState,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        gui_output: GuiOutput,
    ) -> Result<(), BackendError> {
        let screen_descriptor = ScreenDescriptor {
            physical_width: gpu.config.width,
            physical_height: gpu.config.height,
            scale_factor: gpu.scale_factor as f32,
        };

        let GuiOutput(output, context) = gui_output;
        let paint_jobs = context.tessellate(output.shapes);

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
