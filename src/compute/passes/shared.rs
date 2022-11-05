use wgpu::util::DeviceExt;

use crate::compute::grid::SESGrid;

pub struct SharedResources {
    pub ses_grid: SESGrid,

    pub buffers: SharedBuffers,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl SharedResources {
    pub fn new(device: &wgpu::Device, ses_grid: SESGrid) -> Self {
        let buffers = SharedBuffers::new(device, &ses_grid);
        let bind_group_layout = device.create_bind_group_layout(&SharedBindGroup::LAYOUT_DESCRIPTOR);
        let bind_group = SharedBindGroup::new(device,  &buffers, &bind_group_layout).0;

        Self {
            ses_grid,
            buffers,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update_ses_grid(&mut self, queue: &wgpu::Queue, ses_grid: SESGrid) {
        self.ses_grid = ses_grid;
        queue.write_buffer(
            &self.buffers.ses_grid_buffer,
            0,
            bytemuck::cast_slice(&[self.ses_grid.uniform]),
        );
    }

    pub fn update_probe_radius(&mut self, queue: &wgpu::Queue, probe_radius: f32) {
        self.ses_grid.probe_radius = probe_radius;
        queue.write_buffer(
            &self.buffers.probe_radius_buffer,
            0,
            bytemuck::cast_slice(&[self.ses_grid.probe_radius]),
        );
    }
}

pub struct SharedBuffers {
    pub ses_grid_buffer: wgpu::Buffer,
    pub probe_radius_buffer: wgpu::Buffer,
}

impl SharedBuffers {
    pub fn new(device: &wgpu::Device, ses_grid: &SESGrid) -> Self {
        let ses_grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SES Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[ses_grid.uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let probe_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Probe Radius Uniform Buffer"),
            contents: bytemuck::cast_slice(&[ses_grid.probe_radius]),
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
