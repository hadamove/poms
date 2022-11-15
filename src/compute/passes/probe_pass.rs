use super::resources::bind_group::ProbePassBindGroup;
use super::resources::buffer::ProbePassBuffers;
use super::shared::SharedResources;
use crate::compute::grid::NeighborAtomGrid;

pub struct ProbePass {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    buffers: ProbePassBuffers,

    compute_pipeline: wgpu::ComputePipeline,
}

impl ProbePass {
    pub fn new(
        device: &wgpu::Device,
        neighbor_atom_grid: &NeighborAtomGrid,
        shared_resources: &SharedResources,
    ) -> Self {
        let buffers = ProbePassBuffers::new(device, neighbor_atom_grid);

        let bind_group_layout =
            device.create_bind_group_layout(&ProbePassBindGroup::LAYOUT_DESCRIPTOR);
        let bind_group = ProbePassBindGroup::create(device, &bind_group_layout, &buffers);

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Probe Pass Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &shared_resources.bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/probe.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Probe Pass Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        Self {
            bind_group,
            bind_group_layout,
            buffers,
            compute_pipeline,
        }
    }

    pub fn get_grid_point_class_buffer(&self) -> &wgpu::Buffer {
        &self.buffers.grid_point_class_buffer
    }

    pub fn recreate_buffers(
        &mut self,
        device: &wgpu::Device,
        neighbor_atom_grid: &NeighborAtomGrid,
    ) {
        self.buffers = ProbePassBuffers::new(device, neighbor_atom_grid);
        self.bind_group =
            ProbePassBindGroup::create(device, &self.bind_group_layout, &self.buffers);
    }

    pub fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        shared_resources: &SharedResources,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.set_bind_group(1, &shared_resources.bind_group, &[]);

        let num_grid_points = shared_resources.ses_grid.get_num_grid_points();
        let num_work_groups = f32::ceil(num_grid_points as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
