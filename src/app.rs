use winit::{event::*, window::Window};

use crate::context::Context;
use crate::gui::{Gui, GuiOutput};
use crate::passes::{compute::ComputeJobs, render::Renderer, resources::ResourceRepo};
use crate::utils::input::Input;

pub struct App<'a> {
    context: Context<'a>,
    resources: ResourceRepo,

    compute: ComputeJobs,
    renderer: Renderer,
    input: Input,
    gui: Gui,
}

impl<'a> App<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let context = Context::new(window).await;
        let resources = ResourceRepo::new(&context);

        App {
            compute: ComputeJobs::new(&context, &resources),
            renderer: Renderer::new(&context, &resources),
            input: Input::default(),
            gui: Gui::new(window),

            context,
            resources,
        }
    }

    pub fn redraw(&mut self, window: &Window) {
        let (gui_output, gui_events) = self.gui.draw_frame(window);

        self.renderer.handle_events(&gui_events);
        self.resources
            .update(&self.context, &self.input, gui_events);

        self.render(gui_output);
        self.input.reset();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.resources.resize(&self.context);

            #[cfg(target_arch = "wasm32")]
            self.gui.force_resize(new_size, &self.context);
        }
    }

    // TODO: Refactor this
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        if !self.gui.handle_winit_event(window, event) {
            self.input.handle_window_event(event);
        }
        false
    }

    fn render(&mut self, gui_output: GuiOutput) {
        let mut encoder = self.context.get_command_encoder();

        self.compute.execute_passes(&self.resources, &mut encoder);
        self.renderer
            .render(&self.context, &self.resources, encoder, gui_output)
            .expect("Failed to render");
    }
}
