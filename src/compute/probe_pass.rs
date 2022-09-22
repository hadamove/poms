use super::grid::{NeighborAtomGrid, SESGrid};
use crate::render::resources::camera::CameraResource;
use crate::utils::molecule::Molecule;

use super::bind_group::{ProbePassBindGroup, ProbePassBindGroupLayout};
use super::buffer::ProbePassBuffers;

pub struct ProbePass {
    ses_grid: SESGrid,
    bind_group: wgpu::BindGroup,

    grid_points_bind_group: wgpu::BindGroup,

    compute_pipeline: wgpu::ComputePipeline,
    // TODO: temp this will be removed
    render_pipeline: wgpu::RenderPipeline,
}

impl ProbePass {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_resource: &CameraResource,
        molecule: &Molecule,
    ) -> Self {
        let ses_grid = SESGrid::from_molecule(molecule);
        let neighbor_atom_grid = NeighborAtomGrid::from_molecule(molecule);

        let buffers = ProbePassBuffers::new(device, &ses_grid, &neighbor_atom_grid);
        let bind_group_layout = ProbePassBindGroupLayout::init(device);
        let bind_group = ProbePassBindGroup::init(device, &bind_group_layout, &buffers);

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_shader =
            device.create_shader_module(&wgpu::include_wgsl!("shaders/probe.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        // TODO: TEMP WILL BE REMOVED
        // --------------------
        let grid_points_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let grid_points_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &grid_points_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.grid_points_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let render_shader =
            device.create_shader_module(&wgpu::include_wgsl!("../render/shaders/atom.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    camera_resource.get_bind_group_layout(),
                    &grid_points_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline Probe"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
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
        // --------------------

        Self {
            ses_grid,
            bind_group,
            grid_points_bind_group,
            compute_pipeline,
            render_pipeline,
        }
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        let num_work_groups = f32::ceil(self.ses_grid.get_num_grid_points() as f32 / 64.0) as u32;

        compute_pass.dispatch(num_work_groups, 1, 1);
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        camera_resource: &CameraResource,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
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
        render_pass.set_bind_group(0, camera_resource.get_bind_group(), &[]);
        render_pass.set_bind_group(1, &self.grid_points_bind_group, &[]);
        render_pass.draw(0..self.ses_grid.get_num_grid_points() * 6, 0..1);
    }
}
