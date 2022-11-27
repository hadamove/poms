use anyhow::Result;

use crate::context::Context;
use crate::gui::GuiOutput;

use self::gui_pass::GuiRenderPass;
use self::render_pass::RenderPass;

use super::compute::PassId;
use super::resources::GlobalResources;

mod gui_pass;
mod render_pass;

pub struct Renderer {
    pub gui_pass: GuiRenderPass,
    render_passes: Vec<RenderPass>,
}

impl Renderer {
    pub fn new(context: &Context, resources: &GlobalResources) -> Renderer {
        Self {
            gui_pass: GuiRenderPass::new(context),

            render_passes: vec![
                RenderPass::new(context, resources, PassId::Spacefill),
                RenderPass::new(context, resources, PassId::SesRaymarching),
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
        context: &Context,
        resources: &GlobalResources,
        mut encoder: wgpu::CommandEncoder,
        gui_output: GuiOutput,
    ) -> Result<()> {
        // Obtain screen's texture to render to.
        let output_texture = context.surface.get_current_texture()?;

        let view = output_texture.texture.create_view(&Default::default());
        let depth_view = &resources.get_depth_texture().view;

        // Render Spacefill and Ses.
        for render_pass in &mut self.render_passes {
            render_pass.render(&view, depth_view, &mut encoder, resources);
        }

        // Render GUI.
        self.gui_pass
            .render(context, &view, &mut encoder, gui_output)?;

        // Submit commands to the GPU.
        context.queue.submit(Some(encoder.finish()));

        // Draw a frame.
        output_texture.present();

        Ok(())
    }
}
