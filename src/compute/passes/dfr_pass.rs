use wgpu::util::DeviceExt;

use crate::compute::grid::SESGrid;

use super::resources::buffer::{SharedBuffers, MAX_NUM_GRID_POINTS};

pub struct DistanceFieldRefinementPass {
    bind_group: wgpu::BindGroup,
    distance_field_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,

    num_grid_points: u32,
}

impl DistanceFieldRefinementPass {
    pub fn new(device: &wgpu::Device, ses_grid: &SESGrid, shared_buffers: &SharedBuffers) -> Self {
        let distance_field_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Distance field buffer"),
            contents: bytemuck::cast_slice(&vec![0f32; MAX_NUM_GRID_POINTS]),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("DFR Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shared_buffers.ses_grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: shared_buffers
                        .grid_point_classification_buffer
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: distance_field_buffer.as_entire_binding(),
                },
            ],
            label: Some("DFR Bind Group"),
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("DFR Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/dfr.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("DFR Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        Self {
            bind_group,
            distance_field_buffer,
            compute_pipeline,
            num_grid_points: ses_grid.get_num_grid_points(),
        }
    }

    pub fn update_grid(&mut self, ses_grid: &SESGrid) {
        self.num_grid_points = ses_grid.get_num_grid_points();
    }

    pub fn get_distance_field_buffer(&self) -> &wgpu::Buffer {
        &self.distance_field_buffer
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        let num_work_groups = f32::ceil(self.num_grid_points as f32 / 64.0) as u32;
        println!("Executing DFR pass {} work groups", num_work_groups);

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
