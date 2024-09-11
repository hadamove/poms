mod anim;
mod data;
mod input;
mod theme;
mod ui;

use poms_common::limits::{
    MAX_DISTANCE_FIELD_RESOLUTION, MIN_DISTANCE_FIELD_RESOLUTION, MIN_PROBE_RADIUS,
};
use poms_common::{models::atom::calculate_center, resources::CommonResources};
use poms_compute::{ComputeJobs, ComputeParameters};
use poms_render::{PostprocessSettings, RenderJobs, RenderParameters};

use super::gpu_context::GpuContext;
use anim::AnimationController;
use data::{molecule_parser::ParsedMolecule, molecule_storage::MoleculeStorage};
use input::{camera_controller::CameraController, mouse_input::MouseInput};
use ui::{events::UserEvent, state::UIState, UserInterface};

/// Settings for the application, controlling resolution and probe radius.
struct AppSettings {
    init_resolution: u32,
    target_resolution: u32,
    probe_radius: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            init_resolution: MIN_DISTANCE_FIELD_RESOLUTION,
            target_resolution: MAX_DISTANCE_FIELD_RESOLUTION,
            probe_radius: MIN_PROBE_RADIUS,
        }
    }
}

/// Represents the main application, managing rendering, compute jobs, and user interactions.
pub(crate) struct App {
    context: GpuContext,
    settings: AppSettings,

    compute: ComputeJobs,
    renderer: RenderJobs,
    resources: CommonResources,

    molecule_storage: MoleculeStorage,

    ui: UserInterface,
    mouse: MouseInput,
    camera: CameraController,
    animation: AnimationController,
}

impl App {
    /// Initializes the application with default settings and sets up rendering and compute jobs.
    pub(crate) fn new(context: GpuContext) -> Self {
        let settings = AppSettings::default();

        let initial_molecule = ParsedMolecule::h2o_demo();
        let molecule_storage = MoleculeStorage::new(initial_molecule, settings.probe_radius);
        let resources = CommonResources::new(&context.device);

        let render_spacefill = true;
        let render_molecular_surface = false;
        let postprocess_settings = PostprocessSettings::default();
        let animation = AnimationController::default();

        App {
            compute: ComputeJobs::new(
                &context.device,
                ComputeParameters {
                    molecule: &molecule_storage.get_active().atoms.data,
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
                    queue: &context.queue,
                    render_spacefill,
                    render_molecular_surface,
                    postprocess_settings,
                    clear_color: wgpu::Color::BLACK,
                },
            ),
            resources,
            molecule_storage,
            ui: UserInterface::new(
                &context,
                UIState {
                    target_resolution: settings.target_resolution,
                    probe_radius: settings.probe_radius,
                    render_spacefill,
                    render_molecular_surface,
                    is_animation_active: animation.is_active,
                    animation_speed: animation.speed,
                    // This ensures the initial molecule is added to the UI state.
                    events: vec![UserEvent::ChangeActiveMolecule { index: 0 }],
                    ..Default::default()
                },
            ),
            mouse: MouseInput::default(),
            camera: CameraController::from_config(&context.config),
            animation,
            context,
            settings,
        }
    }

    /// Handles the rendering of each frame, processing user interactions,
    /// updating buffers, and submitting commands to the GPU.
    pub(crate) fn redraw(&mut self) {
        self.update_buffers();

        let user_events = self.ui.process_frame();
        self.handle_user_events(user_events);

        let mut encoder = self.context.get_command_encoder();

        self.compute
            .execute(&mut encoder, &self.context.device, &self.resources);

        let output_texture = self.context.surface.get_current_texture().unwrap();
        let output_texture_view = output_texture.texture.create_view(&Default::default());

        self.renderer
            .render(&output_texture_view, &mut encoder, &self.resources);
        self.ui
            .render(&self.context, &output_texture_view, &mut encoder);

        self.context.queue.submit(Some(encoder.finish()));

        output_texture.present();

        if self.animation.advance_tick() {
            // Advance to the next frame if the animation is active and due.
            self.molecule_storage.increment_active();
            self.on_active_molecule_changed();
        }
    }

