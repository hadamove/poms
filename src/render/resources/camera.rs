use cgmath::{SquareMatrix, Zero};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub position: [f32; 4],
    pub view_matrix: [[f32; 4]; 4],
    pub proj_matrix: [[f32; 4]; 4],
    pub view_inverse_matrix: [[f32; 4]; 4],
    pub proj_inverse_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    fn default() -> Self {
        let identity = cgmath::Matrix4::identity();
        Self {
            position: cgmath::Vector4::zero().into(),
            view_matrix: identity.into(),
            proj_matrix: identity.into(),
            view_inverse_matrix: identity.into(),
            proj_inverse_matrix: identity.into(),
        }
    }
}

pub struct CameraResource {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl CameraResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Camera Bind Group Layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer: camera_buffer,
            bind_group_layout: camera_bind_group_layout,
            bind_group: camera_bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, uniform: CameraUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}
