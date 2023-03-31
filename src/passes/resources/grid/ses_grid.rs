use wgpu::util::DeviceExt;

use super::super::{molecule::Molecule, Resource, SesState};
use super::{GridSpacing, GridUniform};

pub struct SesGridResource {
    buffers: SesGridBuffers,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl SesGridResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffers = SesGridBuffers::new(device);
        let bind_group_layout =
            device.create_bind_group_layout(&SesGridBindGroup::LAYOUT_DESCRIPTOR);

        Self {
            bind_group: SesGridBindGroup::new(device, &buffers, &bind_group_layout).0,
            bind_group_layout,
            buffers,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, molecule: &Molecule, ses_state: &SesState) {
        if ses_state.switch_ready() {
            let ses_grid_render = GridUniform::from_molecule(
                molecule,
                GridSpacing::Resolution(ses_state.get_render_resolution()),
                ses_state.probe_radius,
            );
            queue.write_buffer(
                &self.buffers.ses_grid_render_buffer,
                0,
                bytemuck::cast_slice(&[ses_grid_render]),
            );
        }

        let ses_grid_compute = GridUniform::from_molecule(
            molecule,
            GridSpacing::Resolution(ses_state.get_compute_resolution()),
            ses_state.probe_radius,
        );

        queue.write_buffer(
            &self.buffers.ses_grid_compute_buffer,
            0,
            bytemuck::cast_slice(&[ses_grid_compute]),
        );
        queue.write_buffer(
            &self.buffers.probe_radius_buffer,
            0,
            bytemuck::cast_slice(&[ses_state.probe_radius]),
        );
        queue.write_buffer(
            &self.buffers.grid_point_index_offset_buffer,
            0,
            bytemuck::cast_slice(&[ses_state.get_grid_point_index_offset()]),
        );
    }
}

impl Resource for SesGridResource {
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

struct SesGridBuffers {
    ses_grid_render_buffer: wgpu::Buffer,
    ses_grid_compute_buffer: wgpu::Buffer,
    probe_radius_buffer: wgpu::Buffer,
    grid_point_index_offset_buffer: wgpu::Buffer,
}

impl SesGridBuffers {
    fn new(device: &wgpu::Device) -> Self {
        let ses_grid_render_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ses Grid Render Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GridUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ses_grid_compute_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
            ses_grid_render_buffer,
            ses_grid_compute_buffer,
            probe_radius_buffer,
            grid_point_index_offset_buffer,
        }
    }
}

struct SesGridBindGroup(wgpu::BindGroup);

impl SesGridBindGroup {
    fn new(
        device: &wgpu::Device,
        buffers: &SesGridBuffers,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.ses_grid_compute_buffer.as_entire_binding(),
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
                    resource: buffers.ses_grid_render_buffer.as_entire_binding(),
                },
            ],
            label: Some("Shared Bind Group"),
        });

        Self(bind_group)
    }

    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'_> =
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
