use super::resources::ses_state::{ComputePhase, SesStage};
use super::resources::{PassId, ResourceRepo};
use crate::context::Context;
use compute_pass::ComputePass;

mod compute_pass;

pub struct ComputeJobs {
    probe_pass: ComputePass,
    refinement_pass: ComputePass,
}

impl ComputeJobs {
    pub fn new(context: &Context, resources: &ResourceRepo) -> Self {
        Self {
            probe_pass: ComputePass::new(context, resources, PassId::ComputeProbe),
            refinement_pass: ComputePass::new(context, resources, PassId::ComputeRefinement),
        }
    }

    pub fn execute_passes(&mut self, resources: &ResourceRepo, encoder: &mut wgpu::CommandEncoder) {
        if let SesStage::Compute(stage) = resources.get_ses_stage() {
            match stage.phase {
                ComputePhase::Probe => self.probe_pass.execute(encoder, resources),
                ComputePhase::Refinement(_) => self.refinement_pass.execute(encoder, resources),
            }
        }
    }
}
