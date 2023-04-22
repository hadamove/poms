use wgpu::util::DeviceExt;

use crate::utils::{
    constants::{DEFAULT_LIGHT_COLOR, DEFAULT_LIGHT_DIRECTION},
    dtos::LightData,
};

use super::{camera::arcball::ArcballCamera, Resource};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    direction: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

impl LightUniform {
    fn default() -> Self {
        Self {
            direction: DEFAULT_LIGHT_DIRECTION,
            _padding: 0,
            color: DEFAULT_LIGHT_COLOR,
            _padding2: 0,
        }
    }
}

pub struct LightResource {
    pub follow_camera: bool,
    buffer: wgpu::Buffer,
    uniform: LightUniform,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl LightResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let light_uniform = LightUniform::default();
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer: light_buffer,
            uniform: light_uniform,
            bind_group_layout: light_bind_group_layout,
            bind_group: light_bind_group,
            follow_camera: true,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, light_data: LightData) {
        self.follow_camera = light_data.follow_camera.unwrap_or(self.follow_camera);
        self.uniform.direction = light_data.direction.unwrap_or(self.uniform.direction);
        self.uniform.color = light_data.color.unwrap_or(self.uniform.color);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, camera: &ArcballCamera) {
        if self.follow_camera {
            self.uniform.direction = camera.get_look_direction().into();
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        }
    }
}

impl Resource for LightResource {
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
