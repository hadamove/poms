use wgpu::util::DeviceExt;

use crate::common::{
    models::grid::GridUniform, resources::df_texture::create_distance_field_texture,
};

pub struct DistanceFieldCompute {
    pub probe_radius_buffer: wgpu::Buffer,
    pub grid_buffer: wgpu::Buffer,
    pub texture: wgpu::Texture,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DistanceFieldCompute {
    pub fn new(device: &wgpu::Device, resolution: u32) -> Self {
        let probe_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Probe Radius Uniform Buffer"),
            contents: bytemuck::cast_slice(&[1.4f32]), // TODO: Replace with constant
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Buffer"),
            contents: bytemuck::cast_slice(&[GridUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture = create_distance_field_texture(device, resolution);
        let view = texture.create_view(&Default::default());

        let bind_group_layout = device.create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: probe_radius_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
            label: Some("Distance Field Texture Compute Bind Group"),
        });

        Self {
            probe_radius_buffer,
            grid_buffer,
            bind_group_layout,
            bind_group,
            texture,
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, probe_radius: f32, grid: &GridUniform) {
        queue.write_buffer(
            &self.probe_radius_buffer,
            0,
            bytemuck::cast_slice(&[probe_radius]),
        );
        queue.write_buffer(&self.grid_buffer, 0, bytemuck::cast_slice(&[*grid]));
    }

    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Probe radius
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Grid buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Distance field texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
            ],
            label: Some("Distance Field Texture Compute Bind Group Layout"),
        };
}
