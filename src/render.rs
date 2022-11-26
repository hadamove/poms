use anyhow::Result;

use passes::gui_pass::GuiRenderPass;

use crate::compute::PassId;
use crate::gui::GuiOutput;
use crate::render_pass::RenderPass;
use crate::shared::resources::GlobalResources;

use super::gpu::GpuState;

mod passes;

pub struct Renderer {
    pub gui_pass: GuiRenderPass,
    render_passes: Vec<RenderPass>,
}

impl Renderer {
    pub fn new(gpu: &GpuState, global_resources: &GlobalResources) -> Renderer {
        Self {
            gui_pass: GuiRenderPass::new(gpu),

            render_passes: vec![
                RenderPass::new(gpu, global_resources, PassId::Spacefill),
                RenderPass::new(gpu, global_resources, PassId::SesRaymarching),
            ],
        }
    }

    pub fn toggle_render_pass(&mut self, pass_id: PassId, enabled: bool) {
        for pass in self.render_passes.iter_mut() {
            if pass.get_id() == &pass_id {
                pass.set_enabled(enabled);
            }
        }
    }

    pub fn render(
        &mut self,
        gpu: &GpuState,
        global_resources: &GlobalResources,
        mut encoder: wgpu::CommandEncoder,
        gui_output: GuiOutput,
    ) -> Result<()> {
        // Obtain screen's texture to render to.
        let output_texture = gpu.surface.get_current_texture()?;

        let view = output_texture.texture.create_view(&Default::default());
        let depth_view = &global_resources.get_depth_texture().view;

        // Render Spacefill and Ses.
        for render_pass in &mut self.render_passes {
            render_pass.render(&view, depth_view, &mut encoder, global_resources);
        }

        // Render GUI
        self.gui_pass.render(gpu, &view, &mut encoder, gui_output)?;

        // Submit commands to the GPU
        gpu.queue.submit(Some(encoder.finish()));

        // Draw a frame
        output_texture.present();

        Ok(())
    }
}
