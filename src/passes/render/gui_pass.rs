use egui_wgpu::ScreenDescriptor;

use crate::{context::Context, gui::GuiOutput};

pub struct GuiRenderPass {
    renderer: egui_wgpu::Renderer,
}

impl GuiRenderPass {
    pub fn new(context: &Context) -> Self {
        Self {
            renderer: egui_wgpu::Renderer::new(&context.device, context.config.format, None, 1),
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
            pixels_per_point: context.window.scale_factor() as f32,
        };

        let GuiOutput(textures_delta, paint_jobs) = gui_output;

        for (texture_id, image_delta) in textures_delta.set {
            self.renderer
                .update_texture(&context.device, &context.queue, texture_id, &image_delta);
        }

        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        for free_id in textures_delta.free {
            self.renderer.free_texture(&free_id);
        }

        Ok(())
    }
}
