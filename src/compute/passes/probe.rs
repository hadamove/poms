use crate::{
    common::resources::{
        atoms_with_lookup::AtomsWithLookupResource, grid::GridResource, CommonResources,
    },
    compute::{composer::ComputeOwnedResources, resources::df_grid::MixedComputeStuffResource},
};

use super::util;

const WGPU_LABEL: &str = "Compute Probe";

pub struct ProbeResources<'a> {
    pub atoms: &'a AtomsWithLookupResource,         // @group(0)
    pub df_grid: &'a GridResource,                  // @group(1)
    pub mixed_stuff: &'a MixedComputeStuffResource, // @group(2)
}

impl<'a> ProbeResources<'a> {
    pub fn new(resources: &'a ComputeOwnedResources, common: &'a CommonResources) -> Self {
        Self {
            mixed_stuff: &resources.mixed_stuff,
            df_grid: &resources.df_grid,
            atoms: &common.atoms_resource,
        }
    }
}

pub struct ProbePass {
    compute_pipeline: wgpu::ComputePipeline,
}

impl ProbePass {
    pub fn new(device: &wgpu::Device, resources: ProbeResources) -> Self {
        let shader = wgpu::include_wgsl!("../shaders/probe.wgsl");

        let bind_group_layouts = &[
            &resources.atoms.bind_group_layout,
            &resources.df_grid.bind_group_layout,
            &resources.mixed_stuff.bind_group_layout,
        ];

        let compute_pipeline =
            util::create_compute_pipeline(WGPU_LABEL, device, shader, bind_group_layouts);

        Self { compute_pipeline }
    }

    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        grid_points_count: u32,
        resources: ProbeResources,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &resources.atoms.bind_group, &[]);
        compute_pass.set_bind_group(1, &resources.df_grid.bind_group, &[]);
        compute_pass.set_bind_group(2, &resources.mixed_stuff.bind_group, &[]);

        let work_groups_count = f32::ceil(grid_points_count as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(work_groups_count, 1, 1);
    }
}
