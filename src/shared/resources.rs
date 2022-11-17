use super::grid::{GridSpacing, GridUniform, MoleculeData};
use wgpu::util::DeviceExt;

pub struct SharedResources {
    pub ses_grid: GridUniform,
    pub probe_radius: f32,

    // TODO: Refactor into one struct
    pub buffers: SharedBuffers,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    pub distance_field_texture: DistanceFieldTexture,
}

impl SharedResources {
    pub fn new(device: &wgpu::Device) -> Self {
        let ses_grid = GridUniform::default();
        let buffers = SharedBuffers::new(device);
        let bind_group_layout =
            device.create_bind_group_layout(&SharedBindGroup::LAYOUT_DESCRIPTOR);
        let bind_group = SharedBindGroup::new(device, &buffers, &bind_group_layout).0;

        let distance_field_texture = DistanceFieldTexture::create(device, 64);

        Self {
            ses_grid,
            probe_radius: 1.4,

            buffers,
            bind_group_layout,
            bind_group,

            distance_field_texture,
        }
    }

    pub fn update_ses(&mut self, queue: &wgpu::Queue, molecule: &MoleculeData) {
        let spacing = GridSpacing::Resolution(64);
        self.ses_grid = GridUniform::from_atoms(&molecule.atoms_sorted, spacing, self.probe_radius);
        queue.write_buffer(
            &self.buffers.ses_grid_buffer,
            0,
            bytemuck::cast_slice(&[self.ses_grid]),
        );
    }

    pub fn update_ses_resolution(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resolution: u32,
    ) {
        self.ses_grid.update_resolution(resolution);
        queue.write_buffer(
            &self.buffers.ses_grid_buffer,
            0,
            bytemuck::cast_slice(&[self.ses_grid]),
        );
        self.distance_field_texture = DistanceFieldTexture::create(device, resolution);
    }

    pub fn update_probe_radius(&mut self, queue: &wgpu::Queue, probe_radius: f32) {
        self.probe_radius = probe_radius;
        queue.write_buffer(
            &self.buffers.probe_radius_buffer,
            0,
            bytemuck::cast_slice(&[self.probe_radius]),
        );
    }
}

pub struct SharedBuffers {
    pub ses_grid_buffer: wgpu::Buffer,
    pub probe_radius_buffer: wgpu::Buffer,
}

impl SharedBuffers {
    pub fn new(device: &wgpu::Device) -> Self {
        let ses_grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ses Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GridUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let probe_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Probe Radius Uniform Buffer"),
            contents: bytemuck::cast_slice(&[1.4f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            ses_grid_buffer,
            probe_radius_buffer,
        }
    }
}

struct SharedBindGroup(pub wgpu::BindGroup);

impl SharedBindGroup {
    pub fn new(
        device: &wgpu::Device,
        shared_buffers: &SharedBuffers,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shared_buffers.ses_grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: shared_buffers.probe_radius_buffer.as_entire_binding(),
                },
            ],
            label: Some("Shared Bind Group"),
        });

        Self(bind_group)
    }

    const fn create_uniform_bind_group_layout(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'_> =
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                Self::create_uniform_bind_group_layout(0),
                Self::create_uniform_bind_group_layout(1),
            ],
            label: Some("Shared Bind Group Layout"),
        };
}

pub struct DistanceFieldTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    pub bind_group_compute_layout: wgpu::BindGroupLayout,
    pub bind_group_render_layout: wgpu::BindGroupLayout,

    pub bind_group_compute: wgpu::BindGroup,
    pub bind_group_render: wgpu::BindGroup,
}

impl DistanceFieldTexture {
    pub fn create(device: &wgpu::Device, resolution: u32) -> Self {
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
        });
        let view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_compute_layout =
            device.create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR_COMPUTE);
        let bind_group_compute = Self::create_bind_group(device, &bind_group_compute_layout, &view);

        let bind_group_render_layout =
            device.create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR_RENDER);
        let bind_group_render = Self::create_bind_group(device, &bind_group_render_layout, &view);

        Self {
            texture,
            view,
            sampler,
            bind_group_compute_layout,
            bind_group_render_layout,
            bind_group_compute,
            bind_group_render,
        }
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            }],
            label: Some("Distance Field Texture Bind Group"),
        })
    }

    pub const LAYOUT_DESCRIPTOR_COMPUTE: wgpu::BindGroupLayoutDescriptor<'_> =
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

    pub const LAYOUT_DESCRIPTOR_RENDER: wgpu::BindGroupLayoutDescriptor<'_> =
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D3,
                    multisampled: false,
                },
                count: None,
            }],
            label: Some("Distance Field Texture Render Bind Group Layout"),
        };
}
