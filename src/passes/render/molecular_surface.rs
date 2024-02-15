use super::util;
use crate::passes::resources::{
    camera::resource::CameraResource, grid::ses_grid::SesGridResource, light::LightResource,
    textures::df_texture::DistanceFieldTextureRender, GpuResource,
};

const WGPU_LABEL: &str = "Render Molecular Surface";

pub struct RenderMolecularSurfaceResources {
    pub ses_grid: SesGridResource,              // @group(0)
    pub df_texture: DistanceFieldTextureRender, // @group(1)
    pub camera: CameraResource,                 // @group(2)
    pub light: LightResource,                   // @group(3)
}

pub struct RenderMolecularSurfacePass {
    resources: RenderMolecularSurfaceResources,
    render_pipeline: wgpu::RenderPipeline,
}

impl RenderMolecularSurfacePass {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        resources: RenderMolecularSurfaceResources,
    ) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/ses_raymarching.wgsl");

        let bind_group_layouts = &[
            resources.ses_grid.bind_group_layout(),
            resources.df_texture.bind_group_layout(),
            resources.camera.bind_group_layout(),
            resources.light.bind_group_layout(),
        ];

        let render_pipeline: wgpu::RenderPipeline =
            util::create_render_pipeline(WGPU_LABEL, device, config, shader, bind_group_layouts);

        Self {
            resources,
            render_pipeline,
        }
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: wgpu::Color,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(WGPU_LABEL),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, self.resources.ses_grid.bind_group(), &[]);
        render_pass.set_bind_group(1, self.resources.df_texture.bind_group(), &[]);
        render_pass.set_bind_group(2, self.resources.camera.bind_group(), &[]);
        render_pass.set_bind_group(3, self.resources.light.bind_group(), &[]);

        let number_of_vertices: u32 = 6; // Render a full screen quad

        render_pass.draw(0..number_of_vertices, 0..1);
    }
}
