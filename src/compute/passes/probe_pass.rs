use crate::compute::grid::{NeighborAtomGrid, SESGrid};
use crate::utils::molecule::Molecule;

use super::resources::bind_group::ProbePassBindGroup;
use super::resources::buffer::{ProbePassBuffers, SharedBuffers};

pub struct ProbePass {
    bind_group: wgpu::BindGroup,
    shared_buffers: SharedBuffers,
    compute_pipeline: wgpu::ComputePipeline,

    num_grid_points: u32,
}

impl ProbePass {
    pub fn new(device: &wgpu::Device, molecule: &Molecule, ses_grid: &SESGrid) -> Self {
        let neighbor_atom_grid = NeighborAtomGrid::from_molecule(molecule);

        let buffers = ProbePassBuffers::new(device, &neighbor_atom_grid);
        let shared_buffers = SharedBuffers::new(device, ses_grid);

        let bind_group_layout =
            device.create_bind_group_layout(&ProbePassBindGroup::LAYOUT_DESCRIPTOR);
        let bind_group =
            ProbePassBindGroup::init(device, &bind_group_layout, &buffers, &shared_buffers);

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_shader =
            device.create_shader_module(&wgpu::include_wgsl!("../shaders/probe.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        Self {
            bind_group,
            shared_buffers,
            compute_pipeline,
            num_grid_points: ses_grid.get_num_grid_points(),
        }
    }

    pub fn update_grid(&mut self, queue: &wgpu::Queue, ses_grid: &SESGrid) {
        queue.write_buffer(
            &self.shared_buffers.ses_grid_buffer,
            0,
            bytemuck::cast_slice(&[ses_grid.uniform]),
        );
        self.num_grid_points = ses_grid.get_num_grid_points();
    }

    pub fn get_shared_buffers(&self) -> &SharedBuffers {
        &self.shared_buffers
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        let num_work_groups = f32::ceil(self.num_grid_points as f32 / 64.0) as u32;
        println!("Executing Probe pass {} work groups", num_work_groups);

        compute_pass.dispatch(num_work_groups, 1, 1);
    }
}
