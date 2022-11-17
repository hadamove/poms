use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use super::super::camera::{Camera, Projection};

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
        Self {
            position: [0.0, 0.0, 0.0, 0.0],
            view_matrix: cgmath::Matrix4::identity().into(),
            proj_matrix: cgmath::Matrix4::identity().into(),
            view_inverse_matrix: cgmath::Matrix4::identity().into(),
            proj_inverse_matrix: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_uniform(&mut self, camera: &Camera, projection: &Projection) {
        self.position = camera.position.to_homogeneous().into();
        let view_matrix = camera.calc_matrix();
        let proj_matrix = projection.calc_matrix();

        self.view_matrix = view_matrix.into();
        self.proj_matrix = proj_matrix.into();

        // We can unwrap here because the matrices are invertible.
        self.view_inverse_matrix = view_matrix.invert().unwrap().into();
        self.proj_inverse_matrix = proj_matrix.invert().unwrap().into();
    }
}

pub struct CameraResource {
    camera_buffer: wgpu::Buffer,
    camera_uniform: CameraUniform,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
}

impl CameraResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera_uniform = CameraUniform::default();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
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
            camera_buffer,
            camera_uniform,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    pub fn update(&mut self, queue: &wgpu::Queue, camera: &Camera, projection: &Projection) {
        self.camera_uniform.update_uniform(camera, projection);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
