use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::RenderResources;

#[derive(Clone, Copy)]
pub struct PostprocessSettings {
    pub is_ssao_enabled: bool,
    pub ssao_radius: f32,
    pub ssao_bias: f32,
    pub ssao_samples_count: u32,
    pub ssao_is_blur_enabled: bool,
}

impl Default for PostprocessSettings {
    fn default() -> Self {
        Self {
            is_ssao_enabled: true,
            ssao_radius: 5.0,
            ssao_bias: 1.0,
            ssao_samples_count: 32,
            ssao_is_blur_enabled: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PostprocessUniforms {
    pub is_ssao_enabled: u32,
}

/// Wrapper around `wgpu::RenderPipeline` that is used to render the final color texture together with postprocessing effects.
pub struct PostprocessPass {
    /// Direct handle to manipulate postprocessing effects (e.g. ssao)
    pub settings: PostprocessSettings,

    ssao_effect: wgrepp::ssao::SsaoEffect,
    ssao_resources: wgrepp::ssao::SsaoResources,
    uniform_buffer: wgpu::Buffer,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl PostprocessPass {
    const WGPU_LABEL: &'static str = "postprocess_pass";

    /// Creates a new instance of `PostprocessPass` using the provided resources.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        settings: PostprocessSettings,
        resources: &RenderResources,
    ) -> Self {
        let ssao_effect = wgrepp::ssao::SsaoEffect::new(device, queue, true);

        let ssao_resources = wgrepp::ssao::SsaoResources::new(
            &ssao_effect,
            device,
            queue,
            &resources.depth_texture.view,
            &resources.normal_texture.view,
            [config.width, config.height],
            settings.ssao_samples_count,
        );

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("postprocess_uniform_buffer"),
            contents: bytemuck::cast_slice(&[PostprocessUniforms {
                is_ssao_enabled: settings.is_ssao_enabled as u32,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = Self::create_postprocess_bind_group_layout(device);
        let bind_group = Self::create_postprocess_bind_group(
            device,
            &bind_group_layout,
            &uniform_buffer,
            resources,
            &ssao_resources,
        );

        let render_pipeline =
            Self::create_postprocess_render_pipeline(device, config, &bind_group_layout);

        Self {
            settings,
            ssao_effect,
            ssao_resources,
            uniform_buffer,
            bind_group_layout,
            bind_group,
            render_pipeline,
        }
    }

    /// Records the created render pass to the provided `encoder`.
    /// Call this every frame to render the Postprocess effect.
    pub fn render(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        resources: &RenderResources,
    ) {
        if self.settings.is_ssao_enabled {
            self.ssao_resources.draw(
                &self.ssao_effect,
                encoder,
                self.settings.ssao_is_blur_enabled,
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(Self::WGPU_LABEL),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(resources.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        let number_of_vertices = 3;
        render_pass.draw(0..number_of_vertices, 0..1);
    }

    /// Call this function on window resize, in which case, SSAO textures need to adapt.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        resources: &RenderResources,
    ) {
        self.ssao_resources.resize(
            &self.ssao_effect,
            device,
            &resources.depth_texture.view,
            &resources.normal_texture.view,
            [config.width, config.height],
        );
        // Since `color_texture` has been reassigned on resize, we have to recreate the bind group
        self.bind_group = Self::create_postprocess_bind_group(
            device,
            &self.bind_group_layout,
            &self.uniform_buffer,
            resources,
            &self.ssao_resources,
        );
    }

    /// Call this every frame to update the uniform buffer with the current settings.
    pub fn update_buffers(&mut self, queue: &wgpu::Queue, projection_matrix: cgmath::Matrix4<f32>) {
        self.ssao_resources.write_uniforms(
            queue,
            &cgmath::Matrix4::identity().into(),
            &projection_matrix.into(),
            self.settings.ssao_radius,
            self.settings.ssao_bias,
            [0, 0],
        );
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[PostprocessUniforms {
                is_ssao_enabled: self.settings.is_ssao_enabled as u32,
            }]),
        );
    }

    pub fn update_sampels_count(
        &mut self,
        samples_count: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resources: &RenderResources,
    ) {
        self.ssao_resources.update_samples_count(
            samples_count,
            device,
            queue,
            &resources.depth_texture.view,
            &resources.normal_texture.view,
        );
        self.bind_group = Self::create_postprocess_bind_group(
            device,
            &self.bind_group_layout,
            &self.uniform_buffer,
            resources,
            &self.ssao_resources,
        );
    }

    fn create_postprocess_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniform_buffer: &wgpu::Buffer,
        render_resources: &RenderResources,
        ssao_resources: &wgrepp::ssao::SsaoResources,
    ) -> wgpu::BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("postprocess_bind_group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &render_resources.color_texture.view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        ssao_resources.output_texture_view(),
                    ),
                },
            ],
        });
        bind_group
    }

    fn create_postprocess_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("postprocess_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_postprocess_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(Self::WGPU_LABEL),
                bind_group_layouts: &[bind_group_layout],
                ..Default::default()
            });
        let shader_desc = wgpu::include_wgsl!("../shaders/postprocess.wgsl");
        let shader_module = device.create_shader_module(shader_desc);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(Self::WGPU_LABEL),
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            depth_stencil: None,
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }
}
