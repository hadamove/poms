use self::probe::{ComputeProbePass, ComputeProbeResources};
use self::refinement::{ComputeRefinementPass, ComputeRefinementResources};

use super::resources::ses_state::{ComputePhase, SesStage};
use super::resources::ResourceRepo;
use crate::context::Context;

mod probe;
mod refinement;
mod util;

pub struct ComputeJobs {
    probe_pass: ComputeProbePass,
    refinement_pass: ComputeRefinementPass,
}

impl ComputeJobs {
    pub fn new(context: &Context, resources: &ResourceRepo) -> Self {
        Self {
            probe_pass: ComputeProbePass::new(
                &context.device,
                ComputeProbeResources {
                    ses_grid: resources.ses_resource.clone(),
                    molecule: resources.molecule_resource.clone(),
                },
            ),
            refinement_pass: ComputeRefinementPass::new(
                &context.device,
                ComputeRefinementResources {
                    ses_grid: resources.ses_resource.clone(),
                    molecule: resources.molecule_resource.clone(),
                    df_texture: resources.df_texture_back.compute.clone(),
                },
            ),
        }
    }

    pub fn execute(&mut self, resources: &ResourceRepo, encoder: &mut wgpu::CommandEncoder) {
        if let SesStage::Compute(stage) = resources.get_ses_stage() {
            match stage.phase {
                ComputePhase::Probe => self.probe_pass.execute(encoder, resources),
                ComputePhase::Refinement(_) => self.refinement_pass.execute(encoder, resources),
            }
        }
    }
}
