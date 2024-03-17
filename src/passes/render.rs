use molecular_surface::RenderMolecularSurfacePass;
use spacefill::RenderSpacefillPass;

use self::molecular_surface::RenderMolecularSurfaceResources;
use self::spacefill::RenderSpacefillResources;

// TODO: Clean up imports
use super::resources::camera::resource::CameraResource;
use super::resources::grid::molecule_grid::MoleculeGridResource;
use super::resources::grid::ses_grid::SesGridResource;
use super::resources::light::LightResource;
use super::resources::textures::depth_texture::DepthTexture;

use super::resources::textures::df_texture::DistanceFieldTexture;

mod molecular_surface;
mod spacefill;

mod util;

pub struct RenderDependencies<'a> {
    pub molecule_resource: &'a MoleculeGridResource,
    pub ses_resource: &'a SesGridResource,
}

pub struct RenderOwnedResources {
    pub depth_texture: DepthTexture,
    pub light_resource: LightResource,
    pub camera_resource: CameraResource,
    // TODO: Replace with DistanceFieldRender
    pub df_texture_front: DistanceFieldTexture,
}

/// Configuration for the renderer.
pub struct RenderConfig {
    /// Whether to render the spacefill representation.
    pub render_spacefill: bool,
    /// Whether to render the molecular surface representation.
    pub render_molecular_surface: bool,
    /// The clear color of the renderer.
    pub clear_color: wgpu::Color,
    // TODO: Add ArcballCamera?
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            render_spacefill: false,
            render_molecular_surface: true,
            clear_color: wgpu::Color::BLACK,
        }
    }
}

/// A collection of render passes that are executed in order to render the molecule.
pub struct RenderJobs {
    /// Configuration for the renderer. This is used to control what is rendered.
    pub config: RenderConfig,

    /// The resources required for rendering. TODO: Better docs.
    pub resources: RenderOwnedResources,

    spacefill_pass: RenderSpacefillPass,
    molecular_surface_pass: RenderMolecularSurfacePass,
}

impl RenderJobs {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        dependencies: RenderDependencies,
    ) -> RenderJobs {
        let resources = RenderOwnedResources {
            light_resource: LightResource::new(device),
            camera_resource: CameraResource::new(device),
            depth_texture: DepthTexture::new(device, config),
            df_texture_front: DistanceFieldTexture::new(device, 1),
        };

        let spacefill_pass = RenderSpacefillPass::new(
            device,
            config,
            RenderSpacefillResources {
                molecule: dependencies.molecule_resource,
                camera: &resources.camera_resource,
            },
        );

        let molecular_surface_pass = RenderMolecularSurfacePass::new(
            device,
            config,
            RenderMolecularSurfaceResources {
                ses_grid: dependencies.ses_resource,
                df_texture: &resources.df_texture_front.render,
                camera: &resources.camera_resource,
                light: &resources.light_resource,
            },
        );

        Self {
            resources,
            spacefill_pass,
            molecular_surface_pass,
            config: RenderConfig::default(),
        }
    }

    pub fn execute(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        dependencies: RenderDependencies,
    ) {
        let depth_view = &self.resources.depth_texture.view;
        if self.config.render_spacefill {
            self.spacefill_pass.render(
                view,
                depth_view,
                encoder,
                self.config.clear_color,
                RenderSpacefillResources {
                    molecule: dependencies.molecule_resource,
                    camera: &self.resources.camera_resource,
                },
            );
        }

        if self.config.render_molecular_surface {
            self.molecular_surface_pass.render(
                view,
                depth_view,
                encoder,
                self.config.clear_color,
                RenderMolecularSurfaceResources {
                    ses_grid: dependencies.ses_resource,
                    df_texture: &self.resources.df_texture_front.render,
                    camera: &self.resources.camera_resource,
                    light: &self.resources.light_resource,
                },
            );
        }
    }

    /// On resize, the depth texture needs to be recreated.
    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.resources.depth_texture = DepthTexture::new(device, config);
    }
}
