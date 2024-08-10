mod constants;
mod data;
mod input;
mod ui;

use poms_common::{models::atom::calculate_center, resources::CommonResources};
use poms_compute::{ComputeJobs, ComputeParameters};
use poms_render::{RenderJobs, RenderParameters};

use crate::gpu_context::GpuContext;
use constants::ColorTheme;
use data::{molecule_parser::ParsedMolecule, molecule_storage::MoleculeStorage};
use input::{camera_controller::CameraController, mouse_input::MouseInput};
use ui::{events::UserEvent, UserInterface};

struct AppSettings {
    pub init_resolution: u32,
    pub target_resolution: u32,
    pub probe_radius: f32,
}

pub struct App {
    context: GpuContext,
    settings: AppSettings,

    compute: ComputeJobs,
    renderer: RenderJobs,
    resources: CommonResources,

    molecule_storage: MoleculeStorage,

    ui: UserInterface,
    mouse: MouseInput,
    camera: CameraController,
}

impl App {
    pub fn new(context: GpuContext) -> Self {
        let settings = AppSettings {
            init_resolution: 64,
            target_resolution: 256,
            probe_radius: 1.4,
        };

        // Load the initial molecule.
        let initial_molecule = ParsedMolecule::h2o_demo();
        let molecule_storage = MoleculeStorage::new(initial_molecule, settings.probe_radius);
        let atoms = &molecule_storage.get_current().atoms;

        // Upload the initial molecule to the GPU.
        let mut resources = CommonResources::new(&context.device);
        resources.atoms_resource.update(&context.queue, atoms);

        App {
            compute: ComputeJobs::new(
                &context.device,
                ComputeParameters {
                    molecule: &atoms.data,
                    common_resources: &resources,
                    init_resolution: settings.init_resolution,
                    target_resolution: settings.target_resolution,
                    probe_radius: settings.probe_radius,
                },
            ),
            renderer: RenderJobs::new(
                &context.device,
                RenderParameters {
                    common_resources: &resources,
                    surface_config: &context.config,
                    render_spacefill: true,
                    render_molecular_surface: true,
                    clear_color: wgpu::Color::BLACK,
                },
            ),
            settings,
            resources,
            molecule_storage,
            ui: UserInterface::new(&context),
            mouse: MouseInput::default(),
            camera: CameraController::from_config(&context.config),
            context,
        }
    }

    pub fn redraw(&mut self) {
        self.update_buffers();

        let ui_events = self.ui.process_frame();
        self.handle_ui_events(ui_events);

        let mut encoder = self.context.get_command_encoder();

        // Add commands to execute compute passes
        self.compute
            .execute(&mut encoder, &self.context.device, &self.resources);

        let output_texture = self.context.surface.get_current_texture().unwrap();
        let view = output_texture.texture.create_view(&Default::default());

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

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.ui.handle_window_event(event) || self.mouse.handle_window_event(event)
    }

    fn update_buffers(&mut self) {
        self.camera.update(&self.mouse);

        self.renderer.update_camera(
            &self.context.queue,
            self.camera.position,
            self.camera.view_matrix,
            self.camera.projection_matrix(),
        );

        self.renderer
            .update_light(&self.context.queue, self.camera.look_direction());

        self.compute.update_buffers(&self.context.queue);

        // If there is a new resolution of molecular surface available, use it.
        if let Some((texture, grid)) = self.compute.last_computed_distance_field() {
            self.renderer
                .update_distance_field_texture(&self.context.device, texture, grid);
        }
    }

    fn handle_ui_events(&mut self, ui_events: Vec<UserEvent>) {
        for event in ui_events {
            match event {
                UserEvent::RenderMolecularSurfaceChanged { is_enabled } => {
                    self.renderer.toggle_molecular_surface(is_enabled);
                }
                UserEvent::RenderSpacefillChanged { is_enabled } => {
                    self.renderer.toggle_spacefill(is_enabled);
                }
                UserEvent::ToggleTheme { theme } => {
                    self.renderer.change_clear_color(match theme {
                        ColorTheme::Dark => wgpu::Color::BLACK,
                        ColorTheme::Light => wgpu::Color::WHITE,
                    });
                }
                UserEvent::LoadedMolecule { molecule } => {
                    self.on_molecule_loaded(molecule);
                    self.reset_compute_jobs();
                }
                UserEvent::DistanceFieldResolutionChanged { resolution } => {
                    self.settings.target_resolution = resolution;
                    self.reset_compute_jobs();
                }
                UserEvent::ProbeRadiusChanged { probe_radius } => {
                    self.settings.probe_radius = probe_radius;
                    self.molecule_storage.on_probe_radius_changed(probe_radius);
                    self.reset_compute_jobs();
                }
                UserEvent::ToggleAnimation => {
                    // TODO: Fix animations (custom module)
                    // TODO: Recreate ComputeJobs?
                }
                UserEvent::AnimationSpeedChanged { speed: _ } => {
                    // TODO: Fix animations
                }
                UserEvent::OpenFileDialog => self.ui.open_file_dialog(),
            }
        }
    }

    fn on_molecule_loaded(&mut self, molecule: ParsedMolecule) {
        let processed_molecule = self
            .molecule_storage
            .add_from_parsed(molecule, self.settings.probe_radius); // TODO: Remove hardcoded probe radius

        self.camera
            .set_target(calculate_center(&processed_molecule.atoms.data));

        self.resources
            .atoms_resource
            .update(&self.context.queue, &processed_molecule.atoms);
    }

    fn reset_compute_jobs(&mut self) {
        self.compute = ComputeJobs::new(
            &self.context.device,
            ComputeParameters {
                molecule: &self.molecule_storage.get_current().atoms.data,
                common_resources: &self.resources,
                init_resolution: self.settings.init_resolution,
                target_resolution: self.settings.target_resolution,
                probe_radius: self.settings.probe_radius,
            },
        )
    }
}
