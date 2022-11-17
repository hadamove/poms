use super::resources::bind_group::DistanceFieldRefinementPassBindGroup;
use crate::{gpu::GpuState, shared::resources::SharedResources};

pub struct DistanceFieldRefinementPass {
    bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
}

impl DistanceFieldRefinementPass {
    pub fn new(gpu: &GpuState, grid_point_class_buffer: &wgpu::Buffer) -> Self {
        let bind_group_layout = gpu
            .device
            .create_bind_group_layout(&DistanceFieldRefinementPassBindGroup::LAYOUT_DESCRIPTOR);

        let bind_group = DistanceFieldRefinementPassBindGroup::create(
            &gpu.device,
            &bind_group_layout,
            grid_point_class_buffer,
        );

        let compute_pipeline_layout =
            &gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("DFR Compute Pipeline Layout"),
                    bind_group_layouts: &[
                        &bind_group_layout,
                        &gpu.shared_resources.bind_group_layout,
                        &gpu.shared_resources
                            .distance_field_texture
                            .bind_group_compute_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let compute_shader = gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/dfr.wgsl"));

        let compute_pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("DFR Compute Pipeline"),
                    layout: Some(compute_pipeline_layout),
                    module: &compute_shader,
                    entry_point: "main",
                });

        Self {
            bind_group,
            compute_pipeline,
        }
    }

    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        shared_resources: &SharedResources,
    ) {
        let distance_field_texture_bind_group =
            &shared_resources.distance_field_texture.bind_group_compute;

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.set_bind_group(1, &shared_resources.bind_group, &[]);
        compute_pass.set_bind_group(2, distance_field_texture_bind_group, &[]);

        let num_grid_points = shared_resources.ses_grid.get_num_grid_points();
        let num_work_groups = f32::ceil(num_grid_points as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
