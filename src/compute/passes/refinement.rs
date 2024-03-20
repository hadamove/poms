use crate::compute::{
    composer::ComputeOwnedResources,
    resources::{distance_field::DistanceFieldCompute, grid_points::GridPointsResource},
};

const WGPU_LABEL: &str = "Compute Refinement";

pub struct RefinementResources<'a> {
    pub distance_field: &'a DistanceFieldCompute, // @group(0)
    pub df_grid_points: &'a GridPointsResource,   // @group(1)
}

impl<'a> RefinementResources<'a> {
    pub fn new(resources: &'a ComputeOwnedResources) -> Self {
        Self {
            df_grid_points: &resources.df_grid_points,
            distance_field: &resources.distance_field,
        }
    }
}

pub struct RefinementPass {
    compute_pipeline: wgpu::ComputePipeline,
}

impl RefinementPass {
    pub fn new(device: &wgpu::Device, resources: RefinementResources) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/refinement.wgsl");

        let bind_group_layouts = &[
            &resources.distance_field.bind_group_layout,
            &resources.df_grid_points.bind_group_layout,
        ];

        let compute_pipeline =
            super::util::create_compute_pipeline(WGPU_LABEL, device, shader, bind_group_layouts);

        Self { compute_pipeline }
    }

    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        grid_points_count: u32,
        resources: RefinementResources,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &resources.distance_field.bind_group, &[]);
        compute_pass.set_bind_group(1, &resources.df_grid_points.bind_group, &[]);

        let work_groups_count = f32::ceil(grid_points_count as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(work_groups_count, 1, 1);
    }
}
