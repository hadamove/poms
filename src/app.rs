use std::sync::mpsc::{channel, Receiver};

use winit::{event::*, window::Window};

use crate::compute::{ComputeJobs, PassId};
use crate::gpu::GpuState;
use crate::gui::Gui;
use crate::parser::store::MoleculeStore;
use crate::render::Renderer;
use crate::shared::events::AppEvent;

pub struct App {
    pub gpu: GpuState,

    pub compute: ComputeJobs,
    pub renderer: Renderer,

    pub gui: Gui,
    pub store: MoleculeStore,

    pub frame_count: u64,
    pub last_frame_time: f32,

    pub event_listener: Receiver<AppEvent>,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let gpu = GpuState::new(window).await;

        let (dispatch, event_listener) = channel::<AppEvent>();
        App {
            compute: ComputeJobs::new(&gpu),
            renderer: Renderer::new(&gpu),

            gui: Gui::new(&gpu, dispatch.clone()),
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
            self.renderer.resize(new_size);
        }
    }

    pub fn input<T>(&mut self, event: &Event<T>) -> bool {
        self.gui.handle_winit_event(event)
            || self.renderer.camera_controller.process_winit_event(event)
    }

    pub fn render(&mut self, _window: &Window) -> anyhow::Result<()> {
        let gui_output = self.gui.draw_frame();

        let mut encoder = self.gpu.get_command_encoder();

        self.compute.execute_passes(&self.gpu, &mut encoder);
        self.renderer.render(&self.gpu, encoder, gui_output)?;

        Ok(())
    }

    pub fn update(&mut self, time_delta: std::time::Duration) {
        self.process_app_events();
        // self.frame_count += 1;
        // self.gui.frame_time = 0.9 * self.gui.frame_time + 0.1 * time_delta.as_secs_f32();
        self.renderer.update(&mut self.gpu, time_delta);
    }

    fn process_app_events(&mut self) {
        while let Ok(event) = self.event_listener.try_recv() {
            match event {
                AppEvent::MoleculeChanged(molecule) => {
                    self.gpu
                        .global_resources
                        .update_molecule(&self.gpu.queue, molecule);
                }
                AppEvent::SesResolutionChanged(resolution) => {
                    self.gpu.global_resources.update_resolution(
                        &self.gpu.queue,
                        &self.gpu.device,
                        resolution,
                    );
                }
                AppEvent::ProbeRadiusChanged(probe_radius) => {
                    self.gpu
                        .global_resources
                        .update_probe_radius(&self.gpu.queue, probe_radius);

                    self.store.recompute_molecule_grids(probe_radius);
                }
                AppEvent::RenderSesChanged(enabled) => {
                    self.renderer
                        .toggle_render_pass(PassId::RaymarchPass, enabled);
                }
                AppEvent::RenderSpacefillChanged(enabled) => {
                    self.renderer
                        .toggle_render_pass(PassId::SpacefillPass, enabled);
                }
                AppEvent::OpenFileDialogRequested => self.store.load_pdb_files_from_user(),
                AppEvent::FilesLoaded(files) => self.store.parse_molecules_and_grids(files, 1.4),
                AppEvent::FocusCamera(position) => {
                    self.renderer.camera.focus(position);
                }
                AppEvent::DisplayError(_) => self.gui.handle_app_event(&event),
                _ => {}
            }
        }
    }
}
