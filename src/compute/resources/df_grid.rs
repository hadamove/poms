use wgpu::util::DeviceExt;

use crate::compute::composer::ComputeProgress;

// TODO: Come up with a name for this
// TODO: Move this file close to Compute stuff
pub struct GridPointsResource {
    // TODO: Move this to the AtomsWithLookupResource?
    probe_radius_buffer: wgpu::Buffer,
    // TODO: This should be together with grid_point_memory
    grid_point_index_offset_buffer: wgpu::Buffer,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl GridPointsResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let probe_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Probe Radius Uniform Buffer"),
            contents: bytemuck::cast_slice(&[1.4f32]), // TODO: Replace with constant
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let grid_point_index_offset_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid point index offset Buffer"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout = device.create_bind_group_layout(&LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: probe_radius_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: grid_point_index_offset_buffer.as_entire_binding(),
                },
            ],
            label: Some("Shared Bind Group"),
        });

        Self {
            probe_radius_buffer,
            grid_point_index_offset_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, progress: &ComputeProgress) {
        queue.write_buffer(
            &self.probe_radius_buffer,
            0,
            bytemuck::cast_slice(&[progress.probe_radius]),
        );

        queue.write_buffer(
            &self.grid_point_index_offset_buffer,
            0,
            bytemuck::cast_slice(&[progress.grid_points_index_offset()]),
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
                    ty: wgpu::BufferBindingType::Uniform,
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
        label: Some("Shared Bind Group Layout"),
    };
