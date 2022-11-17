use passes::dfr_pass::DistanceFieldRefinementPass;
use passes::probe_pass::ProbePass;

use super::gpu::GpuState;

mod passes;

pub struct ComputeJobs {
    pub probe_compute_pass: ProbePass,
    pub drf_compute_pass: DistanceFieldRefinementPass,
}

impl ComputeJobs {
    pub fn new(gpu: &GpuState) -> Self {
        let probe_compute_pass = ProbePass::new(gpu);
        let drf_compute_pass =
            DistanceFieldRefinementPass::new(gpu, probe_compute_pass.get_grid_point_class_buffer());
        Self {
            probe_compute_pass,
            drf_compute_pass,
        }
    }

    pub fn execute_passes(&mut self, gpu: &GpuState, encoder: &mut wgpu::CommandEncoder) {
        println!("Executing compute passes");
        self.probe_compute_pass
            .execute(encoder, &gpu.shared_resources);
        self.drf_compute_pass
            .execute(encoder, &gpu.shared_resources);
    }
}
