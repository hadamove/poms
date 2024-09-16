use poms_common::models::grid::GridUniform;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComputePhase {
    Probe,
    Refinement,
    Finished,
}

/// The state of the compute pipeline for molecular surface generation.
/// This process is divided into two primary phases:
/// - **Probe Phase**: In this phase, each grid point is classified based on its distance to the nearest atom,
///   identifying whether it lies inside, outside, or on the boundary of the molecular surface.
/// - **Refinement Phase**: In this phase, the surface is refined by iteratively processing the boundary points
///   to achieve the desired resolution.
#[derive(Clone, Debug)]
pub struct ComputeState {
    pub grid: GridUniform,
    pub current_phase: ComputePhase,
    pub target_resolution: u32,
    pub grid_points_computed_count: u32,
}

/// Manages the state and transitions of the molecular surface computation process.
impl ComputeState {
    /// Maximum number of grid points to process per frame to maintain performance.
    const GRID_POINTS_PER_FRAME: u32 = u32::pow(128, 3);

    /// Initializes the compute state with the specified target resolution and grid.
    /// The initial phase is set to `Probe`.
    pub fn new(target_resolution: u32, grid: GridUniform) -> Self {
        Self {
            grid,
            target_resolution,
            current_phase: ComputePhase::Probe,
            grid_points_computed_count: 0,
        }
    }

    /// Determines the next phase of computation based on the current phase and grid resolution.
    /// - Transitions from `Probe` to `Refinement` after classifying all grid points.
    /// - Transitions from `Refinement` to `Finished` when the target resolution is achieved.
    pub fn next_phase(&self) -> ComputePhase {
        match self.current_phase {
            ComputePhase::Probe => ComputePhase::Refinement,
            ComputePhase::Refinement => {
                if self.grid.resolution == self.target_resolution {
                    ComputePhase::Finished
                } else {
                    ComputePhase::Probe
                }
            }
            ComputePhase::Finished => ComputePhase::Finished,
        }
    }

    /// Doubles the resolution of the grid, up to the target resolution.
    /// This function is typically used between refinement cycles to progressively enhance surface detail.
    pub fn double_resolution(&mut self) {
        let next_resolution = u32::min(self.grid.resolution * 2, self.target_resolution);
        self.grid.change_resolution(next_resolution);
    }

    /// Determines how many grid points should be processed in the current frame.
    /// This helps in managing the workload per frame to ensure smooth performance.
    pub fn grid_points_this_frame_count(&self) -> u32 {
        let total_grid_points_count = self.grid_points_count_of_current_resolution();
        u32::min(
            total_grid_points_count - self.grid_points_computed_count % total_grid_points_count,
            Self::GRID_POINTS_PER_FRAME,
        )
    }

    /// Computes the index offset for grid points in the current phase.
    /// This offset is used to continue processing from where the last frame left off.
    pub fn grid_points_index_offset(&self) -> u32 {
        self.grid_points_computed_count % self.grid_points_count_of_current_resolution()
    }

    /// Checks if the current phase is finished.
    /// - For `Probe`, it is finished when all grid points are classified.
    /// - For `Refinement`, it is finished when all refinement cycles are completed for the current resolution.
    pub fn is_phase_finished(&self) -> bool {
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

    /// Returns the total number of grid points for the current resolution.
    pub fn grid_points_count_of_current_resolution(&self) -> u32 {
        self.grid.resolution.pow(3)
    }

    /// Returns the progress of the current phase as a value between 0.0 and 1.0.
    /// If there is no computation in progress, returns `None`.
    /// Can be used to display progress information to the user.
    pub fn progress(&self) -> Option<ComputeProgress> {
        match self.current_phase {
            ComputePhase::Probe => match self.grid_points_computed_count {
                0 => None,
                _ => Some(ComputeProgress {
                    progress: 0.0,
                    current_resolution: self.grid.resolution,
                    target_resolution: self.target_resolution,
                }),
            },
            ComputePhase::Refinement => {
                let total_cycles = self.refinement_cycles_count();
                let current_cycle_index = self.grid_points_computed_count
                    / self.grid_points_count_of_current_resolution();

                Some(ComputeProgress {
                    progress: current_cycle_index as f32 / total_cycles as f32,
                    current_resolution: self.grid.resolution,
                    target_resolution: self.target_resolution,
                })
            }
            ComputePhase::Finished => None,
        }
    }

    /// Calculates the number of refinement cycles needed for the current grid resolution.
    /// Each cycle refines a single layer of grid points to improve surface accuracy.
    fn refinement_cycles_count(&self) -> u32 {
        // The number of cycles depends on the ratio of the probe radius to the grid spacing.
        (self.grid.probe_radius / self.grid.spacing) as u32 + 1
    }
}

/// Struct holding metadata about copute progress
pub struct ComputeProgress {
    pub progress: f32,
    pub current_resolution: u32,
    pub target_resolution: u32,
}
