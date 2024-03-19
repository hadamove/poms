use wgpu::util::DeviceExt;

use crate::{
    common::models::{
        atom::Atom,
        grid::{create_compute_grid_around_molecule, GridUniform},
    },
    compute::composer::ComputeProgress,
};

pub struct DistanceFieldGridResource {
    buffers: DistanceFieldGridBuffers,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DistanceFieldGridResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffers = DistanceFieldGridBuffers::new(device);
        let bind_group_layout =
            device.create_bind_group_layout(&DistanceFieldGridBindGroup::LAYOUT_DESCRIPTOR);

        let bind_group = DistanceFieldGridBindGroup::new(device, &buffers, &bind_group_layout).0;

        Self {
            buffers,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, atoms: &[Atom], progress: ComputeProgress) {
        if let Some(render_resolution) = progress.last_computed_resolution {
            let df_grid_render = create_compute_grid_around_molecule(
                atoms,
                render_resolution,
                progress.probe_radius,
            );
            queue.write_buffer(
                &self.buffers.df_grid_render_buffer,
                0,
                bytemuck::cast_slice(&[df_grid_render]),
            );
        }

        let df_grid_compute = create_compute_grid_around_molecule(
            atoms,
            progress.current_resolution,
            progress.probe_radius,
        );

        queue.write_buffer(
            &self.buffers.df_grid_compute_buffer,
            0,
            bytemuck::cast_slice(&[df_grid_compute]),
        );
        queue.write_buffer(
            &self.buffers.probe_radius_buffer,
            0,
            bytemuck::cast_slice(&[progress.probe_radius]),
        );

        queue.write_buffer(
            &self.buffers.grid_point_index_offset_buffer,
            0,
            bytemuck::cast_slice(&[progress.grid_points_index_offset()]),
        );
    }
}

struct DistanceFieldGridBuffers {
    df_grid_render_buffer: wgpu::Buffer,
    df_grid_compute_buffer: wgpu::Buffer,
    probe_radius_buffer: wgpu::Buffer,
    grid_point_index_offset_buffer: wgpu::Buffer,
}

impl DistanceFieldGridBuffers {
    fn new(device: &wgpu::Device) -> Self {
        let df_grid_render_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ses Grid Render Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GridUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let df_grid_compute_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ses Grid Compute Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GridUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let probe_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Probe Radius Uniform Buffer"),
            contents: bytemuck::cast_slice(&[1.4f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let grid_point_index_offset_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid point index offset Buffer"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Self {
            df_grid_render_buffer,
            df_grid_compute_buffer,
            probe_radius_buffer,
            grid_point_index_offset_buffer,
        }
    }
}

struct DistanceFieldGridBindGroup(wgpu::BindGroup);

impl DistanceFieldGridBindGroup {
    fn new(
        device: &wgpu::Device,
        buffers: &DistanceFieldGridBuffers,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.df_grid_compute_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.probe_radius_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.grid_point_index_offset_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.df_grid_render_buffer.as_entire_binding(),
                },
            ],
            label: Some("Shared Bind Group"),
        });

        Self(bind_group)
    }

    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::all(),
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
}
