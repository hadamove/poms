use poms_common::resources::CommonResources;

use crate::ComputeResources;

/// Wrapper around `wgpu::ComputePipeline` that is used to execute the probe step of the algorithm.
pub struct ProbePass {
    compute_pipeline: wgpu::ComputePipeline,
}

const WGPU_LABEL: &str = "Compute Probe Pass";

impl ProbePass {
    /// Creates a new instance of `ProbePass` using the provided resources.
    /// The probe step is executed to classify each grid point (inside, outside, or on the boundary of the molecular surface).
    pub fn new(
        device: &wgpu::Device,
        compute_resources: &ComputeResources,
        common_resources: &CommonResources,
    ) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/probe.wgsl");

        let bind_group_layouts = &[
            &compute_resources.df_grid_points.bind_group_layout,
            &compute_resources.distance_field.bind_group_layout,
            &common_resources.atoms_resource.bind_group_layout,
        ];

        let compute_pipeline =
            super::create_compute_pipeline(WGPU_LABEL, device, shader, bind_group_layouts);

        Self { compute_pipeline }
    }

    /// Records the created compute pass to the provided `encoder`.
    /// Call this every frame to execute the probe step on `grid_points_count` grid points.
    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        grid_points_count: u32,
        compute_resources: &ComputeResources,
        common_resources: &CommonResources,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &compute_resources.df_grid_points.bind_group, &[]);
        compute_pass.set_bind_group(1, &compute_resources.distance_field.bind_group, &[]);
        compute_pass.set_bind_group(2, &common_resources.atoms_resource.bind_group, &[]);

        let work_groups_count = f32::ceil(grid_points_count as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(work_groups_count, 1, 1);
    }
}
