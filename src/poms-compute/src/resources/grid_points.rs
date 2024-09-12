use wgpu::util::DeviceExt;

/// An auxiliary resource that stores values associated with each grid point (classifications, etc.).
/// This resource also stores the index offset in the buffer since the computation on grid points is split across multiple frames.
pub struct GridPointsResource {
    pub grid_point_memory_buffer: wgpu::Buffer,
    pub grid_point_index_offset_buffer: wgpu::Buffer,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl GridPointsResource {
    /// Creates a new instance of `GridPointsResource`.
    /// We create a buffer of size `max_resolution^3` in order to not have to resize it later as the resolution upscales.
    pub fn new(device: &wgpu::Device, max_resolution: u32) -> Self {
        // The maximum number of grid points is the cube of the maximum resolution.
        let grid_points_memory_size = max_resolution.pow(3) as usize;

        let grid_point_memory_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("grid_point_memory_storage_buffer"),
                contents: bytemuck::cast_slice(&vec![0u32; grid_points_memory_size]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let grid_point_index_offset_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("grid_point_index_offset_uniform_buffer"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout = device.create_bind_group_layout(&LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: grid_point_memory_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: grid_point_index_offset_buffer.as_entire_binding(),
                },
            ],
            label: Some("grid_points_bind_group"),
        });

        Self {
            grid_point_memory_buffer,
            grid_point_index_offset_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, grid_points_index_offset: u32) {
        queue.write_buffer(
            &self.grid_point_index_offset_buffer,
            0,
            bytemuck::cast_slice(&[grid_points_index_offset]),
        );
    }
}

const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
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
        ],
        label: Some("grid_points_bind_group_layout"),
    };
