use crate::resources::{distance_field::DistanceFieldCompute, grid_points::GridPointsResource};
use crate::ComputeOwnedResources;

use poms_common::resources::{atoms_with_lookup::AtomsWithLookupResource, CommonResources};

use super::util;

/// Contains resources required to execute the probe pass.
pub struct ProbeResources<'a> {
    pub atoms: &'a AtomsWithLookupResource,       // @group(0)
    pub distance_field: &'a DistanceFieldCompute, // @group(1)
    pub df_grid_points: &'a GridPointsResource,   // @group(2)
}

impl<'a> ProbeResources<'a> {
    /// Creates a new instance of `ProbeResources`.
    /// It is okay and cheap to construct this each frame, as it only contains references to resources.
    pub fn new(resources: &'a ComputeOwnedResources, common: &'a CommonResources) -> Self {
        Self {
            df_grid_points: &resources.df_grid_points,
            distance_field: &resources.distance_field,
            atoms: &common.atoms_resource,
        }
    }
}

/// Wrapper around `wgpu::ComputePipeline` that is used to execute the probe step of the algorithm.
pub struct ProbePass {
    compute_pipeline: wgpu::ComputePipeline,
}

const WGPU_LABEL: &str = "Compute Probe";

impl ProbePass {
    /// Creates a new instance of `ProbePass` using the provided resources.
    /// The probe step is executed to classify each grid point (inside, outside, or on the boundary of the molecular surface).
    pub fn new(device: &wgpu::Device, resources: ProbeResources) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/probe.wgsl");

        let bind_group_layouts = &[
            &resources.atoms.bind_group_layout,
            &resources.distance_field.bind_group_layout,
            &resources.df_grid_points.bind_group_layout,
        ];

        let compute_pipeline =
            util::create_compute_pipeline(WGPU_LABEL, device, shader, bind_group_layouts);

        Self { compute_pipeline }
    }

    /// Records the created compute pass to the provided `encoder`.
    /// Call this every frame to execute the probe step on `grid_points_count` grid points.
    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        grid_points_count: u32,
        resources: ProbeResources,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &resources.atoms.bind_group, &[]);
        compute_pass.set_bind_group(1, &resources.distance_field.bind_group, &[]);
        compute_pass.set_bind_group(2, &resources.df_grid_points.bind_group, &[]);

        let work_groups_count = f32::ceil(grid_points_count as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(work_groups_count, 1, 1);
    }
}
