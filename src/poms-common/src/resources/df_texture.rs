pub const DF_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Creates a new distance field texture with the given resolution.
/// Used both by the compute pipeline and the rendering pipeline.
pub fn create_distance_field_texture(device: &wgpu::Device, resolution: u32) -> wgpu::Texture {
    // wgpu requires that textures have at least 1 texel in each dimension.
    // When initializing resources, it may happen that the resolution is 0.
    let resolution = u32::max(resolution, 1);

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("distance_field_texture"),
        size: wgpu::Extent3d {
            width: resolution,
            height: resolution,
            depth_or_array_layers: resolution,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: DF_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    })
}
