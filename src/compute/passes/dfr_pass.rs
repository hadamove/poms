use wgpu::util::DeviceExt;

use super::resources::bind_group::SharedBindGroupLayout;
use crate::compute::grid::SESGrid;

pub struct DistanceFieldRefinementPass {
    compute_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    pub num_grid_points: u32,
    pub df_buffer: wgpu::Buffer,
}

impl DistanceFieldRefinementPass {
    pub fn new(device: &wgpu::Device, ses_grid: &SESGrid) -> Self {
        let num_grid_points = ses_grid.get_num_grid_points();

        let df_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DF buffer"),
            contents: bytemuck::cast_slice(&vec![0f32; num_grid_points as usize]),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: df_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let shared_bind_group_layout = SharedBindGroupLayout::init(device);

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("DFR Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &shared_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_shader =
            device.create_shader_module(&wgpu::include_wgsl!("../shaders/dfr.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("DFR Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        Self {
            compute_pipeline,
            bind_group,
            num_grid_points,
            df_buffer,
        }
    }

    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        shared_bind_group: &wgpu::BindGroup,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.set_bind_group(1, shared_bind_group, &[]);

        let num_work_groups = f32::ceil(self.num_grid_points as f32 / 64.0) as u32;
        println!("Executing DFR pass {} work groups", num_work_groups);

        compute_pass.dispatch(num_work_groups, 1, 1);
    }
}
