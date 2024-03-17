use super::super::GpuResource;

pub struct DistanceFieldTexture {
    // Binding groups are different for comptue (write) and render (read).
    pub compute: DistanceFieldTextureCompute,
    pub render: DistanceFieldTextureRender,
    texture: wgpu::Texture,
}

impl DistanceFieldTexture {
    pub fn new(device: &wgpu::Device, resolution: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
        });
        let view = texture.create_view(&Default::default());

        Self {
            texture,
            compute: DistanceFieldTextureCompute::new(device, &view),
            render: DistanceFieldTextureRender::new(device, &view),
        }
    }

    pub fn resolution(&self) -> u32 {
        self.texture.depth_or_array_layers()
    }
}

pub struct DistanceFieldTextureCompute {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

pub struct DistanceFieldTextureRender {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl DistanceFieldTextureCompute {
    pub fn new(device: &wgpu::Device, view: &wgpu::TextureView) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            }],
            label: Some("Distance Field Texture Compute Bind Group"),
        });

        Self {
            bind_group_layout,
            bind_group,
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

impl GpuResource for DistanceFieldTextureCompute {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl DistanceFieldTextureRender {
    pub fn new(device: &wgpu::Device, view: &wgpu::TextureView) -> Self {
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
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Distance Field Texture Render Bind Group"),
        });

        Self {
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Distance Field Texture Render Bind Group Layout"),
        };
}

impl GpuResource for DistanceFieldTextureRender {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
