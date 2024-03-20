use super::passes::probe::{ProbePass, ProbeResources};
use super::passes::refinement::{RefinementPass, RefinementResources};
use super::resources::df_grid::GridPointsResource;
use super::resources::df_texture::DistanceFieldTextureCompute;
use crate::common::resources::grid::GridResource;
use crate::{app::constants::MIN_DISTANCE_FIELD_RESOLUTION, common::resources::CommonResources};

#[derive(Clone, Debug)]
pub enum ComputePhase {
    Probe,
    Refinement,
    Finished,
}

// TODO: Move this to a separate submodule
#[derive(Clone, Debug)]
pub struct ComputeProgress {
    pub current_resolution: u32,
    pub last_computed_resolution: Option<u32>,
    pub target_resolution: u32,

    pub current_phase: ComputePhase,
    pub grid_points_computed_count: u32,

    pub probe_radius: f32,
    pub grid_size: f32,
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

pub struct ComputeOwnedResources {
    pub df_grid: GridResource,
    pub df_texture: DistanceFieldTextureCompute,
    pub mixed_stuff: GridPointsResource, // TODO: Rename this
}

/// TODO: Add docs!!!
pub struct ComputeJobs {
    pub progress: ComputeProgress,
    pub resources: ComputeOwnedResources,
    probe_pass: ProbePass,
    refinement_pass: RefinementPass,
}

impl ComputeJobs {
    pub fn new(device: &wgpu::Device, common: &CommonResources) -> Self {
        let resources = ComputeOwnedResources {
            df_grid: GridResource::new(device),
            df_texture: DistanceFieldTextureCompute::new_with_resolution(
                device,
                MIN_DISTANCE_FIELD_RESOLUTION,
            ),
            mixed_stuff: GridPointsResource::new(device),
        };

        let probe_resources = ProbeResources::new(&resources, common);
        let refinement_resources = RefinementResources::new(&resources, common);

        Self {
            probe_pass: ProbePass::new(device, probe_resources),
            refinement_pass: RefinementPass::new(device, refinement_resources),
            progress: ComputeProgress::new(
                256,  // TODO: get this from outside
                70.0, // TODO: get this from outside
                1.4,  // TODO: get this from outside
            ),
            resources,
        }
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, common: &CommonResources) {
        let grid_points_count = self.progress.grid_points_this_frame_count();

        match self.progress.current_phase {
            ComputePhase::Probe => {
                let probe_resources = ProbeResources::new(&self.resources, common);
                self.probe_pass
                    .execute(encoder, grid_points_count, probe_resources);
            }
            ComputePhase::Refinement => {
                let refinement_resources = RefinementResources::new(&self.resources, common);
                self.refinement_pass
                    .execute(encoder, grid_points_count, refinement_resources);
            }
            ComputePhase::Finished => {
                // Do not advance progress if we are finished.
                return;
            }
        };

        self.progress.advance();
    }
}
