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
use poms_render::{RenderJobs, RenderParameters};

use super::gpu_context::GpuContext;
use anim::AnimationController;
use data::{molecule_parser::ParsedMolecule, molecule_storage::MoleculeStorage};
use input::{camera_controller::CameraController, mouse_input::MouseInput};
use ui::{events::UserEvent, state::UIState, UserInterface};

/// Settings for the application, controlling resolution and probe radius.
struct AppSettings {
    pub init_resolution: u32,
    pub target_resolution: u32,
    pub probe_radius: f32,
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
    animation: AnimationController,
}

impl App {
    /// Initializes the application with default settings and sets up rendering and compute jobs.
    pub fn new(context: GpuContext) -> Self {
        let settings = AppSettings::default();

        let initial_molecule = ParsedMolecule::h2o_demo();
        let molecule_storage = MoleculeStorage::new(initial_molecule, settings.probe_radius);
        let resources = CommonResources::new(&context.device);

        let render_spacefill = true;
        let render_molecular_surface = false;
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
                    render_spacefill,
                    render_molecular_surface,
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
                    events: vec![UserEvent::ActivateFile { index: 0 }],
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
    pub fn redraw(&mut self) {
        self.update_buffers();

        let ui_events = self.ui.process_frame();
        self.handle_ui_events(ui_events);

        let mut encoder = self.context.get_command_encoder();

        self.compute
            .execute(&mut encoder, &self.context.device, &self.resources);

        let output_texture = self.context.surface.get_current_texture().unwrap();
        let view = output_texture.texture.create_view(&Default::default());

        self.renderer.render(&view, &mut encoder, &self.resources);
        self.ui.render(&self.context, &view, &mut encoder);

        self.context.queue.submit(Some(encoder.finish()));

        output_texture.present();

        if self.animation.advance_tick() {
            // Advance to the next frame if the animation is active and due.
            self.molecule_storage.increment_active();
            self.on_active_molecule_changed();
        }
    }

    /// Resizes the application when the window size changes, updating the renderer and camera.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.renderer
                .resize(&self.context.device, &self.context.config);
            self.camera.resize(&self.context.config);
        }
    }

    /// Handles window events like resizing or input, returning true if the event was consumed.
    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.ui.handle_window_event(event) || self.mouse.handle_window_event(event)
    }

    /// Handles device events (e.g., mouse motion) that are not tied to a specific window.
    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        self.mouse.handle_device_event(event);
    }

    /// Updates various buffers before rendering, including camera and lighting information.
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
                        theme::ColorTheme::Dark => wgpu::Color::BLACK,
                        theme::ColorTheme::Light => wgpu::Color::WHITE,
                    });
                }
                UserEvent::MoleculesLoaded { molecules } => {
                    self.molecule_storage
                        .add_from_parsed(molecules, self.settings.probe_radius);
                    self.on_active_molecule_changed();
                }
                UserEvent::ActivateFile { index } => {
                    self.molecule_storage.set_active(index);
                    self.on_active_molecule_changed();
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
                    self.animation.is_active = !self.animation.is_active;
                }
                UserEvent::AnimationSpeedChanged { speed } => {
                    self.animation.speed = speed;
                }
                UserEvent::OpenFileDialog => self.ui.open_file_dialog(),
            }
        }
    }

    /// Handles changes necessary after different molecule was chosen to be displayed,
    /// updating the GPU resources, UI state, and setting the camera's focus to the new molecule.
    fn on_active_molecule_changed(&mut self) {
        let active_molecule = self.molecule_storage.get_active();

        self.camera
            .set_target(calculate_center(&active_molecule.atoms.data));

        self.resources
            .atoms_resource
            .update(&self.context.queue, &active_molecule.atoms);

        // Update the state of the UI
        self.ui.set_files(&self.molecule_storage.loaded_molecules);
        self.ui.activate_file(self.molecule_storage.active_index);

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
}
