pub mod molecular_surface;
pub mod postprocess;
pub mod spacefill;

use super::resources::color_texture::COLOR_TEXTURE_FORMAT;
use super::resources::depth_texture::DEPTH_TEXTURE_FORMAT;
use super::resources::normal_texture::NORMAL_TEXTURE_FORMAT;

fn create_render_pipeline(
    label: &'static str,
    device: &wgpu::Device,
    shader_desc: wgpu::ShaderModuleDescriptor,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts,
        ..Default::default()
    });
    let shader_module = device.create_shader_module(shader_desc);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: COLOR_TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: NORMAL_TEXTURE_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ],
            compilation_options: Default::default(),
        }),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_TEXTURE_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
