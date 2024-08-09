/// Creates a new distance field texture with the given resolution.
/// Used both by the compute pipeline and the rendering pipeline.
pub fn create_distance_field_texture(device: &wgpu::Device, resolution: u32) -> wgpu::Texture {
    // Due to the requirements of the underlying wgpu API, we must ensure that the resolution is not zero.
    // This should not happen in practice.
    assert!(resolution > 0);

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
