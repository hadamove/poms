use crate::gpu::GpuState;
use crate::shared::resources::{GlobalResources, GroupIndex};

pub struct SpacefillPass {
    render_pipeline: wgpu::RenderPipeline,
}

impl SpacefillPass {
    const VERTICES_PER_ATOM: u32 = 6;

    pub fn new(gpu: &GpuState) -> Self {
        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Spacefill Render Pipeline Layout"),
                    bind_group_layouts: &gpu.global_resources.spacefill_pass_bind_group_layouts(),
                    ..Default::default()
                });

        let shader = wgpu::include_wgsl!("../shaders/spacefill.wgsl");
        let shader_module = gpu.device.create_shader_module(shader);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Spacefill Render Pipeline"),
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
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                // TODO: try to replace with default
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
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
            label: Some("Render Pass"),
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

        let bind_groups = global_resources.spacefill_pass_bind_groups();
        for (GroupIndex(index), bind_group) in bind_groups {
            render_pass.set_bind_group(index, bind_group, &[]);
        }

        let vertex_count = global_resources.get_num_atoms() * Self::VERTICES_PER_ATOM;
        render_pass.draw(0..vertex_count, 0..1);
    }
}
