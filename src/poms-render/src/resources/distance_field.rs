use poms_common::{
    models::grid::GridUniform, resources::df_texture::create_distance_field_texture,
};
use wgpu::util::DeviceExt;

/// The signed distance field that is used for rendering of the molecular surface using raymarching.
/// Each voxel in the distance field is a signed distance to the nearest surface.
/// The distance field can either be provided externally or generated using the `poms-compute` crate.
pub struct DistanceFieldRender {
    pub grid_buffer: wgpu::Buffer,
    pub texture: wgpu::Texture,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DistanceFieldRender {
    /// Creates a new instance of `DistanceFieldRender` with a new distance field texture.
    /// The resolution, origin, and scale of the grid are provided in the `GridUniform` struct.
    pub fn new(device: &wgpu::Device, grid: GridUniform) -> Self {
        let texture = create_distance_field_texture(device, grid.resolution);
        Self::from_texture(device, grid, texture)
    }

    /// Constructs a new instance of `DistanceFieldRender` from an existing texture.
    pub fn from_texture(device: &wgpu::Device, grid: GridUniform, texture: wgpu::Texture) -> Self {
        let view = texture.create_view(&Default::default());

        let grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Buffer"),
            contents: bytemuck::cast_slice(&[grid]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Distance Field Texture Render Bind Group"),
        });

        Self {
            grid_buffer,
            texture,
            bind_group_layout,
            bind_group,
        }
    }

    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Distance Field Texture Render Bind Group Layout"),
        };
}
