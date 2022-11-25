use wgpu::ShaderModuleDescriptor;

use crate::compute::PassId;
use crate::gpu::GpuState;
use crate::shared::resources::{GlobalResources, GroupIndex};

const VERTICES_PER_ATOM: u32 = 6;
const FULLSCREEN_VERTICES: u32 = 6;

pub struct RenderPass {
    id: PassId,
    render_pipeline: wgpu::RenderPipeline,
    enabled: bool,
}

impl<'a> RenderPass {
    pub fn new(gpu: &GpuState, id: PassId, shader: ShaderModuleDescriptor<'a>) -> Self {
        let shader_module = gpu.device.create_shader_module(shader);
        let resources = gpu.global_resources.get_resources(&id);

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{:?} Render Pipeline Layout", id)),
                    bind_group_layouts: &resources.get_bind_group_layouts(),
                    ..Default::default()
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{:?} Render Pipeline", id)),
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
                        format: gpu.config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
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

    pub fn get_id(&self) -> &PassId {
        &self.id
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        global_resources: &GlobalResources,
    ) {
        if !self.enabled {
            return;
        }

        let resources = global_resources.get_resources(&self.id);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("{:?} Render Pass", self.id)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);

        for (GroupIndex(index), bind_group) in resources.get_bind_groups() {
            render_pass.set_bind_group(index, bind_group, &[]);
        }

        let num_vertices = self.get_num_vertices(global_resources);
        render_pass.draw(0..num_vertices, 0..1);
    }

    fn get_num_vertices(&self, global_resources: &GlobalResources) -> u32 {
        match self.id {
            PassId::RaymarchPass => FULLSCREEN_VERTICES,
            PassId::SpacefillPass => global_resources.get_num_atoms() * VERTICES_PER_ATOM,
            _ => unreachable!(),
        }
    }
}
