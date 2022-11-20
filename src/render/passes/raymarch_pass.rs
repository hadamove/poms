use crate::gpu::GpuState;
use crate::shared::resources::{GlobalResources, GroupIndex};

pub struct RaymarchDistanceFieldPass {
    render_pipeline: wgpu::RenderPipeline,
}

impl RaymarchDistanceFieldPass {
    pub fn new(gpu: &GpuState) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/raymarch.wgsl");
        let shader_module = gpu.device.create_shader_module(shader);

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Raymarch Distance Field Render Pipeline Layout"),
                    bind_group_layouts: &gpu.global_resources.raymarch_pass_bind_group_layouts(),
                    push_constant_ranges: &[],
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Raymarch Distance Field Render Pipeline"),
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

        Self { render_pipeline }
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        global_resources: &GlobalResources,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Raymarch Distance Field Render Pass"),
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

        let bind_groups = global_resources.raymarch_pass_bind_groups();
        for (GroupIndex(index), bind_group) in bind_groups {
            render_pass.set_bind_group(index, bind_group, &[]);
        }

        render_pass.draw(0..6, 0..1);
    }
}
