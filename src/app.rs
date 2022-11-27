use std::sync::mpsc::{channel, Receiver};

use winit::{event::*, window::Window};

use crate::context::Context;
use crate::gui::Gui;
use crate::passes::resources::repo::MoleculeRepo;
use crate::passes::resources::GlobalResources;
use crate::shared::camera::ArcballCamera;
use crate::shared::events::AppEvent;
use crate::shared::input::Input;

use crate::passes::compute::{ComputeJobs, PassId};
use crate::passes::render::Renderer;

pub struct App {
    pub context: Context,
    pub resources: GlobalResources,

    pub compute: ComputeJobs,
    pub renderer: Renderer,
    pub camera: ArcballCamera,

    pub gui: Gui,
    pub input: Input,
    pub molecule_repo: MoleculeRepo,

    pub frame_count: u64,

    pub event_listener: Receiver<AppEvent>,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let context = Context::new(window).await;
        let resources = GlobalResources::new(&context);

        let (dispatch, event_listener) = channel::<AppEvent>();

        App {
            resources: GlobalResources::new(&context),

            compute: ComputeJobs::new(&context, &resources),
            renderer: Renderer::new(&context, &resources),
            camera: ArcballCamera::from_config(&context.config),

            gui: Gui::new(&context, dispatch.clone()),
            input: Input::default(),
            molecule_repo: MoleculeRepo::new(dispatch),

            frame_count: 0,
            event_listener,

            context,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.resources.resize(&self.context);
            self.camera.resize(new_size);
        }
    }

    pub fn input<T>(&mut self, event: &Event<T>) {
        if !self.gui.handle_winit_event(event) {
            self.input.handle_winit_event(event);
        }
    }

    pub fn redraw(&mut self) {
        self.process_app_events();
        self.camera.update(&self.input);
        self.resources
            .update_camera(&self.context.queue, &self.camera);
        self.render();
    }

    fn render(&mut self) {
        let gui_output = self.gui.draw_frame();

        let mut encoder = self.context.get_command_encoder();

        self.compute.execute_passes(&self.resources, &mut encoder);
        self.renderer
            .render(&self.context, &self.resources, encoder, gui_output)
            .expect("Failed to render");
    }

    fn process_app_events(&mut self) {
        while let Ok(event) = self.event_listener.try_recv() {
            match event {
                AppEvent::MoleculeChanged(molecule) => {
                    self.resources
                        .update_molecule(&self.context.queue, molecule);
                }
                AppEvent::SesResolutionChanged(resolution) => {
                    self.resources.update_resolution(&self.context, resolution);
                }
                AppEvent::ProbeRadiusChanged(probe_radius) => {
                    self.resources
                        .update_probe_radius(&self.context.queue, probe_radius);

                    self.molecule_repo.recompute_molecule_grids(probe_radius);
                }
                AppEvent::RenderSesChanged(enabled) => {
                    self.renderer
                        .toggle_render_pass(PassId::SesRaymarching, enabled);
                }
                AppEvent::RenderSpacefillChanged(enabled) => {
                    self.renderer.toggle_render_pass(PassId::Spacefill, enabled);
                }
                AppEvent::OpenFileDialogRequested => self.molecule_repo.load_pdb_files_from_user(),
                AppEvent::FilesLoaded(files) => {
                    self.molecule_repo.parse_molecules_and_grids(files, 1.4)
                }
                AppEvent::FocusCamera(position) => self.camera.set_target(position),
                AppEvent::DisplayError(_) => self.gui.handle_app_event(&event),
                _ => {}
            }
        }
    }
}
