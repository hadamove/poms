use std::sync::mpsc::{channel, Receiver};

use winit::{event::*, window::Window};

use crate::compute::ComputeJobs;
use crate::gpu::GpuState;
use crate::gui::Gui;
use crate::parser::store::MoleculeStore;
use crate::render::Renderer;
use crate::shared::events::AppEvent;
use crate::shared::molecule::Molecule;

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

            gui: Gui::new(dispatch.clone()),
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
            self.renderer.resize(&self.gpu, new_size);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.renderer.camera_controller.process_events(event)
    }

    pub fn render(&mut self, _window: &Window) -> anyhow::Result<()> {
        let mut encoder = self.gpu.get_command_encoder();

        self.compute.execute_passes(&self.gpu, &mut encoder);
        self.renderer.render(&self.gpu, encoder, &mut self.gui)?;

        Ok(())
    }

    pub fn update(&mut self, time_delta: std::time::Duration) {
        self.process_events();
        // self.frame_count += 1;
        // self.gui.frame_time = 0.9 * self.gui.frame_time + 0.1 * time_delta.as_secs_f32();
        self.renderer.update(&self.gpu, time_delta);
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.event_listener.try_recv() {
            match event {
                AppEvent::MoleculeChanged(molecule) => {
                    self.compute
                        .probe_compute_pass
                        .on_molecule_changed(&self.gpu.queue, &molecule);
                    self.renderer
                        .spacefill_pass
                        .on_molecule_changed(&self.gpu.queue, &molecule);
                    self.renderer
                        .camera
                        .focus(molecule.atoms_sorted.calculate_center());
                    self.gpu
                        .shared_resources
                        .update_ses(&self.gpu.queue, &molecule);
                }
                AppEvent::SesResolutionChanged(resolution) => {
                    self.gpu.shared_resources.update_ses_resolution(
                        &self.gpu.device,
                        &self.gpu.queue,
                        resolution,
                    );
                }
                AppEvent::ProbeRadiusChanged(probe_radius) => {
                    self.gpu
                        .shared_resources
                        .update_probe_radius(&self.gpu.queue, probe_radius);

                    self.store.recompute_molecule_grids(probe_radius)
                }
                AppEvent::RenderSesChanged(render_ses) => {
                    self.renderer.settings.render_ses = render_ses;
                }
                AppEvent::RenderSpacefillChanged(render_spacefill) => {
                    self.renderer.settings.render_spacefill = render_spacefill;
                }
                AppEvent::OpenFileDialogRequested => self.store.load_pdb_files_from_user(),
                AppEvent::FilesLoaded(files) => self.store.parse_molecules_and_grids(files, 1.4),
                AppEvent::DisplayError(_) => self.gui.process_event(&event),
                _ => {}
            }
        }
    }
}
