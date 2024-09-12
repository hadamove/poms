use poms_common::models::grid::GridUniform;
use poms_common::resources::df_texture::{create_distance_field_texture, DF_TEXTURE_FORMAT};
use wgpu::util::DeviceExt;

/// The signed distance field produced by the probe and refinement steps.
/// Each voxel in the distance field is a signed distance to the nearest surface.
pub struct DistanceField {
    pub grid_buffer: wgpu::Buffer,
    pub texture: wgpu::Texture,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DistanceField {
    /// Creates a new instance of `DistanceFieldCompute` with a new distance field texture.
    /// The resolution, origin, and scale of the grid are provided in the `GridUniform` struct.
    pub fn new(device: &wgpu::Device, grid: GridUniform) -> Self {
        let grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Buffer"),
            contents: bytemuck::cast_slice(&[grid]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture = create_distance_field_texture(device, grid.resolution);
        let view = texture.create_view(&Default::default());

        let bind_group_layout = device.create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR);

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
            ],
            label: Some("Distance Field Texture Compute Bind Group"),
        });

        Self {
            grid_buffer,
            bind_group_layout,
            bind_group,
            texture,
        }
    }

    /// Updates the grid buffer with the new grid data.
    pub fn update(&self, queue: &wgpu::Queue, grid: GridUniform) {
        queue.write_buffer(&self.grid_buffer, 0, bytemuck::cast_slice(&[grid]));
    }

    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Grid buffer
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
                // Distance field texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: DF_TEXTURE_FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
            ],
            label: Some("Distance Field Texture Compute Bind Group Layout"),
        };
}
