use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    position: [f32; 4],
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 0.0],
            view_matrix: cgmath::Matrix4::identity().into(),
            proj_matrix: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_uniform(&mut self, camera: &crate::Camera) {
        self.position = camera.get_position();
        self.view_matrix = camera.get_view_matrix().into();
        self.proj_matrix = camera.get_projection_matrix().into();
    }
}

pub struct CameraRender {
    camera_buffer: wgpu::Buffer,
    camera_uniform: CameraUniform,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
}

impl CameraRender {
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

    pub fn update(&mut self, queue: &wgpu::Queue, camera: &crate::Camera) {
        self.camera_uniform.update_uniform(camera);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
