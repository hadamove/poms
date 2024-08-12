mod passes;
mod resources;
mod state;

use poms_common::models::atom::Atom;
use poms_common::models::grid::{create_compute_grid_around_molecule, GridUniform};
use poms_common::resources::CommonResources;

use passes::probe::{ProbePass, ProbeResources};
use passes::refinement::{RefinementPass, RefinementResources};
use resources::distance_field::DistanceFieldCompute;
use resources::grid_points::GridPointsResource;
use state::{ComputePhase, ComputeState};

/// Contains all resources that are owned by the compute pipeline.
pub struct ComputeOwnedResources {
    pub distance_field: DistanceFieldCompute,
    pub df_grid_points: GridPointsResource,
}

/// Manages the computation of a molecular surface using a grid-based approach.
/// For further details, see the `ComputeState` struct.
pub struct ComputeJobs {
    state: ComputeState,
    resources: ComputeOwnedResources,

    probe_pass: ProbePass,
    refinement_pass: RefinementPass,

    /// Stores the last computed distance field texture for potential use in rendering.
    last_computed_texture: Option<wgpu::Texture>,
}

/// Things required to create a new instance of `ComputeJobs`.
pub struct ComputeParameters<'a> {
    /// Reference to the molecule to compute the surface of. Required to create the initial grid.
    pub molecule: &'a [Atom],
    /// Resources shared between the compute and render pipelines. Contains molecule data on the GPU.
    pub common_resources: &'a CommonResources,
    /// Initial resolution of the molecular surface. At the start the computation will be performed on a grid of this resolution.
    /// Gradually, the resolution will be increased to the `target_resolution`, which takes more time but produces a more accurate surface.
    pub init_resolution: u32,
    /// Target resolution of the molecular surface.
    pub target_resolution: u32,
    /// Radius of the probe used to compute the molecular surface.
    pub probe_radius: f32,
}

impl ComputeJobs {
    /// Creates a new instance of `ComputeJobs` with the given parameters.
    pub fn new(device: &wgpu::Device, params: ComputeParameters) -> Self {
        // Create a grid around the molecule with the initial resolution.
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

    /// Executes the current phase of the compute pipeline.
    ///
    /// This method processes the molecular surface in the current `ComputePhase` (either `Probe` or `Refinement`).
    /// Once the grid points for the current frame are processed, it checks if the phase is complete and, if so,
    /// transitions to the next phase or increases the grid resolution.
    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        common: &CommonResources,
    ) {
        // Determine the number of grid points to process in this frame.
        let grid_points_count = self.state.grid_points_this_frame_count();

        // Execute the appropriate compute pass based on the current phase.
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

        // Update the count of computed grid points.
        self.state.grid_points_computed_count += grid_points_count;

        // Check if the current phase is finished and transition if necessary.
        if self.state.is_phase_finished() {
            let next_phase = self.state.next_phase();

            if let ComputePhase::Refinement = self.state.current_phase {
                // If refinement is complete, double the grid resolution.
                self.state.double_resolution();
                self.swap_out_df_texture(device);
            }
            self.state.current_phase = next_phase;
            self.state.grid_points_computed_count = 0;
        }
    }

    /// Ensure that the buffers representing the grid points and distance field are synchronized
    /// with the latest state of the computation. Call this function before executing the compute passes.
    pub fn update_buffers(&mut self, queue: &wgpu::Queue) {
        self.resources.distance_field.update(queue, self.state.grid);
        self.resources
            .df_grid_points
            .update(queue, self.state.grid_points_index_offset());
    }

    /// Returns the last computed distance field texture and its associated grid.
    /// The associated grid is also returned to ensure that the grid resolution matches the texture resolution.
    pub fn last_computed_distance_field(&mut self) -> Option<(wgpu::Texture, GridUniform)> {
        let texture = self.last_computed_texture.take()?;
        let mut grid = self.state.grid;

        // Resize the grid to match the resolution of the texture.
        grid.change_resolution(texture.depth_or_array_layers());

        Some((texture, grid))
    }

    /// Called after a refinement phase to swap the texture with upscaled resolution
    /// while keeping the old texture for potential rendering
    fn swap_out_df_texture(&mut self, device: &wgpu::Device) {
        let new_df_texture = DistanceFieldCompute::new(device, self.state.grid);

        let old_df_texture = std::mem::replace(&mut self.resources.distance_field, new_df_texture);

        self.last_computed_texture = Some(old_df_texture.texture)
    }
}
