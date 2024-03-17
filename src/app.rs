use winit::event::*;

use crate::context::Context;

use crate::passes::compute::ComputeDependencies;
use crate::passes::render::RenderDependencies;
use crate::passes::resources::atom::calculate_center;
use crate::passes::resources::molecule::MoleculeStorage;
use crate::passes::resources::textures::df_texture::{
    DistanceFieldTextureCompute, DistanceFieldTextureRender,
};
use crate::passes::{compute::ComputeJobs, render::RenderJobs, resources::CommonResources};
use crate::ui::event::UserEvent;
use crate::ui::UserInterface;
use crate::utils::constants::ColorTheme;

pub struct App {
    context: Context,
    storage: MoleculeStorage,
    resources: CommonResources,

    compute: ComputeJobs,
    render: RenderJobs,
    ui: UserInterface,
}

impl App {
    pub fn new(context: Context) -> Self {
        let resources = CommonResources::new(&context.device, &context.config);

        App {
            storage: MoleculeStorage::new(),
            compute: ComputeJobs::new(
                &context.device,
                ComputeDependencies {
                    molecule: &resources.molecule_resource,
                    ses_grid: &resources.ses_resource,
                },
            ),
            render: RenderJobs::new(
                &context.device,
                &context.config,
                RenderDependencies {
                    molecule_resource: &resources.molecule_resource,
                    ses_resource: &resources.ses_resource,
                },
            ),
            ui: UserInterface::new(&context),

            context,
            resources,
        }
    }

    pub fn redraw(&mut self) {
        let ui_events = self.ui.process_frame();

        self.update_resources();
        self.handle_ui_events(ui_events);

        // Initialize rendering stuff.
        let mut encoder = self.context.get_command_encoder();
        let output_texture = self.context.surface.get_current_texture().unwrap();
        let view = output_texture.texture.create_view(&Default::default());

        // TODO: Bad workaround
        if self.storage.get_current().is_some() {
            self.update_compute_progress();
            self.compute.execute(
                &mut encoder,
                ComputeDependencies {
                    molecule: &self.resources.molecule_resource,
                    ses_grid: &self.resources.ses_resource,
                },
            );
        }

        self.render.execute(
            &view,
            &mut encoder,
            RenderDependencies {
                molecule_resource: &self.resources.molecule_resource,
                ses_resource: &self.resources.ses_resource,
            },
        );
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
            self.resources
                .resize(&self.context.device, &self.context.config);

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
        self.resources.camera_controller.update(&self.ui.input);
        self.render
            .resources
            .camera_resource
            .update(&self.context.queue, &self.resources.camera_controller);
        self.render
            .resources
            .light_resource
            .update_camera(&self.context.queue, &self.resources.camera_controller);
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
                        self.resources
                            .camera_controller
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
