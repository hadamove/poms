use super::camera::CameraRender;
use crate::parser::Molecule;
use crate::texture;
use wgpu::util::DeviceExt;

const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.03,
    g: 0.03,
    b: 0.04,
    a: 1.00,
};

pub struct AtomRenderPass {
    pub render_pipeline: wgpu::RenderPipeline,
    pub atoms_bind_group: wgpu::BindGroup,
    pub depth_texture: texture::Texture,
    pub vertex_count: u32,
}

impl AtomRenderPass {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_render: &CameraRender,
        molecule: &Molecule,
    ) -> Self {
        let atoms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atoms Buffer"),
            contents: bytemuck::cast_slice(&molecule.atoms),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let atoms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Atoms Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let atoms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atoms Bind Group"),
            layout: &atoms_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 1,
                resource: atoms_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    camera_render.get_bind_group_layout(),
                    &atoms_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(&wgpu::include_wgsl!("shaders/atom.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let depth_texture = texture::Texture::create_depth_texture(device, config);
        let vertex_count = molecule.atoms.len() as u32 * 6;

        Self {
            atoms_bind_group,
            render_pipeline,
            depth_texture,
            vertex_count,
        }
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        camera_render: &CameraRender,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_render.get_bind_group(), &[]);
        render_pass.set_bind_group(1, &self.atoms_bind_group, &[]);
        render_pass.draw(0..self.vertex_count, 0..1);
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.depth_texture = texture::Texture::create_depth_texture(device, config)
    }
}
