use egui_wgpu::renderer::ScreenDescriptor;

use crate::{context::Context, gui::GuiOutput};

pub struct GuiRenderPass {
    render_pass: egui_wgpu::Renderer,
}

impl GuiRenderPass {
    pub fn new(context: &Context) -> Self {
        Self {
            render_pass: egui_wgpu::Renderer::new(&context.device, context.config.format, None, 1),
        }
    }

    pub fn render(
        &mut self,
        context: &Context,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        gui_output: GuiOutput,
    ) -> anyhow::Result<()> {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [context.config.width, context.config.height],
            pixels_per_point: context.scale_factor as f32,
        };

        let GuiOutput(output, gui_context) = gui_output;
        let paint_jobs = gui_context.tessellate(output.shapes);

        for (texture_id, image_delta) in output.textures_delta.set {
            self.render_pass.update_texture(
                &context.device,
                &context.queue,
                texture_id,
                &image_delta,
            );
        }

        self.render_pass.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("Egui Render Pass"),
            });

            self.render_pass
                .render(&mut rpass, &paint_jobs, &screen_descriptor);
        }

        for free_id in output.textures_delta.free {
            self.render_pass.free_texture(&free_id);
        }

        Ok(())
    }
}
