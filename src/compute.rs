use super::gpu::GpuState;
use super::shared::resources::{GlobalResources, GroupIndex};

// TODO: move this somewhere else.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PassId {
    Probe,
    DistanceFieldRefinement,
    Spacefill,
    SesRaymarching,
}

pub struct ComputeJobs {
    passes: Vec<ComputePass>,
}

impl ComputeJobs {
    pub fn new(gpu: &GpuState, global_resources: &GlobalResources) -> Self {
        use PassId::*;

        let passes = vec![
            ComputePass::new(gpu, global_resources, Probe),
            ComputePass::new(gpu, global_resources, DistanceFieldRefinement),
            // More passes can be added here.
        ];

        Self { passes }
    }

    pub fn execute_passes(
        &mut self,
        global_resources: &GlobalResources,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        for pass in &mut self.passes {
            pass.execute(encoder, global_resources);
        }
    }
}

pub struct ComputePass {
    id: PassId,
    compute_pipeline: wgpu::ComputePipeline,
}

impl ComputePass {
    pub fn new(gpu: &GpuState, global_resources: &GlobalResources, pass_id: PassId) -> Self {
        let resources = global_resources.get_resources(&pass_id);
        let shader = GlobalResources::get_shader(&pass_id);

        let compute_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{:?} Compute Pipeline Layout", pass_id)),
                    bind_group_layouts: &resources.get_bind_group_layouts(),
                    ..Default::default()
                });

        let shader_module = gpu.device.create_shader_module(shader);

        let compute_pipeline =
            gpu.device
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

    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        global_resources: &GlobalResources,
    ) {
        let resources = global_resources.get_resources(&self.id);

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);

        for (GroupIndex(index), bind_group) in resources.get_bind_groups() {
            compute_pass.set_bind_group(index, bind_group, &[]);
        }

        let num_grid_points = global_resources.get_num_grid_points();
        let num_work_groups = f32::ceil(num_grid_points as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
