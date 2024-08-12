pub mod probe;
pub mod refinement;

fn create_compute_pipeline(
    label: &'static str,
    device: &wgpu::Device,
    shader_desc: wgpu::ShaderModuleDescriptor,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts,
        ..Default::default()
    });
    let shader_module = device.create_shader_module(shader_desc);

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&compute_pipeline_layout),
        module: &shader_module,
        entry_point: "main",
        compilation_options: Default::default(),
    })
}
