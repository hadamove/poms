use crate::compute::grid::{GridSpacing, SESGrid};
use crate::compute::passes::dfr_pass::DistanceFieldRefinementPass;
use crate::compute::passes::probe_pass::ProbePass;
use crate::gui::egui;
use crate::render::passes::raymarch_pass::RaymarchDistanceFieldPass;
use crate::render::passes::spacefill_pass::SpacefillPass;
use cgmath::Vector3;

use crate::render::passes::resources::camera::CameraResource;
use crate::render::passes::resources::texture;

use crate::utils::camera::{self, Camera, CameraController, Projection};
use crate::utils::parser;
use winit::{event::*, window::Window};

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    pub camera: Camera,
    pub projection: Projection,
    pub camera_resource: CameraResource,
    pub camera_controller: CameraController,

    pub ses_grid: SESGrid,

    pub probe_compute_pass: ProbePass,
    pub drf_compute_pass: DistanceFieldRefinementPass,

    pub spacefill_pass: SpacefillPass,
    pub raymarch_pass: RaymarchDistanceFieldPass,

    pub depth_texture: texture::Texture,

    pub gui: egui::Gui,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let size = window.inner_size();
        let supported_format = surface
            .get_supported_formats(&adapter)
            .get(0)
            .expect("No format supported")
            .to_owned();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: supported_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let gui = egui::Gui::new(window, &device, &config);

        let molecule = parser::parse_pdb_file(&"./molecules/1cqw.pdb".to_string());

        let camera_eye: cgmath::Point3<f32> = molecule.calculate_centre().into();
        let offset = Vector3::new(0.0, 0.0, 50.0);

        let camera = camera::Camera::new(camera_eye + offset, cgmath::Deg(-90.0), cgmath::Deg(0.0));

        let projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 1000.0);

        let camera_controller = camera::CameraController::new(100.0, 0.3);

        let camera_resource = CameraResource::new(&device);

        let ses_grid = SESGrid::from_molecule(&molecule, gui.my_app.ses_resolution);

        let probe_compute_pass = ProbePass::new(&device, &molecule, &ses_grid);
        let shared_buffers = probe_compute_pass.get_shared_buffers();

        let drf_compute_pass = DistanceFieldRefinementPass::new(&device, &ses_grid, shared_buffers);

        let spacefill_pass = SpacefillPass::new(&device, &config, &camera_resource, &molecule);
        let raymarch_pass = RaymarchDistanceFieldPass::new(
            &device,
            &config,
            &camera_resource,
            &shared_buffers.ses_grid_buffer,
            drf_compute_pass.get_distance_field_buffer(),
        );

        let depth_texture = texture::Texture::create_depth_texture(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,

            camera,
            projection,
            camera_resource,
            camera_controller,

            ses_grid,

            probe_compute_pass,
            drf_compute_pass,

            spacefill_pass,
            raymarch_pass,

            depth_texture,

            gui,
        }
    }

    fn update_molecule(&mut self) {
        if let Some(path) = &self.gui.my_app.file_to_load {
            let molecule = parser::parse_pdb_file(path);
            self.ses_grid = SESGrid::from_molecule(&molecule, self.gui.my_app.ses_resolution);

            self.spacefill_pass =
                SpacefillPass::new(&self.device, &self.config, &self.camera_resource, &molecule);
            self.probe_compute_pass = ProbePass::new(&self.device, &molecule, &self.ses_grid);
            let shared_buffers = self.probe_compute_pass.get_shared_buffers();

            self.drf_compute_pass =
                DistanceFieldRefinementPass::new(&self.device, &self.ses_grid, shared_buffers);

            self.gui.my_app.file_to_load = None;

            let camera_eye: cgmath::Point3<f32> = molecule.calculate_centre().into();
            let offset = Vector3::new(0.0, 0.0, 50.0);

            self.camera =
                camera::Camera::new(camera_eye + offset, cgmath::Deg(-90.0), cgmath::Deg(0.0));
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        self.projection.resize(new_size.width, new_size.height);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config)
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn render(&mut self, window: &Window) {
        #[cfg(target_arch = "wasm32")]
        {
            // Dynamically change the size of the canvas in the browser window
            match wasm::update_canvas_size(&window) {
                None => {}
                Some(new_size) => state.resize(new_size),
            }
        }

        let surface_texture = self.surface.get_current_texture().unwrap();

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder: wgpu::CommandEncoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let depth_view = &self.depth_texture.view;

        self.probe_compute_pass.execute(&mut encoder);

        // Render atoms
        if self.gui.my_app.render_spacefill {
            self.spacefill_pass
                .render(&view, depth_view, &mut encoder, &self.camera_resource);
        }

        // Render SES surface
        if self.gui.my_app.render_ses_surface {
            self.drf_compute_pass.execute(&mut encoder);
            self.raymarch_pass
                .render(&view, depth_view, &mut encoder, &self.camera_resource)
        }

        // Render GUI
        self.gui.render(
            &view,
            &mut encoder,
            window,
            &self.device,
            &self.queue,
            &self.config,
        );

        // Submit commands to the GPU
        self.queue.submit(Some(encoder.finish()));

        // Draw a frame
        surface_texture.present();
    }

    pub fn update(&mut self, time_delta: std::time::Duration) {
        self.camera_controller
            .update_camera(&mut self.camera, time_delta);

        self.camera_resource
            .update(&self.queue, &self.camera, &self.projection);
        self.update_molecule();

        self.ses_grid
            .uniform
            .update_spacing(GridSpacing::Resolution(self.gui.my_app.ses_resolution));

        self.probe_compute_pass
            .update_grid(&self.queue, &self.ses_grid);

        self.drf_compute_pass.update_grid(&self.ses_grid);
    }
}
