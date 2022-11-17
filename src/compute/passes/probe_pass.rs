use super::resources::bind_group::ProbePassBindGroup;
use super::resources::buffer::ProbePassBuffers;
use crate::gpu::GpuState;
use crate::shared::grid::MoleculeData;
use crate::shared::resources::SharedResources;

pub struct ProbePass {
    bind_group: wgpu::BindGroup,
    buffers: ProbePassBuffers,

    compute_pipeline: wgpu::ComputePipeline,
}

impl ProbePass {
    pub fn new(gpu: &GpuState) -> Self {
        let buffers = ProbePassBuffers::new(&gpu.device);

        let bind_group_layout = gpu
            .device
            .create_bind_group_layout(&ProbePassBindGroup::LAYOUT_DESCRIPTOR);

        let bind_group = ProbePassBindGroup::create(&gpu.device, &bind_group_layout, &buffers);

        let compute_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Probe Pass Compute Pipeline Layout"),
                    bind_group_layouts: &[
                        &bind_group_layout,
                        &gpu.shared_resources.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let compute_shader = gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/probe.wgsl"));

        let compute_pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Probe Pass Compute Pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &compute_shader,
                    entry_point: "main",
                });

        Self {
            bind_group,
            buffers,
            compute_pipeline,
        }
    }

    pub fn get_grid_point_class_buffer(&self) -> &wgpu::Buffer {
        &self.buffers.grid_point_class_buffer
    }

    pub fn on_molecule_changed(&mut self, queue: &wgpu::Queue, molecule: &MoleculeData) {
        self.buffers.update(queue, molecule);
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
        println!("num_grid_points: {}", num_grid_points);
        let num_work_groups = f32::ceil(num_grid_points as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
