use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};

use crate::{context::Context, gui::GuiOutput};

pub struct GuiRenderPass {
    render_pass: egui_wgpu_backend::RenderPass,
}

impl GuiRenderPass {
    pub fn new(context: &Context) -> Self {
        Self {
            render_pass: RenderPass::new(&context.device, context.config.format, 1),
        }
    }

    pub fn render(
        &mut self,
        context: &Context,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        gui_output: GuiOutput,
    ) -> Result<(), BackendError> {
        let screen_descriptor = ScreenDescriptor {
            physical_width: context.config.width,
            physical_height: context.config.height,
            scale_factor: context.scale_factor as f32,
        };

        let GuiOutput(output, gui_context) = gui_output;
        let paint_jobs = gui_context.tessellate(output.shapes);

        self.render_pass
            .add_textures(&context.device, &context.queue, &output.textures_delta)?;
        self.render_pass.remove_textures(output.textures_delta)?;

        self.render_pass.update_buffers(
            &context.device,
            &context.queue,
            &paint_jobs,
            &screen_descriptor,
        );

        self.render_pass
            .execute(encoder, view, &paint_jobs, &screen_descriptor, None)?;

        Ok(())
    }
}
