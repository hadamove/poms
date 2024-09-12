use crate::ComputeResources;

/// Contains resources required to execute the refinement pass.
/// Bind groups are sorted by the frequency of change as advised by `wgpu` documentation.

/// Wrapper around `wgpu::ComputePipeline` that is used to execute the refinement step of the algorithm.
pub struct RefinementPass {
    compute_pipeline: wgpu::ComputePipeline,
}

const WGPU_LABEL: &str = "Compute Refinement Pass";

impl RefinementPass {
    /// Creates a new instance of `RefinementPass` using the provided resources.
    /// The refinement step computes the distance field for grid points classified as on the boundary of the molecular surface.
    pub fn new(device: &wgpu::Device, compute_resources: &ComputeResources) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/refinement.wgsl");

        let bind_group_layouts = &[
            &compute_resources.df_grid_points.bind_group_layout,
            &compute_resources.distance_field.bind_group_layout,
        ];

        let compute_pipeline =
            super::create_compute_pipeline(WGPU_LABEL, device, shader, bind_group_layouts);

        Self { compute_pipeline }
    }

    /// Records the created compute pass to the provided `encoder`.
    /// This method should be called every frame to execute the refinement step on `grid_points_count` grid points.
    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        grid_points_count: u32,
        compute_resources: &ComputeResources,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &compute_resources.df_grid_points.bind_group, &[]);
        compute_pass.set_bind_group(1, &compute_resources.distance_field.bind_group, &[]);

        let work_groups_count = f32::ceil(grid_points_count as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(work_groups_count, 1, 1);
    }
}
