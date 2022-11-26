use std::sync::mpsc::{channel, Receiver};

use winit::{event::*, window::Window};

use crate::compute::{ComputeJobs, PassId};
use crate::gpu::GpuState;
use crate::gui::Gui;
use crate::parser::store::MoleculeStore;
use crate::render::Renderer;
use crate::shared::camera::ArcballCamera;
use crate::shared::events::AppEvent;
use crate::shared::input::Input;
use crate::shared::resources::GlobalResources;

pub struct App {
    pub gpu: GpuState,
    pub global_resources: GlobalResources,

    pub compute: ComputeJobs,
    pub renderer: Renderer,
    pub camera: ArcballCamera,

    pub gui: Gui,
    pub input: Input,
    pub store: MoleculeStore,

    pub frame_count: u64,
    pub last_frame_time: f32,

    pub event_listener: Receiver<AppEvent>,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let gpu = GpuState::new(window).await;
        let global_resources = GlobalResources::new(&gpu);

        let (dispatch, event_listener) = channel::<AppEvent>();

        App {
            global_resources: GlobalResources::new(&gpu),

            compute: ComputeJobs::new(&gpu, &global_resources),
            renderer: Renderer::new(&gpu, &global_resources),
            camera: ArcballCamera::from_config(&gpu.config),

            gui: Gui::new(&gpu, dispatch.clone()),
            input: Input::default(),
            store: MoleculeStore::new(dispatch),

            frame_count: 0,
            last_frame_time: 0.0,
            event_listener,

            gpu,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.gpu.resize(new_size);
            self.global_resources.resize(&self.gpu);
            self.camera.resize(new_size);
        }
    }

    pub fn input<T>(&mut self, event: &Event<T>) {
        if !self.gui.handle_winit_event(event) {
            self.input.handle_winit_event(event);
        }
    }

    pub fn render(&mut self, _window: &Window) -> anyhow::Result<()> {
        let gui_output = self.gui.draw_frame();

        let mut encoder = self.gpu.get_command_encoder();

        self.compute
            .execute_passes(&self.global_resources, &mut encoder);
        self.renderer
            .render(&self.gpu, &self.global_resources, encoder, gui_output)?;

        Ok(())
    }

    pub fn update(&mut self, _time_delta: std::time::Duration) {
        self.process_app_events();
        self.camera.update(&self.input);
        self.global_resources
            .update_camera(&self.gpu.queue, &self.camera);

        // self.frame_count += 1;
        // self.gui.frame_time = 0.9 * self.gui.frame_time + 0.1 * time_delta.as_secs_f32();
    }

    fn process_app_events(&mut self) {
        while let Ok(event) = self.event_listener.try_recv() {
            match event {
                AppEvent::MoleculeChanged(molecule) => {
                    self.global_resources
                        .update_molecule(&self.gpu.queue, molecule);
                }
                AppEvent::SesResolutionChanged(resolution) => {
                    self.global_resources
                        .update_resolution(&self.gpu, resolution);
                }
                AppEvent::ProbeRadiusChanged(probe_radius) => {
                    self.global_resources
                        .update_probe_radius(&self.gpu.queue, probe_radius);

                    self.store.recompute_molecule_grids(probe_radius);
                }
                AppEvent::RenderSesChanged(enabled) => {
                    self.renderer
                        .toggle_render_pass(PassId::SesRaymarching, enabled);
                }
                AppEvent::RenderSpacefillChanged(enabled) => {
                    self.renderer.toggle_render_pass(PassId::Spacefill, enabled);
                }
                AppEvent::OpenFileDialogRequested => self.store.load_pdb_files_from_user(),
                AppEvent::FilesLoaded(files) => self.store.parse_molecules_and_grids(files, 1.4),
                AppEvent::FocusCamera(position) => self.camera.set_target(position),
                AppEvent::DisplayError(_) => self.gui.handle_app_event(&event),
                _ => {}
            }
        }
    }
}
