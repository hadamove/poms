use anyhow::Result;

use passes::gui_pass::GuiRenderPass;
use passes::raymarch_pass::RaymarchDistanceFieldPass;
use passes::spacefill_pass::SpacefillPass;

use shared::camera::{Camera, CameraController, Projection};
use shared::resources::{camera::CameraResource, texture::Texture};

use super::gpu::GpuState;
use super::gui::Gui;

mod passes;
mod shared;

pub struct RenderSettings {
    pub render_ses: bool,
    pub render_spacefill: bool,
}

pub struct Renderer {
    pub camera: Camera,
    projection: Projection,
    camera_resource: CameraResource,
    pub camera_controller: CameraController,

    depth_texture: Texture,

    pub gui_pass: GuiRenderPass,

    //TODO: add clear pass
    pub spacefill_pass: SpacefillPass,
    raymarch_pass: RaymarchDistanceFieldPass,

    pub settings: RenderSettings,
}

impl Renderer {
    pub fn new(gpu: &GpuState) -> Renderer {
        let camera_resource = CameraResource::new(&gpu.device);
        Self {
            camera: Camera::default(),
            projection: Projection::from_config(&gpu.config),
            camera_controller: CameraController::new(100.0, 0.3),

            depth_texture: Texture::create_depth_texture(&gpu.device, &gpu.config),
            gui_pass: GuiRenderPass::new(gpu),
            spacefill_pass: SpacefillPass::new(gpu, &camera_resource),
            raymarch_pass: RaymarchDistanceFieldPass::new(gpu, &camera_resource),

            camera_resource,

            settings: RenderSettings {
                render_ses: true,
                render_spacefill: true,
            },
        }
    }

    pub fn resize(&mut self, gpu: &GpuState, size: winit::dpi::PhysicalSize<u32>) {
        self.projection.resize(size.width, size.height);
        self.depth_texture = Texture::create_depth_texture(&gpu.device, &gpu.config);
    }

    pub fn update(&mut self, gpu: &GpuState, time_delta: instant::Duration) {
        self.camera_controller
            .update_camera(&mut self.camera, time_delta);

        self.camera_resource
            .update(&gpu.queue, &self.camera, &self.projection);
    }

    pub fn render(
        &mut self,
        gpu: &GpuState,
        mut encoder: wgpu::CommandEncoder,
        gui: &mut Gui,
    ) -> Result<()> {
        // Obtain screen's texture to render to.
        let output_texture = gpu.surface.get_current_texture()?;

        let view = output_texture.texture.create_view(&Default::default());
        let depth_view = &self.depth_texture.view;

        // Render spacefill representation.
        if self.settings.render_spacefill {
            self.spacefill_pass
                .render(&view, depth_view, &mut encoder, &self.camera_resource);
        }

        // Render Ses surface using raymarching.
        if self.settings.render_ses {
            self.raymarch_pass.render(
                &view,
                depth_view,
                &mut encoder,
                &self.camera_resource,
                &gpu.shared_resources,
            );
        }
        // Render GUI
        self.gui_pass.render(&view, &mut encoder, gpu, gui)?;

        // Submit commands to the GPU
        gpu.queue.submit(Some(encoder.finish()));

        // Draw a frame
        output_texture.present();

        Ok(())
    }
}
