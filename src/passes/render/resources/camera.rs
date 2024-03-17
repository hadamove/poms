use cgmath::{SquareMatrix, Zero};
use wgpu::util::DeviceExt;

use crate::utils::arcball::CameraController;

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

    pub fn update(&self, queue: &wgpu::Queue, camera: &CameraController) {
        let uniform = CameraUniform::from_camera(camera);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    position: [f32; 4],
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    view_inverse_matrix: [[f32; 4]; 4],
    proj_inverse_matrix: [[f32; 4]; 4],
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

    // TODO: Move this construction to ArcballCameraController instead to remove the dependency
    fn from_camera(camera: &CameraController) -> Self {
        let view_matrix = camera.view;
        let proj_matrix = camera.projection_matrix();

        Self {
            position: camera.position.to_homogeneous().into(),
            view_matrix: view_matrix.into(),
            proj_matrix: proj_matrix.into(),

            // We can unwrap here because the matrices are invertible.
            view_inverse_matrix: view_matrix.invert().unwrap().into(),
            proj_inverse_matrix: proj_matrix.invert().unwrap().into(),
        }
    }
}
