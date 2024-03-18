use winit::event::*;

use crate::{
    common::{models::atom::calculate_center, resources::CommonResources},
    compute::{composer::ComputeJobs, resources::df_texture::DistanceFieldTextureCompute},
    context::Context,
    render::{composer::RenderJobs, resources::df_texture::DistanceFieldTextureRender},
};

use self::{
    camera::CameraController,
    constants::ColorTheme,
    storage::MoleculeStorage,
    ui::{event::UserEvent, UserInterface},
};

pub mod camera;
pub mod constants;
pub mod dtos; // TODO: Refactor this into something else please
pub mod file;
pub mod input;
pub mod storage;
pub mod ui;
pub mod utils;

pub struct App {
    context: Context,

    // TODO: These are imported from outside crates
    compute: ComputeJobs,
    render: RenderJobs,

    // TODO: These are within app/ submodules
    ui: UserInterface,
    storage: MoleculeStorage,
    resources: CommonResources,
    camera: CameraController,
}

impl App {
    pub fn new(context: Context) -> Self {
        // Resources that are shared between render and compute passes.
        let resources = CommonResources::new(&context.device);

        App {
            storage: MoleculeStorage::new(),
            compute: ComputeJobs::new(&context.device, &resources),
            render: RenderJobs::new(&context.device, &context.config, &resources),
            ui: UserInterface::new(&context),
            camera: CameraController::from_config(&context.config),
            context,
            resources,
        }
    }

    pub fn redraw(&mut self) {
        let ui_events = self.ui.process_frame();

        self.update_resources();
        self.handle_ui_events(ui_events);

        let mut encoder = self.context.get_command_encoder();
        let output_texture = self.context.surface.get_current_texture().unwrap();
        let view = output_texture.texture.create_view(&Default::default());

        // TODO: Bad workaround
        if self.storage.get_current().is_some() {
            self.update_compute_progress();
            self.compute.execute(&mut encoder, &self.resources);
        }

        self.render.execute(&view, &mut encoder, &self.resources);
        self.ui.render(&self.context, &view, &mut encoder);

        // Submit commands to the GPU.
        self.context.queue.submit(Some(encoder.finish()));

        // Draw a frame.
        output_texture.present();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.render
                .resize(&self.context.device, &self.context.config);
            self.camera.resize(&self.context.config);

            #[cfg(target_arch = "wasm32")]
            self.ui.force_resize(new_size, &self.context);
        }
    }

    // TODO: Refactor this
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        if !self.ui.handle_winit_event(event) {
            self.ui.input.handle_window_event(event);
        }
        false
    }

    // TODO: Refactor this
    fn update_resources(&mut self) {
        self.camera.update(&self.ui.input);
        self.render
            .resources
            .camera_resource
            .update(&self.context.queue, &self.camera);
        self.render
            .resources
            .light_resource
            .update_camera(&self.context.queue, &self.camera);
    }

    // TODO: Refactor
    fn update_compute_progress(&mut self) {
        let Some(molecule) = self.storage.get_current() else {
            return;
        };

        let progress = self.compute.progress.clone();
        if let Some(render_resolution) = progress.last_computed_resolution {
            if render_resolution != self.render.resources.df_texture.resolution() {
                // New resolution has been computed, swap the texture
                let new_compute_texture = DistanceFieldTextureCompute::new_with_resolution(
                    &self.context.device,
                    progress.current_resolution,
                );

                let old_compute_texture =
                    std::mem::replace(&mut self.compute.resources.df_texture, new_compute_texture);

                self.render.resources.df_texture = DistanceFieldTextureRender::from_texture(
                    &self.context.device,
                    old_compute_texture.texture,
                );
            }
        }
        self.resources
            .ses_resource
            .update(&self.context.queue, &molecule.atoms.data, progress);
    }

    fn handle_ui_events(&mut self, ui_events: Vec<UserEvent>) {
        for event in ui_events {
            match event {
                UserEvent::RenderSesChanged(enabled) => {
                    self.render.config.render_molecular_surface = enabled;
                }
                UserEvent::RenderSpacefillChanged(enabled) => {
                    self.render.config.render_spacefill = enabled;
                }
                UserEvent::ToggleTheme(theme) => {
                    self.render.config.clear_color = match theme {
                        ColorTheme::Dark => wgpu::Color::BLACK,
                        ColorTheme::Light => wgpu::Color::WHITE,
                    };
                }
                UserEvent::LoadedMolecule(molecule) => {
                    // TODO: Recreate ComputeJobs
                    self.storage.add_from_parsed(molecule, 1.4); // TODO: Remove hardcoded probe radius

                    if let Some(current) = self.storage.get_current() {
                        self.camera
                            .set_target(calculate_center(&current.atoms.data));

                        self.resources
                            .molecule_resource
                            .update(&self.context.queue, &current.atoms);
                    }
                }
                UserEvent::SesResolutionChanged(_resolution) => {
                    // TODO: Recreate ComputeJobs?
                }
                UserEvent::ProbeRadiusChanged(probe_radius) => {
                    self.storage.on_probe_radius_changed(probe_radius);
                    // TODO: Recreate ComputeJobs?
                }
                UserEvent::ToggleAnimation => {
                    // TODO: Fix animations (custom module)
                    // TODO: Recreate ComputeJobs?
                }
                UserEvent::AnimationSpeedChanged(_) => {
                    // TODO: Fix animations
                }
                UserEvent::UpdateLight(light_data) => {
                    // TODO: Make this nicer
                    self.render
                        .resources
                        .light_resource
                        .update(&self.context.queue, light_data);
                }
                _ => {}
            }
        }
    }
}
