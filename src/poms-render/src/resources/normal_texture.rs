pub const NORMAL_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub struct NormalTexture {
    pub view: wgpu::TextureView,
}

impl NormalTexture {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Normal texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: NORMAL_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
        }
    }
}
