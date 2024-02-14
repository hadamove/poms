use anyhow::Result;

use crate::context::Context;
use crate::gui::{GuiEvent, GuiEvents, GuiOutput};
use crate::utils::constants::ColorTheme;

use molecular_surface::RenderMolecularSurfacePass;
use spacefill::RenderSpacefillPass;

use self::molecular_surface::RenderMolecularSurfaceResources;
use self::spacefill::RenderSpacefillResources;

use super::resources::ResourceRepo;
use gui_pass::GuiRenderPass;

mod gui_pass;
mod molecular_surface;
mod spacefill;

mod util;

pub struct RendererConfig {
    pub render_ui: bool,
    pub render_spacefill: bool,
    pub render_molecular_surface: bool,

    pub clear_color: wgpu::Color,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            render_ui: true,
            render_spacefill: false,
            render_molecular_surface: true,

            clear_color: wgpu::Color::BLACK,
        }
    }
}

pub struct Renderer {
    spacefill_pass: RenderSpacefillPass,
    molecular_surface_pass: RenderMolecularSurfacePass,
    gui_pass: GuiRenderPass,

    config: RendererConfig,
}

impl Renderer {
    pub fn new(context: &Context, resources: &ResourceRepo) -> Renderer {
        let spacefill_pass = RenderSpacefillPass::new(
            &context.device,
            &context.config,
            RenderSpacefillResources {
                camera: resources.camera_resource.clone(),
                molecule: resources.molecule_resource.clone(),
            },
        );

        let molecular_surface_pass = RenderMolecularSurfacePass::new(
            &context.device,
            &context.config,
            RenderMolecularSurfaceResources {
                ses_grid: resources.ses_resource.clone(),
                df_texture: resources.df_texture_front.render.clone(),
                camera: resources.camera_resource.clone(),
                light: resources.light_resource.clone(),
            },
        );

        Self {
            spacefill_pass,
            molecular_surface_pass,
            gui_pass: GuiRenderPass::new(context),
            config: RendererConfig::default(),
        }
    }

    pub fn render(
        &mut self,
        context: &Context,
        resources: &mut ResourceRepo,
        mut encoder: wgpu::CommandEncoder,
        gui_output: GuiOutput,
    ) -> Result<()> {
        // Obtain screen's texture to render to.
        let output_texture = context.surface.get_current_texture()?;

        let view = output_texture.texture.create_view(&Default::default());
        {
            let depth_view = &resources.get_depth_texture().get_view();

            if self.config.render_spacefill {
                self.spacefill_pass.render(
                    &view,
                    depth_view,
                    &mut encoder,
                    self.config.clear_color,
                );
            }
        }
        if self.config.render_molecular_surface {
            // TODO: THIS IS A TEMP FIX FOR SWITCHING DF TEXTURES, remove
            if resources.just_switched {
                self.molecular_surface_pass = RenderMolecularSurfacePass::new(
                    &context.device,
                    &context.config,
                    RenderMolecularSurfaceResources {
                        ses_grid: resources.ses_resource.clone(),
                        df_texture: resources.df_texture_front.render.clone(),
                        camera: resources.camera_resource.clone(),
                        light: resources.light_resource.clone(),
                    },
                );
                resources.just_switched = false;
            }
            let depth_view = &resources.get_depth_texture().get_view();
            self.molecular_surface_pass.render(
                &view,
                depth_view,
                &mut encoder,
                self.config.clear_color,
            );
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
                    self.config.render_molecular_surface = *enabled;
                }
                GuiEvent::RenderSpacefillChanged(enabled) => {
                    self.config.render_spacefill = *enabled;
                }
                GuiEvent::ToggleTheme(theme) => {
                    self.config.clear_color = match theme {
                        ColorTheme::Dark => wgpu::Color::BLACK,
                        ColorTheme::Light => wgpu::Color::WHITE,
                    };
                }
                // TODO: Add event for toggling UI
                _ => {}
            }
        }
    }
}
