use self::probe::{ComputeProbePass, ComputeProbeResources};
use self::refinement::{ComputeRefinementPass, ComputeRefinementResources};

use super::resources::CommonResources;
use crate::context::Context;

mod probe;
mod refinement;
mod util;

pub struct ComputeJobs {
    pub progress: ComputeProgress,
    probe_pass: ComputeProbePass,
    refinement_pass: ComputeRefinementPass,
}

#[derive(Clone, Debug)]
enum ComputePhase {
    Probe,
    Refinement,
    Finished,
}

#[derive(Clone, Debug)]
pub struct ComputeProgress {
    pub current_resolution: u32,
    pub last_computed_resolution: Option<u32>,
    target_resolution: u32,

    current_phase: ComputePhase,
    grid_points_computed_count: u32,

    probe_radius: f32,
    grid_size: f32,
}

/// TODO: Add docs
impl ComputeProgress {
    const POINTS_PER_FRAME: u32 = u32::pow(128, 3);

    pub fn new(target_resolution: u32, grid_size: f32, probe_radius: f32) -> Self {
        Self {
            current_resolution: 64, // TODO: make it a constant
            target_resolution,
            last_computed_resolution: None,

            current_phase: ComputePhase::Probe,
            grid_points_computed_count: 0,

            probe_radius,
            grid_size,
        }
    }

    pub fn advance(&mut self) {
        let points_this_frame_count = self.grid_points_this_frame_count();
        self.grid_points_computed_count += points_this_frame_count;

        if self.is_phase_finished() {
            match self.current_phase {
                ComputePhase::Probe => {
                    self.current_phase = ComputePhase::Refinement;
                }
                ComputePhase::Refinement => {
                    self.last_computed_resolution = Some(self.current_resolution);

                    if self.current_resolution == self.target_resolution {
                        self.current_phase = ComputePhase::Finished;
                    } else {
                        // Start from the beginning with a higher resolution.
                        self.current_phase = ComputePhase::Probe;
                        self.current_resolution = self.next_resolution();
                    }
                }
                ComputePhase::Finished => {
                    // `advance` should not be called when the phase is finished.
                    unreachable!()
                }
            }
            self.grid_points_computed_count = 0;
        }
    }

    pub fn grid_points_this_frame_count(&self) -> u32 {
        let total_grid_points_count = self.grid_points_count_of_current_resolution();
        u32::min(
            total_grid_points_count - self.grid_points_computed_count % total_grid_points_count,
            Self::POINTS_PER_FRAME,
        )
    }

    pub fn grid_points_index_offset(&self) -> u32 {
        self.grid_points_computed_count % self.grid_points_count_of_current_resolution()
    }

    fn is_phase_finished(&self) -> bool {
        let total_count = self.grid_points_count_of_current_resolution();
        match self.current_phase {
            ComputePhase::Probe => self.grid_points_computed_count == total_count,
            ComputePhase::Refinement => {
                let current_cycle_index = self.grid_points_computed_count / total_count;
                let is_last_cycle = current_cycle_index == self.refinement_cycles_count();
                is_last_cycle && self.grid_points_computed_count % total_count == 0
            }
            ComputePhase::Finished => true,
        }
    }

    fn refinement_cycles_count(&self) -> u32 {
        let grid_offset = self.grid_size / self.current_resolution as f32;
        (self.probe_radius / grid_offset) as u32
    }

    fn grid_points_count_of_current_resolution(&self) -> u32 {
        self.current_resolution.pow(3)
    }

    fn next_resolution(&self) -> u32 {
        u32::min(self.current_resolution * 2, self.target_resolution)
    }
}

impl ComputeJobs {
    pub fn new(device: &wgpu::Device, resources: &CommonResources) -> Self {
        Self {
            probe_pass: ComputeProbePass::new(
                &device,
                ComputeProbeResources {
                    ses_grid: resources.ses_resource.clone(),
                    molecule: resources.molecule_resource.clone(),
                },
            ),
            refinement_pass: ComputeRefinementPass::new(
                &device,
                ComputeRefinementResources {
                    ses_grid: resources.ses_resource.clone(),
                    molecule: resources.molecule_resource.clone(),
                    df_texture: resources.df_texture_back.compute.clone(),
                },
            ),
            progress: ComputeProgress::new(
                256,  // TODO: get this from outside
                70.0, // TODO: get this from outside
                1.4,  // TODO: get this from outside
            ),
        }
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let grid_points_count = self.progress.grid_points_this_frame_count();

        match self.progress.current_phase {
            ComputePhase::Probe => self.probe_pass.execute(encoder, grid_points_count),
            ComputePhase::Refinement => self.refinement_pass.execute(encoder, grid_points_count),
            ComputePhase::Finished => {
                // Do not advance progress if we are finished.
                return;
            }
        };

        self.progress.advance();
    }

    /// Keep the compute progress but recreate the passes with updated resources (e.g. after switching distance field textures).
    /// Note: if the molecule or probe radius has changed, we have to recreate the whole struct with [`ComputeJobs::new`] and start from the beginning instead.
    pub fn recreate_passes(&mut self, context: &Context, resources: &CommonResources) {
        let progress = self.progress.clone();
        *self = Self::new(context, resources);
        self.progress = progress;
    }
}
