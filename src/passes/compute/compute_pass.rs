use super::super::resources::{GroupIndex, ResourceRepo};
use super::PassId;
use crate::context::Context;

pub struct ComputePass {
    id: PassId,
    compute_pipeline: wgpu::ComputePipeline,
}

impl ComputePass {
    pub fn new(context: &Context, resources: &ResourceRepo, pass_id: PassId) -> Self {
        let resources = resources.get_resources(&pass_id);
        let shader = ResourceRepo::get_shader(&pass_id);

        let compute_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{:?} Compute Pipeline Layout", pass_id)),
                    bind_group_layouts: &resources.get_bind_group_layouts(),
                    ..Default::default()
                });

        let shader_module = context.device.create_shader_module(shader);

        let compute_pipeline =
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(&format!("{:?} Compute Pipeline", pass_id)),
                    layout: Some(&compute_pipeline_layout),
                    module: &shader_module,
                    entry_point: "main",
                });

        Self {
            id: pass_id,
            compute_pipeline,
        }
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, resources: &ResourceRepo) {
        let pass_resources = resources.get_resources(&self.id);

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);

        for (GroupIndex(index), bind_group) in pass_resources.get_bind_groups() {
            compute_pass.set_bind_group(index, bind_group, &[]);
        }

        let num_grid_points = resources.get_num_grid_points();
        let num_work_groups = f32::ceil(num_grid_points as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
