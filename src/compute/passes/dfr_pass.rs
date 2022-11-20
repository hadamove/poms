use crate::gpu::GpuState;
use crate::shared::resources::{GlobalResources, GroupIndex};

pub struct DistanceFieldRefinementPass {
    compute_pipeline: wgpu::ComputePipeline,
}

impl DistanceFieldRefinementPass {
    pub fn new(gpu: &GpuState) -> Self {
        let compute_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("DFR Compute Pipeline Layout"),
                    bind_group_layouts: &gpu.global_resources.dfr_pass_bind_group_layouts(),
                    ..Default::default()
                });

        let shader = wgpu::include_wgsl!("../shaders/dfr.wgsl");
        let shader_module = gpu.device.create_shader_module(shader);

        let compute_pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("DFR Compute Pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &shader_module,
                    entry_point: "main",
                });

        Self { compute_pipeline }
    }

    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        global_resources: &GlobalResources,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);

        let bind_group = global_resources.dfr_pass_bind_groups();

        for (GroupIndex(index), bind_group) in bind_group {
            compute_pass.set_bind_group(index, bind_group, &[]);
        }

        let num_grid_points = global_resources.get_num_grid_points();
        let num_work_groups = f32::ceil(num_grid_points as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
