use winit::{event::*, window::Window};

use crate::context::Context;
use crate::gui::{Gui, GuiOutput};
use crate::passes::resources::ResourceRepo;
use crate::utils::input::Input;

use crate::passes::compute::ComputeJobs;
use crate::passes::render::Renderer;

pub struct App {
    pub context: Context,
    pub resources: ResourceRepo,

    pub compute: ComputeJobs,
    pub renderer: Renderer,
    pub input: Input,
    pub gui: Gui,

    pub frame_count: u64,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let context = Context::new(window).await;
        let resources = ResourceRepo::new(&context);

        App {
            compute: ComputeJobs::new(&context, &resources),
            renderer: Renderer::new(&context, &resources),
            input: Input::default(),
            gui: Gui::new(&context),

            context,
            resources,

            frame_count: 0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.resources.resize(&self.context);
        }
    }

    pub fn input<T>(&mut self, event: &Event<T>) {
        if !self.gui.handle_winit_event(event) {
            self.input.handle_winit_event(event);
        }
    }

    pub fn redraw(&mut self) {
        let (gui_output, gui_events) = self.gui.draw_frame();

        self.renderer.handle_events(&gui_events);
        self.resources
            .update(&self.context, &self.input, gui_events);

        self.render(gui_output);
    }

    fn render(&mut self, gui_output: GuiOutput) {
        let mut encoder = self.context.get_command_encoder();

        self.compute.execute_passes(&self.resources, &mut encoder);
        self.renderer
            .render(&self.context, &self.resources, encoder, gui_output)
            .expect("Failed to render");
    }
}
