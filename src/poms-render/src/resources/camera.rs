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
    /// Constructs a new instance of `CameraResource`.
    /// This resource is used to store the camera's position and matrices.
    pub fn new(device: &wgpu::Device) -> Self {
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_uniform_buffer"),
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
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
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

    /// Updates the camera uniform buffer with the new camera data.
    /// Usually called once per frame.
    pub fn update(
        &self,
        queue: &wgpu::Queue,
        position: cgmath::Point3<f32>,
        view_matrix: cgmath::Matrix4<f32>,
        projection_matrix: cgmath::Matrix4<f32>,
    ) {
        let uniform = CameraUniform {
            position: position.to_homogeneous().into(),
            view_matrix: view_matrix.into(),
            proj_matrix: projection_matrix.into(),
            // We can unwrap here because the matrices are invertible (unless something really weird happens).
            view_inverse_matrix: view_matrix.invert().unwrap().into(),
            proj_inverse_matrix: projection_matrix.invert().unwrap().into(),
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}
