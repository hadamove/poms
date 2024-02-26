use winit::event::*;

use crate::context::Context;

use crate::passes::resources::atom::calculate_center;
use crate::passes::resources::molecule::MoleculeStorage;
use crate::passes::resources::textures::df_texture::DistanceFieldTexture;
use crate::passes::{compute::ComputeJobs, render::RenderJobs, resources::ResourceRepo};
use crate::ui::event::UserEvent;
use crate::ui::UserInterface;

pub struct App {
    context: Context,
    storage: MoleculeStorage,
    resources: ResourceRepo,

    compute: ComputeJobs,
    render: RenderJobs,
    ui: UserInterface,
}

impl App {
    pub fn new(context: Context) -> Self {
        let resources = ResourceRepo::new(&context);

        App {
            storage: MoleculeStorage::new(),
            compute: ComputeJobs::new(&context, &resources),
            render: RenderJobs::new(&context, &resources),
            ui: UserInterface::new(&context),

            context,
            resources,
        }
    }

    pub fn redraw(&mut self) {
        let ui_events = self.ui.process_frame();

        self.render.handle_events(&ui_events);
        self.resources.update(&self.context, &self.ui.input);
        self.handle_ui_events(ui_events);

        // Initialize rendering stuff.
        let mut encoder = self.context.get_command_encoder();
        let output_texture = self.context.surface.get_current_texture().unwrap();
        let view = output_texture.texture.create_view(&Default::default());

        // TODO: Bad workaround
        if self.storage.get_current().is_some() {
            self.update_compute_progress();
            self.compute.execute(&mut encoder);
        }

        let depth_view = self.resources.get_depth_texture().get_view();
        self.render
            .execute(&self.context, &view, depth_view, &mut encoder);
        self.ui.render(&self.context, &view, &mut encoder);

        // Submit commands to the GPU.
        self.context.queue.submit(Some(encoder.finish()));

        // Draw a frame.
        output_texture.present();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.resources.resize(&self.context);

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

    // TODO: Refactor
    fn update_compute_progress(&mut self) {
        let Some(molecule) = self.storage.get_current() else {
            return;
        };

        let progress = self.compute.progress.clone();
        if let Some(render_resolution) = progress.last_computed_resolution {
            if render_resolution != self.resources.df_texture_front.resolution() {
                // New resolution has been computed, swap the texture
                self.resources.df_texture_front = std::mem::replace(
                    &mut self.resources.df_texture_back,
                    DistanceFieldTexture::new(&self.context.device, progress.current_resolution),
                );
                // Recreate passes with new resources
                self.render = RenderJobs::new(&self.context, &self.resources);
                self.compute.recreate_passes(&self.context, &self.resources);
            }
        }
        self.resources
            .ses_resource
            .update(&self.context.queue, &molecule.atoms.data, progress);
    }

    fn handle_ui_events(&mut self, ui_events: Vec<UserEvent>) {
        for event in ui_events {
            match event {
                UserEvent::LoadedMolecule(molecule) => {
                    // TODO: Recreate ComputeJobs
                    self.storage.add_from_parsed(molecule, 1.4); // TODO: Remove hardcoded probe radius

                    if let Some(current) = self.storage.get_current() {
                        self.resources
                            .camera
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
                    self.resources
                        .light_resource
                        .update(&self.context.queue, light_data);
                }
                _ => {}
            }
        }
    }
}
