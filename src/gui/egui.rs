use std::sync::Arc;
use winit::{event::WindowEvent, event_loop::EventLoopProxy, window::Window};

const MAX_TEXTURE_SIZE: usize = 4096;

use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use epi::*;

#[derive(Debug)]
pub enum Event {
    RequestRedraw,
}

struct RepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<Event>>);

impl epi::backend::RepaintSignal for RepaintSignal {
    fn request_repaint(&self) {
        self.0.lock().unwrap().send_event(Event::RequestRedraw).ok();
    }
}

unsafe impl Sync for RepaintSignal {}
unsafe impl Send for RepaintSignal {}

pub struct Gui {
    state: egui_winit::State,
    context: egui::Context,
    render_pass: egui_wgpu_backend::RenderPass,
    repaint_signal: Arc<RepaintSignal>,

    demo_app: egui_demo_lib::WrapApp,
}

impl Gui {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        event_loop_proxy: EventLoopProxy<Event>,
    ) -> Self {
        let state = egui_winit::State::new(MAX_TEXTURE_SIZE, &window);
        let context = egui::Context::default();
        let render_pass = RenderPass::new(&device, config.format, 1);
        let repaint_signal =
            std::sync::Arc::new(RepaintSignal(std::sync::Mutex::new(event_loop_proxy)));
        let demo_app = egui_demo_lib::WrapApp::default();

        Self {
            render_pass,
            state,
            context,
            repaint_signal,
            demo_app,
        }
    }

    pub fn handle_events(&mut self, event: &WindowEvent) -> bool {
        self.state.on_event(&self.context, event)
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let input = self.state.take_egui_input(&window);
        self.context.begin_frame(input);

        let app_output = epi::backend::AppOutput::default();

        let frame = epi::Frame::new(epi::backend::FrameData {
            info: epi::IntegrationInfo {
                name: "egui_example",
                web_info: None,
                cpu_usage: Some(0.0),
                native_pixels_per_point: Some(window.scale_factor() as _),
                prefer_dark_mode: None,
            },
            output: app_output,
            repaint_signal: self.repaint_signal.clone(),
        });

        // Draw the demo application.
        self.demo_app.update(&self.context, &frame);

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let output = self.context.end_frame();
        let paint_jobs = self.context.tessellate(output.shapes);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: config.width,
            physical_height: config.height,
            scale_factor: window.scale_factor() as f32,
        };
        self.render_pass
            .add_textures(&device, &queue, &output.textures_delta)
            .unwrap();

        self.render_pass
            .remove_textures(output.textures_delta)
            .unwrap();

        self.render_pass
            .update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

        self.render_pass
            .execute(encoder, &view, &paint_jobs, &screen_descriptor, None)
            .unwrap();
    }
}
