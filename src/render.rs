use anyhow::Result;

use passes::gui_pass::GuiRenderPass;

use shared::camera::{Camera, CameraController, Projection};

use crate::compute::PassId;
use crate::gui::GuiOutput;
use crate::render_pass::RenderPass;

use super::gpu::GpuState;

mod passes;
pub mod shared;

pub struct Renderer {
    // TODO: move this somewhere else
    pub camera: Camera,
    projection: Projection,
    pub camera_controller: CameraController,

    pub gui_pass: GuiRenderPass,
    render_passes: Vec<RenderPass>,
}

impl Renderer {
    pub fn new(gpu: &GpuState) -> Renderer {
        Self {
            camera: Camera::default(),
            projection: Projection::from_config(&gpu.config),
            camera_controller: CameraController::new(100.0, 0.3),

            gui_pass: GuiRenderPass::new(gpu),

            render_passes: vec![
                RenderPass::new(
                    gpu,
                    PassId::SpacefillPass,
                    wgpu::include_wgsl!("./shaders/spacefill.wgsl"),
                ),
                RenderPass::new(
                    gpu,
                    PassId::RaymarchPass,
                    wgpu::include_wgsl!("./shaders/raymarch.wgsl"),
                ),
            ],
        }
    }

    // TODO: move this somewhere else
    pub fn update(&mut self, gpu: &mut GpuState, time_delta: instant::Duration) {
        self.camera_controller
            .update_camera(&mut self.camera, time_delta);

        gpu.global_resources
            .camera_resource
            .update(&gpu.queue, &self.camera, &self.projection);
    }
    // TODO: move this elsewhere
    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.projection.resize(size.width, size.height);
    }

    pub fn toggle_render_pass(&mut self, pass_id: PassId, enabled: bool) {
        for pass in self.render_passes.iter_mut() {
            if pass.get_id() == &pass_id {
                pass.set_enabled(enabled);
            }
        }
    }

    pub fn render(
        &mut self,
        gpu: &GpuState,
        mut encoder: wgpu::CommandEncoder,
        gui_output: GuiOutput,
    ) -> Result<()> {
        // Obtain screen's texture to render to.
        let output_texture = gpu.surface.get_current_texture()?;

        let view = output_texture.texture.create_view(&Default::default());
        let depth_view = &gpu.global_resources.get_depth_texture().view;

        // Render Spacefill and Ses.
        for render_pass in &mut self.render_passes {
            render_pass.render(&view, depth_view, &mut encoder, &gpu.global_resources);
        }

        // Render GUI
        self.gui_pass.render(gpu, &view, &mut encoder, gui_output)?;

        // Submit commands to the GPU
        gpu.queue.submit(Some(encoder.finish()));

        // Draw a frame
        output_texture.present();

        Ok(())
    }
}
