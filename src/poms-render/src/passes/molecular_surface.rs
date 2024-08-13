use crate::resources::{
    camera::CameraResource, distance_field::DistanceFieldRender, light::LightResource,
};
use crate::RenderOwnedResources;

/// Contains resources that are required to render the molecular surface representation of the molecule.
/// Bind groups are sorted by the frequency of change as advised by `wgpu` documentation.
pub struct MolecularSurfaceResources<'a> {
    pub camera: &'a CameraResource,              // @group(0)
    pub light: &'a LightResource,                // @group(1)
    pub distance_field: &'a DistanceFieldRender, // @group(2)
}

impl<'a> MolecularSurfaceResources<'a> {
    /// Creates a new instance of `MolecularSurfaceResources`.
    /// It is okay and cheap to construct this each frame, as it only contains references to resources.
    pub fn new(resources: &'a RenderOwnedResources) -> Self {
        Self {
            camera: &resources.camera_resource,
            light: &resources.light_resource,
            distance_field: &resources.distance_field,
        }
    }
}

/// Wrapper around `wgpu::RenderPipeline` that is used to render the molecular surface representation of the molecule.
pub struct MolecularSurfacePass {
    render_pipeline: wgpu::RenderPipeline,
}

const WGPU_LABEL: &str = "Render Molecular Surface Pass";

impl MolecularSurfacePass {
    /// Creates a new instance of `MolecularSurfacePass` using the provided resources.
    /// The surface is rendered using raymarching and the signed distance field.
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        resources: MolecularSurfaceResources,
    ) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/molecular_surface.wgsl");

        let bind_group_layouts = &[
            &resources.camera.bind_group_layout,
            &resources.light.bind_group_layout,
            &resources.distance_field.bind_group_layout,
        ];

        let render_pipeline: wgpu::RenderPipeline =
            super::create_render_pipeline(WGPU_LABEL, device, config, shader, bind_group_layouts);

        Self { render_pipeline }
    }

    /// Records the created render pass to the provided `encoder`.
    /// Call this every frame to render the molecular surface.
    pub fn render(
        &self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: wgpu::Color,
        resources: MolecularSurfaceResources,
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
        render_pass.set_bind_group(0, &resources.camera.bind_group, &[]);
        render_pass.set_bind_group(1, &resources.light.bind_group, &[]);
        render_pass.set_bind_group(2, &resources.distance_field.bind_group, &[]);

        // Render a full screen quad used for raymarching.
        let number_of_vertices: u32 = 6;

        render_pass.draw(0..number_of_vertices, 0..1);
    }
}
