use common::models::atom::Atom;
use common::models::grid::{create_compute_grid_around_molecule, GridUniform};
use common::resources::CommonResources;

use crate::passes::probe::{ProbePass, ProbeResources};
use crate::passes::refinement::{RefinementPass, RefinementResources};
use crate::resources::distance_field::DistanceFieldCompute;
use crate::resources::grid_points::GridPointsResource;

pub mod passes;
pub mod resources;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComputePhase {
    Probe,
    Refinement,
    Finished,
}

// TODO: Move this to a separate submodule
#[derive(Clone, Debug)]
pub struct ComputeState {
    pub grid: GridUniform,
    pub current_phase: ComputePhase,
    pub target_resolution: u32,
    pub grid_points_computed_count: u32,
}

/// TODO: Add docs
impl ComputeState {
    const GRID_POINTS_PER_FRAME: u32 = u32::pow(128, 3);

    pub fn new(target_resolution: u32, grid: GridUniform) -> Self {
        Self {
            grid,
            target_resolution,
            current_phase: ComputePhase::Probe,
            grid_points_computed_count: 0,
        }
    }

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

    fn double_resolution(&mut self) {
        let next_resolution = u32::min(self.grid.resolution * 2, self.target_resolution);
        self.grid.change_resolution(next_resolution);
    }

    fn grid_points_this_frame_count(&self) -> u32 {
        let total_grid_points_count = self.grid_points_count_of_current_resolution();
        u32::min(
            total_grid_points_count - self.grid_points_computed_count % total_grid_points_count,
            Self::GRID_POINTS_PER_FRAME,
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
        // In each refinement cycle, a single layer of grid points is processed
        // The maximum number of cycles is equivalent to number of layers within probe
        // TODO: better comment
        (self.grid.probe_radius / self.grid.spacing) as u32 + 1
    }

    fn grid_points_count_of_current_resolution(&self) -> u32 {
        self.grid.resolution.pow(3)
    }
}

pub struct ComputeOwnedResources {
    pub distance_field: DistanceFieldCompute,
    pub df_grid_points: GridPointsResource,
}

/// TODO: Add docs!!!
pub struct ComputeJobs {
    state: ComputeState,
    resources: ComputeOwnedResources,

    probe_pass: ProbePass,
    refinement_pass: RefinementPass,

    last_computed_texture: Option<wgpu::Texture>,
}

pub struct ComputeParameters<'a> {
    pub molecule: &'a [Atom],
    pub common_resources: &'a CommonResources,
    pub init_resolution: u32,
    pub target_resolution: u32,
    pub probe_radius: f32,
}

impl ComputeJobs {
    pub fn new(device: &wgpu::Device, params: ComputeParameters) -> Self {
        // Computation takes place in a grid around the molecule.
        // We start with a grid of size `init_resolution` and progressively increase it to `target_resolution`.
        let grid = create_compute_grid_around_molecule(
            params.molecule,
            params.init_resolution,
            params.probe_radius,
        );
        let resources = ComputeOwnedResources {
            distance_field: DistanceFieldCompute::new(device, grid),
            df_grid_points: GridPointsResource::new(device, params.target_resolution),
        };
        let probe_resources = ProbeResources::new(&resources, params.common_resources);
        let refinement_resources = RefinementResources::new(&resources);

        Self {
            probe_pass: ProbePass::new(device, probe_resources),
            refinement_pass: RefinementPass::new(device, refinement_resources),
            state: ComputeState::new(params.target_resolution, grid),
            resources,
            last_computed_texture: None,
        }
    }

    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        common: &CommonResources,
    ) {
        let grid_points_count = self.state.grid_points_this_frame_count();

        match self.state.current_phase {
            ComputePhase::Probe => {
                let probe_resources = ProbeResources::new(&self.resources, common);
                self.probe_pass
                    .execute(encoder, grid_points_count, probe_resources);
            }
            ComputePhase::Refinement => {
                let refinement_resources = RefinementResources::new(&self.resources);
                self.refinement_pass
                    .execute(encoder, grid_points_count, refinement_resources);
            }
            ComputePhase::Finished => {}
        };

        self.state.grid_points_computed_count += grid_points_count;
        if self.state.is_phase_finished() {
            let next_phase = self.state.next_phase();

            if let ComputePhase::Refinement = self.state.current_phase {
                // In case refinement was finished, we can swap the textures
                self.state.double_resolution();
                self.swap_out_df_texture(device);
            }
            self.state.current_phase = next_phase;
            self.state.grid_points_computed_count = 0;
        }
    }

    pub fn update_buffers(&mut self, queue: &wgpu::Queue) {
        self.resources.df_grid_points.update(queue, &self.state);
        self.resources.distance_field.update(queue, &self.state);
    }

    pub fn last_computed_distance_field(&mut self) -> Option<(wgpu::Texture, GridUniform)> {
        let texture = self.last_computed_texture.take()?;
        let mut grid = self.state.grid;

        // The grid has been upscaled after texture is computed, resize it back to texture resolution
        grid.change_resolution(texture.depth_or_array_layers());

        Some((texture, grid))
    }

    fn swap_out_df_texture(&mut self, device: &wgpu::Device) {
        let new_df_texture = DistanceFieldCompute::new(device, self.state.grid);

        let old_df_texture = std::mem::replace(&mut self.resources.distance_field, new_df_texture);

        self.last_computed_texture = Some(old_df_texture.texture)
    }
}
