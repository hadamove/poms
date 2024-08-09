use common::resources::{atoms_with_lookup::AtomsWithLookupResource, CommonResources};

use crate::{resources::camera::CameraResource, RenderOwnedResources};

use super::util;

/// Contains resources that are required to render the spacefill representation of the molecule.
pub struct SpacefillResources<'a> {
    pub molecule: &'a AtomsWithLookupResource, // @group(0)
    pub camera: &'a CameraResource,            // @group(1)
}

impl<'a> SpacefillResources<'a> {
    /// Creates a new instance of `SpacefillResources`.
    /// It is okay and cheap to construct this each frame, as it only contains references to resources.
    pub fn new(resources: &'a RenderOwnedResources, common: &'a CommonResources) -> Self {
        Self {
            molecule: &common.atoms_resource,
            camera: &resources.camera_resource,
        }
    }
}

/// Wrapper around `wgpu::RenderPipeline` that is used to render the spacefill representation of the molecule.
pub struct SpacefillPass {
    render_pipeline: wgpu::RenderPipeline,
}

const WGPU_LABEL: &str = "Render Spacefill";

impl SpacefillPass {
    /// Creates a new instance of `SpacefillPass` using the provided resources.
    /// The spacefill representation is rendered using sphere impostors.
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        resources: SpacefillResources,
    ) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/spacefill.wgsl");

        let bind_group_layouts = &[
            &resources.molecule.bind_group_layout,
            &resources.camera.bind_group_layout,
        ];

        let render_pipeline: wgpu::RenderPipeline =
            util::create_render_pipeline(WGPU_LABEL, device, config, shader, bind_group_layouts);

        Self { render_pipeline }
    }

    /// Records the created render pass to the provided `encoder`.
    /// Call this every frame to render the spacefill representation.
    pub fn render(
        &self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: wgpu::Color,
        resources: SpacefillResources,
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
        render_pass.set_bind_group(0, &resources.molecule.bind_group, &[]);
        render_pass.set_bind_group(1, &resources.camera.bind_group, &[]);

        let number_of_atoms: u32 = resources.molecule.number_of_atoms;
        // Each atom is drawn as a sphere impostor with 6 vertices.
        let vertices_per_atom: u32 = 6;
        let number_of_vertices: u32 = number_of_atoms * vertices_per_atom;

        render_pass.draw(0..number_of_vertices, 0..1);
    }
}
