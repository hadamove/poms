use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    direction: [f32; 3],
    /// Padding to align the struct to a multiple of 16 bytes.
    _padding: f32,
}

impl LightUniform {
    pub fn new(direction: [f32; 3]) -> Self {
        Self {
            direction,
            _padding: 0.0,
        }
    }
}

/// A lightweight (pun intended) resource that only contains the light direction
/// as the main render shaders only use directional lighting.
pub struct LightResource {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl LightResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_uniform_buffer"),
            contents: bytemuck::cast_slice(&[LightUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("light_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// This might be called every frame to update the light direction to match the camera's view.
    pub fn update(&self, queue: &wgpu::Queue, direction: cgmath::Vector3<f32>) {
        let uniform = LightUniform::new(direction.into());
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}
