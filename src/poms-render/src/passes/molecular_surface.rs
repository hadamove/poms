use crate::RenderResources;

/// Wrapper around `wgpu::RenderPipeline` that is used to render the molecular surface representation of the molecule.
pub struct MolecularSurfacePass {
    render_pipeline: wgpu::RenderPipeline,
}

const WGPU_LABEL: &str = "Render Molecular Surface Pass";

impl MolecularSurfacePass {
    /// Creates a new instance of `MolecularSurfacePass` using the provided resources.
    /// The surface is rendered using raymarching and the signed distance field.
    pub fn new(device: &wgpu::Device, render_resources: &RenderResources) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/molecular_surface.wgsl");

        let bind_group_layouts = &[
            &render_resources.camera.bind_group_layout,
            &render_resources.light.bind_group_layout,
            &render_resources.distance_field.bind_group_layout,
        ];

        let render_pipeline: wgpu::RenderPipeline =
            super::create_render_pipeline(WGPU_LABEL, device, shader, bind_group_layouts);

        Self { render_pipeline }
    }

    /// Records the created render pass to the provided `encoder`.
    /// Call this every frame to render the molecular surface.
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, render_resources: &RenderResources) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(WGPU_LABEL),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &render_resources.color_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(render_resources.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &render_resources.normal_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &render_resources.depth_texture.view,
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
        render_pass.set_bind_group(0, &render_resources.camera.bind_group, &[]);
        render_pass.set_bind_group(1, &render_resources.light.bind_group, &[]);
        render_pass.set_bind_group(2, &render_resources.distance_field.bind_group, &[]);

        // Render a full screen quad used for raymarching.
        let number_of_vertices: u32 = 6;

        render_pass.draw(0..number_of_vertices, 0..1);
    }
}