    /// Resizes the application when the window size changes, updating the renderer and camera.
    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.renderer
                .resize(&self.context.device, &self.context.config);
            self.camera.resize(&self.context.config);
        }
    }

    /// Handles window events like resizing or input, returning true if the event was consumed.
    pub(crate) fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.ui.handle_window_event(event) || self.mouse.handle_window_event(event)
    }

    /// Handles device events (e.g., mouse motion) that are not tied to a specific window.
    pub(crate) fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        self.mouse.handle_device_event(event);
    }

    fn update_buffers(&mut self) {
        self.camera.update(&self.mouse);
        self.mouse.decay_input();

        self.renderer.update_camera(
            &self.context.queue,
            self.camera.position,
            self.camera.view_matrix,
            self.camera.projection_matrix(),
        );

        self.renderer
            .update_light(&self.context.queue, self.camera.look_direction());

        self.compute.update_buffers(&self.context.queue);

        // Update the molecular surface texture if a new one is computed.
        if let Some((texture, grid)) = self.compute.last_computed_distance_field() {
            self.renderer
                .update_distance_field_texture(&self.context.device, texture, grid);
        }
    }

    /// Processes events generated by the UI, updating the application state accordingly.
    fn handle_user_events(&mut self, ui_events: Vec<UserEvent>) {
        for event in ui_events {
            match event {
                UserEvent::ChangeRenderMolecularSurface { is_enabled } => {
                    self.renderer.toggle_molecular_surface_pass(is_enabled);
                }
                UserEvent::ChangeRenderSpacefill { is_enabled } => {
                    self.renderer.toggle_spacefill_pass(is_enabled);
                }
                UserEvent::UpdatePostprocessSettings { settings } => {
                    self.renderer.update_postprocess_settings(settings);
                }
                UserEvent::ToggleTheme { theme } => {
                    self.renderer.update_clear_color(match theme {
                        theme::ColorTheme::Dark => wgpu::Color::BLACK,
                        theme::ColorTheme::Light => wgpu::Color::WHITE,
                    });
                }
                UserEvent::MoleculesParsed { molecules } => {
                    self.molecule_storage
                        .add_from_parsed(molecules, self.settings.probe_radius);
                    self.on_active_molecule_changed();
                }
                UserEvent::ChangeActiveMolecule { index } => {
                    self.molecule_storage.set_active(index);
                    self.on_active_molecule_changed();
                }
                UserEvent::ChangeDistanceFieldResolution { resolution } => {
                    self.settings.target_resolution = resolution;
                    self.reset_compute_jobs();
                }
                UserEvent::ChangeProbeRadius { probe_radius } => {
                    self.settings.probe_radius = probe_radius;
                    self.molecule_storage.on_probe_radius_changed(probe_radius);
                    self.update_atoms_resource();
                    self.reset_compute_jobs();
                }
                UserEvent::ToggleAnimation => {
                    self.animation.is_active = !self.animation.is_active;
                }
                UserEvent::ChangeAnimationSpeed { speed } => {
                    self.animation.speed = speed;
                }
                UserEvent::InitDownloadMolecule { assembly } => {
                    self.ui.file_loader.download_file(assembly);
                }
                UserEvent::InitMoleculeSearch { query } => {
                    self.ui.file_loader.search_pdb_files(query);
                }
                UserEvent::InitOpenFileDialog => {
                    self.ui.file_loader.pick_files();
                }
            }
        }
    }

    /// Handles changes necessary after different molecule was chosen to be displayed,
    /// updating the GPU resources, UI state, and setting the camera's focus to the new molecule.
    fn on_active_molecule_changed(&mut self) {
        self.update_atoms_resource();

        self.camera.set_target(calculate_center(
            &self.molecule_storage.get_active().atoms.data,
        ));

        // Update the state of the UI
        self.ui.update_files_state(
            &self.molecule_storage.loaded_molecules,
            self.molecule_storage.active_index,
        );

        // Finally, when mocule changes, we need to start computation from the beginning
        self.reset_compute_jobs();
    }

    /// Resets compute jobs with updated parameters when the resolution, probe radius, or rendered molecule changes.
    fn reset_compute_jobs(&mut self) {
        self.compute = ComputeJobs::new(
            &self.context.device,
            ComputeParameters {
                molecule: &self.molecule_storage.get_active().atoms.data,
                common_resources: &self.resources,
                init_resolution: self.settings.init_resolution,
                target_resolution: self.settings.target_resolution,
                probe_radius: self.settings.probe_radius,
            },
        )
    }

    /// Updates the atoms resource when the molecule data changes.
    fn update_atoms_resource(&mut self) {
        let active_molecule = self.molecule_storage.get_active();
        self.resources
            .atoms_resource
            .update(&self.context.queue, &active_molecule.atoms);
    }
}
