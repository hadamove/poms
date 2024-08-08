pub fn create_distance_field_texture(device: &wgpu::Device, resolution: u32) -> wgpu::Texture {
    // wgpu requires that textures have at least 1 texel in each dimension.
    // When initializing resources, it may happen that the resolution is 0.
    let resolution = if resolution < 1 { 1 } else { resolution };
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Distance field texture"),
        size: wgpu::Extent3d {
            width: resolution,
            height: resolution,
            depth_or_array_layers: resolution,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba16Float],
    })
}
