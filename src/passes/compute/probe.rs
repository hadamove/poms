use super::super::resources::ResourceRepo;
use crate::passes::resources::grid::molecule_grid::MoleculeGridResource;
use crate::passes::resources::{grid::ses_grid::SesGridResource, GpuResource};

use super::util;

const WGPU_LABEL: &str = "Compute Probe";

pub struct ComputeProbeResources {
    pub ses_grid: SesGridResource,      // @group(0)
    pub molecule: MoleculeGridResource, // @group(1)
}

pub struct ComputeProbePass {
    resources: ComputeProbeResources,
    compute_pipeline: wgpu::ComputePipeline,
}

impl ComputeProbePass {
    pub fn new(device: &wgpu::Device, resources: ComputeProbeResources) -> Self {
        let shader = wgpu::include_wgsl!("./shaders/probe.wgsl");

        let bind_group_layouts = &[
            resources.ses_grid.bind_group_layout(),
            resources.molecule.bind_group_layout(),
        ];

        let compute_pipeline =
            util::create_compute_pipeline(WGPU_LABEL, device, shader, bind_group_layouts);

        Self {
            resources,
            compute_pipeline,
        }
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, resources: &ResourceRepo) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, self.resources.ses_grid.bind_group(), &[]);
        compute_pass.set_bind_group(1, self.resources.molecule.bind_group(), &[]);

        let num_grid_points = resources.get_num_grid_points();
        let num_work_groups = f32::ceil(num_grid_points as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
