use crate::passes::resources::grid::molecule_grid::MoleculeGridResource;
use crate::passes::resources::grid::ses_grid::SesGridResource;

use super::{util, ComputeDependencies};

const WGPU_LABEL: &str = "Compute Probe";

pub struct ProbeResources<'a> {
    pub ses_grid: &'a SesGridResource,      // @group(0)
    pub molecule: &'a MoleculeGridResource, // @group(1)
}

impl<'a> ProbeResources<'a> {
    pub fn new(dependencies: &'a ComputeDependencies) -> Self {
        Self {
            ses_grid: dependencies.ses_grid,
            molecule: dependencies.molecule,
        }
    }
}

pub struct ProbePass {
    compute_pipeline: wgpu::ComputePipeline,
}

impl ProbePass {
    pub fn new(device: &wgpu::Device, resources: ProbeResources) -> Self {
        let shader = wgpu::include_wgsl!("./shaders/probe.wgsl");

        let bind_group_layouts = &[
            &resources.ses_grid.bind_group_layout,
            &resources.molecule.bind_group_layout,
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
        compute_pass.set_bind_group(0, &resources.ses_grid.bind_group, &[]);
        compute_pass.set_bind_group(1, &resources.molecule.bind_group, &[]);

        let work_groups_count = f32::ceil(grid_points_count as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(work_groups_count, 1, 1);
    }
}
