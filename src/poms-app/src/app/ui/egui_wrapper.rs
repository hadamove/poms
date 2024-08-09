use std::sync::Arc;

use crate::gpu_context::GpuContext;

use super::{elements::UiElement, UIState};

/// TODO: Add documentation, glue between winit, wgpu and egui
pub struct EguiWrapper {
    pub egui_handle: egui::Context,
    egui_winit_state: egui_winit::State,

    window: Arc<winit::window::Window>,
    renderer: egui_wgpu::Renderer,

    render_recipe: Option<egui::FullOutput>,
}

impl EguiWrapper {
    pub fn new(context: &GpuContext) -> Self {
        let egui_handle = egui::Context::default();

        let egui_winit_state = egui_winit::State::new(
            egui_handle.clone(),
            egui::ViewportId::ROOT,
            context.window.as_ref(),
            Some(context.window.scale_factor() as f32),
            None,
        );

        let renderer = egui_wgpu::Renderer::new(&context.device, context.config.format, None, 1);

        Self {
            window: context.window.clone(),
            egui_handle,
            egui_winit_state,
            renderer,
            render_recipe: None,
        }
    }

    pub fn add_elements(&mut self, state: &mut UIState, elements: &[UiElement]) {
        self.begin_frame();

        for &element in elements {
            element(&self.egui_handle, state);
        }

        self.end_frame();
    }

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_winit_state
            .on_window_event(&self.window, event)
            .consumed
    }

    pub fn render(
        &mut self,
        context: &GpuContext,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let Some(render_recipe) = self.render_recipe.take() else {
            return;
        };

        let pixels_per_point = self.window.scale_factor() as f32;

        let primitives = self
            .egui_handle
            .tessellate(render_recipe.shapes, pixels_per_point);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [context.config.width, context.config.height],
            pixels_per_point,
        };

        for (texture_id, image_delta) in render_recipe.textures_delta.set {
            self.renderer
                .update_texture(&context.device, &context.queue, texture_id, &image_delta);
        }

        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &primitives,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.renderer
                .render(&mut render_pass, &primitives, &screen_descriptor);
        }

        for free_id in render_recipe.textures_delta.free {
            self.renderer.free_texture(&free_id);
        }
    }

    fn begin_frame(&mut self) {
        let egui_input = self.egui_winit_state.take_egui_input(&self.window);
        self.egui_handle.begin_frame(egui_input);
    }

    fn end_frame(&mut self) {
        let render_recipe = self.egui_handle.end_frame();
        self.render_recipe = Some(render_recipe);
    }
}
