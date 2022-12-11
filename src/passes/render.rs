use anyhow::Result;

use crate::context::Context;
use crate::gui::{GuiEvent, GuiEvents, GuiOutput};
use crate::utils::constants::ColorTheme;

use super::resources::{PassId, ResourceRepo};
use gui_pass::GuiRenderPass;
use render_pass::RenderPass;

mod gui_pass;
mod render_pass;

pub struct Renderer {
    gui_pass: GuiRenderPass,
    render_passes: Vec<RenderPass>,
    clear_color: wgpu::Color,
}

impl Renderer {
    pub fn new(context: &Context, resources: &ResourceRepo) -> Renderer {
        Self {
            gui_pass: GuiRenderPass::new(context),

            render_passes: vec![
                RenderPass::new(context, resources, PassId::RenderSpacefill),
                RenderPass::new(context, resources, PassId::RenderSesRaymarching),
            ],
            clear_color: wgpu::Color::BLACK,
        }
    }

    pub fn render(
        &mut self,
        context: &Context,
        resources: &ResourceRepo,
        mut encoder: wgpu::CommandEncoder,
        gui_output: GuiOutput,
    ) -> Result<()> {
        // Obtain screen's texture to render to.
        let output_texture = context.surface.get_current_texture()?;

        let view = output_texture.texture.create_view(&Default::default());
        let depth_view = &resources.get_depth_texture().get_view();

        // Render Spacefill and Ses.
        for render_pass in &mut self.render_passes {
            render_pass.render(&view, depth_view, &mut encoder, resources, self.clear_color);
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

    pub fn handle_events(&mut self, events: &GuiEvents) {
        for event in events {
            match event {
                GuiEvent::RenderSesChanged(enabled) => {
                    self.toggle_render_pass(PassId::RenderSesRaymarching, *enabled);
                }
                GuiEvent::RenderSpacefillChanged(enabled) => {
                    self.toggle_render_pass(PassId::RenderSpacefill, *enabled);
                }
                GuiEvent::ToggleTheme(theme) => {
                    self.toggle_theme(theme);
                }
                _ => {}
            }
        }
    }

    fn toggle_render_pass(&mut self, pass_id: PassId, enabled: bool) {
        for pass in self.render_passes.iter_mut() {
            if pass.get_id() == &pass_id {
                pass.set_enabled(enabled);
            }
        }
    }

    fn toggle_theme(&mut self, theme: &ColorTheme) {
        match theme {
            ColorTheme::Dark => self.clear_color = wgpu::Color::BLACK,
            ColorTheme::Light => self.clear_color = wgpu::Color::WHITE,
        }
    }
}
