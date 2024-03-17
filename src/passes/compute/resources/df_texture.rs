use crate::passes::resources::common::df_texture::create_distance_field_texture;

pub struct DistanceFieldTextureCompute {
    pub texture: wgpu::Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DistanceFieldTextureCompute {
    pub fn new_with_resolution(device: &wgpu::Device, resolution: u32) -> Self {
        let texture = create_distance_field_texture(device, resolution);
        Self::from_texture(device, texture)
    }

    pub fn from_texture(device: &wgpu::Device, texture: wgpu::Texture) -> Self {
        let view = texture.create_view(&Default::default());

        let bind_group_layout = device.create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
            label: Some("Distance Field Texture Compute Bind Group"),
        });

        Self {
            bind_group_layout,
            bind_group,
            texture,
        }
    }

    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D3,
                },
                count: None,
            }],
            label: Some("Distance Field Texture Compute Bind Group Layout"),
        };
}
