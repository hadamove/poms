use super::resources::{PassId, ResourceRepo};
use crate::context::Context;
use compute_pass::ComputePass;

mod compute_pass;

pub struct ComputeJobs {
    passes: Vec<ComputePass>,
}

impl ComputeJobs {
    pub fn new(context: &Context, resources: &ResourceRepo) -> Self {
        let passes = vec![
            ComputePass::new(context, resources, PassId::ComputeProbe),
            ComputePass::new(context, resources, PassId::ComputeDistanceFieldRefinement),
            // More passes can be added here.
        ];

        Self { passes }
    }

    pub fn execute_passes(&mut self, resources: &ResourceRepo, encoder: &mut wgpu::CommandEncoder) {
        for pass in &mut self.passes {
            pass.execute(encoder, resources);
        }
    }
}
