use super::{
    resources::{bind_group::DistanceFieldRefinementPassBindGroup, texture::DistanceFieldTexture},
    shared::SharedResources,
};

pub struct DistanceFieldRefinementPass {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,

    df_texture: wgpu::Texture,
    compute_pipeline: wgpu::ComputePipeline,
}

impl DistanceFieldRefinementPass {
    pub fn new(
        device: &wgpu::Device,
        shared_resources: &SharedResources,
        grid_point_class_buffer: &wgpu::Buffer,
    ) -> Self {
        let resolution = shared_resources.ses_grid.get_resolution();

        let bind_group_layout = device
            .create_bind_group_layout(&DistanceFieldRefinementPassBindGroup::LAYOUT_DESCRIPTOR);

        let df_texture = DistanceFieldTexture::create(device, resolution);
        let df_texture_view = df_texture.create_view(&Default::default());

        let bind_group = DistanceFieldRefinementPassBindGroup::create(
            device,
            &bind_group_layout,
            grid_point_class_buffer,
            &df_texture_view,
        );

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("DFR Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &shared_resources.bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/dfr.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("DFR Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        Self {
            bind_group,
            bind_group_layout,

            df_texture,

            compute_pipeline,
        }
    }

    pub fn recreate_df_texture(&mut self, device: &wgpu::Device, shared_resources: &SharedResources, grid_point_class_buffer: &wgpu::Buffer) {
        // In case the resolution has changed, we need to recreate the texture.
        let resolution = shared_resources.ses_grid.get_resolution();

        self.df_texture = DistanceFieldTexture::create(device, resolution);
        let df_texture_view = self.df_texture.create_view(&Default::default());

        self.bind_group = DistanceFieldRefinementPassBindGroup::create(
            device,
            &self.bind_group_layout,
            grid_point_class_buffer,
            &df_texture_view,
        );
    }

    pub fn get_df_texture(&self) -> &wgpu::Texture {
        &self.df_texture
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, shared_resources: &SharedResources) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.set_bind_group(1, &shared_resources.bind_group, &[]);

        let num_grid_points = shared_resources.ses_grid.get_num_grid_points();
        let num_work_groups = f32::ceil(num_grid_points as f32 / 64.0) as u32;

        compute_pass.dispatch_workgroups(num_work_groups, 1, 1);
    }
}
