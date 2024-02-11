use super::super::resources::{GroupIndex, ResourceRepo};
use super::PassId;
use crate::context::Context;

const VERTICES_PER_ATOM: u32 = 6;
const FULLSCREEN_VERTICES: u32 = 6;

pub struct RenderPass {
    id: PassId,
    render_pipeline: wgpu::RenderPipeline,
    enabled: bool,
}

impl RenderPass {
    pub fn new(context: &Context, resources: &ResourceRepo, id: PassId) -> Self {
        let shader = ResourceRepo::get_shader(&id);
        let shader_module = context.device.create_shader_module(shader);
        let resources = resources.get_resources(&id);

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{id:?} Render Pipeline Layout")),
                    bind_group_layouts: &resources.get_bind_group_layouts(),
                    ..Default::default()
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(&format!("{id:?} Render Pipeline")),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth24PlusStencil8,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        Self {
            id,
            render_pipeline,
            enabled: true,
        }
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        resources: &ResourceRepo,
        clear_color: wgpu::Color,
    ) {
        if !self.enabled {
            return;
        }

        let pass_resources = resources.get_resources(&self.id);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("{:?} Render Pass", self.id)),
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
                // stencil_ops: Some(wgpu::Operations {
                //     load: wgpu::LoadOp::Clear(0),
                //     store: true,
                // }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        for (GroupIndex(index), bind_group) in pass_resources.get_bind_groups() {
            render_pass.set_bind_group(index, bind_group, &[]);
        }

        let num_vertices = self.get_num_vertices(resources);
        render_pass.draw(0..num_vertices, 0..1);
    }

    pub fn get_id(&self) -> &PassId {
        &self.id
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_num_vertices(&self, resources: &ResourceRepo) -> u32 {
        match self.id {
            PassId::RenderSesRaymarching => FULLSCREEN_VERTICES,
            PassId::RenderSpacefill => resources.get_num_atoms() as u32 * VERTICES_PER_ATOM,
            _ => unreachable!(),
        }
    }
}
