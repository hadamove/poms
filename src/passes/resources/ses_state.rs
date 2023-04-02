use crate::utils::constants::{DEFAULT_PROBE_RADIUS, DEFAULT_SES_RESOLUTION};

#[derive(Debug)]
pub struct SesState {
    pub probe_radius: f32,
    pub max_resolution: u32,
    pub stage: SesStage,

    pub should_clear_predecessor: bool,
}

#[derive(Debug)]
pub enum SesStage {
    Init,
    Compute(ComputeStage),
    SwitchReady(u32),
    Done,
}

#[derive(Debug)]
pub struct ComputeStage {
    pub phase: ComputePhase,

    pub resolution: u32,
    pub previous_resolution: u32,

    pub num_grid_points: u32,
    pub grid_points_computed: u32,
}

impl ComputeStage {
    const PROBE_GRID_POINTS_PER_DISPATCH: u32 = u32::pow(128, 3);
    const REFINEMENT_GRID_POINTS_PER_DISPATCH: u32 = u32::pow(128, 3);

    pub fn new(previous_resolution: u32, resolution: u32) -> Self {
        Self {
            phase: ComputePhase::Probe,
            resolution,
            previous_resolution,
            grid_points_computed: 0,
            num_grid_points: Self::PROBE_GRID_POINTS_PER_DISPATCH,
        }
    }
}

#[derive(Debug)]
pub enum ComputePhase {
    Probe,
    Refinement(u64),
}

impl Default for SesState {
    fn default() -> Self {
        Self {
            probe_radius: DEFAULT_PROBE_RADIUS,
            max_resolution: DEFAULT_SES_RESOLUTION,
            stage: SesStage::Init,
            should_clear_predecessor: false
        }
    }
}

impl SesState {
    pub fn increase_frame(&mut self, offset: f32) {
        match self.stage {
            SesStage::Init => {
                self.stage = SesStage::Compute(ComputeStage::new(0, DEFAULT_SES_RESOLUTION));
            }
            SesStage::Compute(ref mut stage) => {
                let remaining_grid_points =
                    u32::max(stage.resolution.pow(3) - stage.grid_points_computed, 0);

                stage.num_grid_points = u32::min(stage.num_grid_points, remaining_grid_points);
                stage.grid_points_computed += stage.num_grid_points;

                if stage.grid_points_computed >= stage.resolution.pow(3) {
                    match &mut stage.phase {
                        ComputePhase::Probe => {
                            // Probe stage is finished, start refinement stage.
                            stage.phase = ComputePhase::Refinement(0);
                            stage.grid_points_computed = 0;
                            stage.num_grid_points =
                                ComputeStage::REFINEMENT_GRID_POINTS_PER_DISPATCH;
                            self.should_clear_predecessor = true;
                        }
                        ComputePhase::Refinement(index) => {
                            // We are done. Ready to switch textures.
                            if *index == (self.probe_radius / offset) as u64 {
                                self.stage = SesStage::SwitchReady(stage.resolution);
                            } else {
                                // We are not done with refinement, start next refinement stage.
                                stage.grid_points_computed = 0;
                                stage.phase = ComputePhase::Refinement(*index + 1);
                            }
                        }
                    }
                }
            }
            SesStage::SwitchReady(resolution) if resolution == self.max_resolution => {
                // Highest resolution is done, we are done.
                self.stage = SesStage::Done;
            }
            SesStage::SwitchReady(resolution) => {
                // Textures are switched, start next resolution.
                let next_resolution = u32::min(resolution * 2, self.max_resolution);
                self.stage = SesStage::Compute(ComputeStage::new(resolution, next_resolution));
            }
            SesStage::Done => {}
        };
    }

    pub fn reset_stage(&mut self) {
        self.stage = SesStage::Init;
    }

    pub fn switch_ready(&self) -> bool {
        matches!(self.stage, SesStage::SwitchReady(_))
    }

    pub fn get_grid_points_count(&self) -> u32 {
        match &self.stage {
            SesStage::Compute(stage) => stage.num_grid_points,
            _ => 0,
        }
    }

    pub fn get_grid_point_index_offset(&self) -> u32 {
        match &self.stage {
            SesStage::Compute(stage) => stage.grid_points_computed,
            _ => 0,
        }
    }

    pub fn get_compute_resolution(&self) -> u32 {
        match self.stage {
            SesStage::Compute(ref stage) => stage.resolution,
            SesStage::SwitchReady(resolution) => u32::min(resolution * 2, self.max_resolution),
            _ => 0,
        }
    }

    pub fn get_render_resolution(&self) -> u32 {
        match self.stage {
            SesStage::Init => 0,
            SesStage::Compute(ref stage) => stage.previous_resolution,
            SesStage::SwitchReady(resolution) => resolution,
            SesStage::Done => self.max_resolution,
        }
    }
}
