use winit::event::*;

use crate::context::Context;
use crate::passes::{compute::ComputeJobs, render::RenderJobs, resources::ResourceRepo};
use crate::ui::UserInterface;

pub struct App {
    context: Context,
    resources: ResourceRepo,

    compute: ComputeJobs,
    render: RenderJobs,
    gui: UserInterface,
}

impl App {
    pub fn new(context: Context) -> Self {
        let resources = ResourceRepo::new(&context);

        App {
            compute: ComputeJobs::new(&context, &resources),
            render: RenderJobs::new(&context, &resources),
            gui: UserInterface::new(&context),

            context,
            resources,
        }
    }

    pub fn redraw(&mut self) {
        let gui_events = self.gui.process_frame();

        self.render.handle_events(&gui_events);
        self.resources
            .update(&self.context, &self.gui.input, gui_events);

        // Initialize rendering stuff.
        let mut encoder = self.context.get_command_encoder();
        let output_texture = self.context.surface.get_current_texture().unwrap();
        let view = output_texture.texture.create_view(&Default::default());
        let depth_view = self.resources.get_depth_texture().get_view();

        // Compute and render stuff.
        self.compute.execute(&self.resources, &mut encoder);
        self.render
            .execute(&self.context, &view, depth_view, &mut encoder);
        self.gui.render(&self.context, &view, &mut encoder);

        // Submit commands to the GPU.
        self.context.queue.submit(Some(encoder.finish()));

        // Draw a frame.
        output_texture.present();

        //
        //
        //

        // TODO: Remove this hotfix for texture switching
        if self.resources.just_switched {
            self.render = RenderJobs::new(&self.context, &self.resources);
            self.compute = ComputeJobs::new(&self.context, &self.resources);
        }
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
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        if !self.gui.handle_winit_event(event) {
            self.gui.input.handle_window_event(event);
        }
        false
    }
}
