use wgpu::util::DeviceExt;

use crate::utils::{
    constants::{DEFAULT_LIGHT_COLOR, DEFAULT_LIGHT_DIRECTION},
    dtos::LightData,
};

use super::{camera::arcball::ArcballCameraController, GpuResource};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    direction: [f32; 3],
    _padding: f32,
    color: [f32; 3],
    _padding2: f32,
}

impl LightUniform {
    fn default() -> Self {
        Self {
            direction: DEFAULT_LIGHT_DIRECTION,
            color: DEFAULT_LIGHT_COLOR,
            _padding: 0.0,
            _padding2: 0.0,
        }
    }
}

pub struct LightResource {
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl LightResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[LightUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
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
            label: Some("Light Bind Group"),
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

    pub fn update(&self, queue: &wgpu::Queue, light_data: LightData) {
        if let Some(direction) = light_data.direction {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[direction]));
        }
        if let Some(color) = light_data.color {
            let offset = std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress;
            queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(&[color]));
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &ArcballCameraController) {
        // TODO: Only update if we follow the camera (Add if statement)
        self.update(
            queue,
            LightData {
                direction: Some(camera.look_direction().into()),
                ..Default::default()
            },
        );
    }
}

impl GpuResource for LightResource {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
