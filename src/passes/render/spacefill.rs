use super::util;
use crate::passes::resources::grid::molecule_grid::MoleculeGridResource;
use crate::passes::resources::{camera::resource::CameraResource, GpuResource};

const WGPU_LABEL: &str = "Render Spacefill";

pub struct RenderSpacefillResources {
    pub molecule: MoleculeGridResource, // @group(0)
    pub camera: CameraResource,         // @group(1)
}

pub struct RenderSpacefillPass {
    resources: RenderSpacefillResources,
    render_pipeline: wgpu::RenderPipeline,
}

impl RenderSpacefillPass {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        resources: RenderSpacefillResources,
    ) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/spacefill.wgsl");

        let bind_group_layouts = &[
            resources.molecule.bind_group_layout(),
            resources.camera.bind_group_layout(),
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
        render_pass.set_bind_group(0, self.resources.molecule.bind_group(), &[]);
        render_pass.set_bind_group(1, self.resources.camera.bind_group(), &[]);

        let num_atoms = 10; // TODO: FIX THIS
        let vertices_per_atom: u32 = 6; // Draw a quad (sphere impostor) for each atom
        let number_of_vertices: u32 = num_atoms * vertices_per_atom;

        render_pass.draw(0..number_of_vertices, 0..1);
    }
}
