use utils::parser::ParsedMolecule;
use winit::event::*;

use common::{models::atom::calculate_center, resources::CommonResources};
use compute::{ComputeJobs, ComputeParameters};
use render::resources::light::LightUniform;
use render::{RenderJobs, RenderParameters};

use self::{
    camera::CameraController,
    constants::ColorTheme,
    context::Context,
    storage::MoleculeStorage,
    ui::{event::UserEvent, UserInterface},
};

pub mod camera;
pub mod constants;
pub mod context;
pub mod file;
pub mod input;
pub mod storage;
pub mod ui;
pub mod utils;

pub struct App {
    context: Context,

    compute: ComputeJobs,
    renderer: RenderJobs,

    ui: UserInterface,
    molecules: MoleculeStorage,
    resources: CommonResources,
    camera: CameraController,
}

impl App {
    pub fn new(context: Context) -> Self {
        // Load the initial molecule.
        let initial_molecule = ParsedMolecule::h2o_demo();
        let molecule_storage = MoleculeStorage::new(initial_molecule);

        // Resources that are shared between render and compute passes.
        let mut resources = CommonResources::new(&context.device);
        resources
            .atoms_resource
            .update(&context.queue, &molecule_storage.get_current().atoms);

        App {
            compute: ComputeJobs::new(
                &context.device,
                ComputeParameters {
                    molecule: &molecule_storage.get_current().atoms.data,
                    common_resources: &resources,
                    init_resolution: 64,
                    target_resolution: 128,
                    probe_radius: 1.4,
                },
            ),
            renderer: RenderJobs::new(
                &context.device,
                RenderParameters {
                    common_resources: &resources,
                    surface_config: &context.config,
                    render_spacefill: false,
                    render_molecular_surface: true,
                    clear_color: wgpu::Color::BLACK,
                    number_of_atoms: molecule_storage.get_current().atoms.data.len() as u32,
                },
            ),
            ui: UserInterface::new(&context),
            camera: CameraController::from_config(&context.config),
            molecules: molecule_storage,
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

        // If there is a new resolution of molecular surface available, use it.
        if let Some((texture, grid)) = self.compute.last_computed_distance_field() {
            self.renderer
                .update_distance_field_texture(&self.context.device, texture, grid);
        }

        // Add commands to execute compute passes
        self.compute
            .execute(&mut encoder, &self.context.device, &self.resources);

        // Add commands to execute render passes
        self.renderer.render(&view, &mut encoder, &self.resources);
        self.ui.render(&self.context, &view, &mut encoder);

        // Submit commands to the GPU.
        self.context.queue.submit(Some(encoder.finish()));

        // Draw a frame.
        output_texture.present();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.renderer
                .resize(&self.context.device, &self.context.config);
            self.camera.resize(&self.context.config);

            #[cfg(target_arch = "wasm32")]
            self.ui.force_resize(new_size, &self.context);
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.ui.handle_window_event(event)
    }

    // TODO: Refactor this
    fn update_resources(&mut self) {
        self.camera.update(&self.ui.input);
        self.renderer
            .update_camera(&self.context.queue, self.camera.to_uniform());

        self.renderer.update_light(
            &self.context.queue,
            LightUniform::new(self.camera.look_direction().into()),
        );

        self.compute.update_buffers(&self.context.queue);
    }

    fn handle_ui_events(&mut self, ui_events: Vec<UserEvent>) {
        for event in ui_events {
            match event {
                UserEvent::RenderMolecularSurfaceChanged(is_enabled) => {
                    self.renderer.toggle_molecular_surface(is_enabled);
                }
                UserEvent::RenderSpacefillChanged(enabled) => {
                    self.renderer.toggle_spacefill(enabled);
                }
                UserEvent::ToggleTheme(theme) => {
                    self.renderer.change_clear_color(match theme {
                        ColorTheme::Dark => wgpu::Color::BLACK,
                        ColorTheme::Light => wgpu::Color::WHITE,
                    });
                }
                UserEvent::LoadedMolecule(molecule) => {
                    // TODO: Recreate ComputeJobs
                    let current = self.molecules.add_from_parsed(molecule, 1.4); // TODO: Remove hardcoded probe radius

                    self.camera
                        .set_target(calculate_center(&current.atoms.data));

                    self.resources
                        .atoms_resource
                        .update(&self.context.queue, &current.atoms);

                    self.compute = ComputeJobs::new(
                        &self.context.device,
                        ComputeParameters {
                            molecule: &current.atoms.data,
                            common_resources: &self.resources,
                            init_resolution: 64,
                            target_resolution: 256,
                            probe_radius: 1.4,
                        },
                    );
                }
                UserEvent::DistanceFieldResolutionChanged(resolution) => {
                    println!("res  changed {}", resolution);
                    let current = self.molecules.get_current();
                    self.compute = ComputeJobs::new(
                        &self.context.device,
                        ComputeParameters {
                            molecule: &current.atoms.data,
                            common_resources: &self.resources,
                            init_resolution: 64,
                            target_resolution: resolution,
                            probe_radius: 1.4,
                        },
                    );
                }
                UserEvent::ProbeRadiusChanged(probe_radius) => {
                    println!("probe radius changed {}", probe_radius);
                    self.molecules.on_probe_radius_changed(probe_radius);
                    let current = self.molecules.get_current();
                    self.compute = ComputeJobs::new(
                        &self.context.device,
                        ComputeParameters {
                            molecule: &current.atoms.data,
                            common_resources: &self.resources,
                            init_resolution: 64,
                            target_resolution: 256,
                            probe_radius,
                        },
                    );
                }
                UserEvent::ToggleAnimation => {
                    // TODO: Fix animations (custom module)
                    // TODO: Recreate ComputeJobs?
                }
                UserEvent::AnimationSpeedChanged(_) => {
                    // TODO: Fix animations
                }
                UserEvent::UpdateLight(uniform) => {
                    self.renderer.update_light(&self.context.queue, uniform);
                }
                UserEvent::OpenFileDialog => self.ui.open_file_dialog(),
                _ => {}
            }
        }
    }
}
