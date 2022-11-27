use crate::context::Context;

use self::compute_pass::ComputePass;

use super::resources::GlobalResources;

mod compute_pass;

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
    pub fn new(context: &Context, resources: &GlobalResources) -> Self {
        use PassId::*;

        let passes = vec![
            ComputePass::new(context, resources, Probe),
            ComputePass::new(context, resources, DistanceFieldRefinement),
            // More passes can be added here.
        ];

        Self { passes }
    }

    pub fn execute_passes(
        &mut self,
        resources: &GlobalResources,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        for pass in &mut self.passes {
            pass.execute(encoder, resources);
        }
    }
}
