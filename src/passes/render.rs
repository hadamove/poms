use crate::context::Context;
use crate::ui::event::UserEvent;
use crate::utils::constants::ColorTheme;

use molecular_surface::RenderMolecularSurfacePass;
use spacefill::RenderSpacefillPass;

use self::molecular_surface::RenderMolecularSurfaceResources;
use self::spacefill::RenderSpacefillResources;

use super::resources::ResourceRepo;

mod molecular_surface;
mod spacefill;

mod util;

pub struct RendererConfig {
    pub render_spacefill: bool,
    pub render_molecular_surface: bool,

    pub clear_color: wgpu::Color,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            render_spacefill: false,
            render_molecular_surface: true,

            clear_color: wgpu::Color::BLACK,
        }
    }
}

pub struct RenderJobs {
    spacefill_pass: RenderSpacefillPass,
    molecular_surface_pass: RenderMolecularSurfacePass,

    config: RendererConfig,
}

impl RenderJobs {
    pub fn new(context: &Context, resources: &ResourceRepo) -> RenderJobs {
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
            config: RendererConfig::default(),
        }
    }

    pub fn execute(
        &mut self,
        _context: &Context,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if self.config.render_spacefill {
            self.spacefill_pass
                .render(view, depth_view, encoder, self.config.clear_color);
        }

        if self.config.render_molecular_surface {
            self.molecular_surface_pass
                .render(view, depth_view, encoder, self.config.clear_color);
        }
    }

    // TODO: App should handle this
    pub fn handle_events(&mut self, events: &Vec<UserEvent>) {
        for event in events {
            match event {
                UserEvent::RenderSesChanged(enabled) => {
                    self.config.render_molecular_surface = *enabled;
                }
                UserEvent::RenderSpacefillChanged(enabled) => {
                    self.config.render_spacefill = *enabled;
                }
                UserEvent::ToggleTheme(theme) => {
                    self.config.clear_color = match theme {
                        ColorTheme::Dark => wgpu::Color::BLACK,
                        ColorTheme::Light => wgpu::Color::WHITE,
                    };
                }
                _ => {}
            }
        }
    }
}
