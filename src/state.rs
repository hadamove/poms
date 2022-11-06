use std::vec;

use crate::compute::grid::{SESGrid, NeighborAtomGrid};
use crate::compute::passes::shared::SharedResources;
use crate::compute::passes::{dfr_pass::DistanceFieldRefinementPass, probe_pass::ProbePass};
use crate::gui::Gui;
use crate::render::passes::{
    gui_pass::GuiRenderPass, raymarch_pass::RaymarchDistanceFieldPass,
    spacefill_pass::SpacefillPass,
};
use crate::utils::molecule::ComputedMolecule;

use anyhow::Result;
use cgmath::Vector3;

use crate::render::passes::resources::{camera::CameraResource, texture};

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

    pub probe_compute_pass: ProbePass,
    pub drf_compute_pass: DistanceFieldRefinementPass,

    pub spacefill_pass: SpacefillPass,
    pub raymarch_pass: RaymarchDistanceFieldPass,
    pub gui_pass: GuiRenderPass,

    pub depth_texture: texture::Texture,

    pub gui: Gui,

    pub molecules: Vec<ComputedMolecule>,

    pub frame_count: u64,
    pub last_frame_time: f32,

    pub shared_resources: SharedResources,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let instance =
            // TODO: Fix Vulkan
            wgpu::Instance::new(wgpu::Backends::all().difference(wgpu::Backends::VULKAN));
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Could not find a suitable adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Could not create device");

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
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let gui_pass = GuiRenderPass::new(window, &device, &config);
        let gui = Gui::default();

        let file = rfd::AsyncFileDialog::new().pick_file().await.unwrap();
        let content = file.read().await;

        let molecule = parser::parse_pdb_file(&content).unwrap();

        let camera_eye: cgmath::Point3<f32> = molecule.calculate_centre().into();
        let offset = Vector3::new(-55., 36., -117.);

        let camera = camera::Camera::new(camera_eye + offset, cgmath::Deg(55.), cgmath::Deg(-11.0));

        let projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 1000.0);

        let camera_controller = camera::CameraController::new(100.0, 0.3);

        let camera_resource = CameraResource::new(&device);

        let ses_grid = SESGrid::from_molecule(&molecule, gui.ses_resolution, gui.probe_radius);
        let molecules = vec![ComputedMolecule::new(molecule, gui.probe_radius)];

        let shared_resources = SharedResources::new(&device, ses_grid);

        let neighbor_atom_grid = &molecules[0].neighbor_atom_grid;
        let probe_compute_pass = ProbePass::new(&device, neighbor_atom_grid, &shared_resources);

        let grid_point_class_buffer = probe_compute_pass.get_grid_point_class_buffer();
        let drf_compute_pass = DistanceFieldRefinementPass::new(&device, &shared_resources, grid_point_class_buffer);

        let spacefill_pass = SpacefillPass::new(&device, &config, &camera_resource, &molecules[0].molecule);
        let raymarch_pass = RaymarchDistanceFieldPass::new(
            &device,
            &config,
            &camera_resource,
            &shared_resources,
            drf_compute_pass.get_df_texture(),
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

            probe_compute_pass,
            drf_compute_pass,

            spacefill_pass,
            raymarch_pass,
            gui_pass,

            depth_texture,

            gui,
            molecules,
            frame_count: 0,
            last_frame_time: 0.0,
            
            shared_resources
        }
    }

    fn update_molecules(&mut self) {
        let files = &self.gui.files_to_load;
        if files.is_empty() {
            return;
        }

        let parsed_molecules_result = files
            .iter()
            .map(|file| parser::parse_pdb_file(file))
            .collect::<Result<Vec<_>>>();

        match parsed_molecules_result {
            Ok(parsed_molecules) => {
                self.molecules = parsed_molecules
                    .into_iter()
                    .map(|molecule| ComputedMolecule::new(molecule, self.gui.probe_radius))
                    .collect();

                self.update_passes();
                self.focus_camera();

                self.gui.error = None;
            }
            Err(e) => {
                self.gui.error = Some(format!("Could not load file:\n{}", e));
            }
        }
        self.gui.files_to_load = Vec::new();
    }

    fn update_passes(&mut self) {
        if self.molecules.is_empty() {
            return;
        }
        let molecule_index = (self.frame_count / 3) as usize % self.molecules.len();
        let molecule = &self.molecules[molecule_index];

        let ses_grid = SESGrid::from_molecule(&molecule.molecule, self.gui.ses_resolution, self.gui.probe_radius);
        self.shared_resources.update_ses_grid(&self.queue, ses_grid);
        self.shared_resources.update_probe_radius(&self.queue, self.gui.probe_radius);

        self.spacefill_pass = SpacefillPass::new(
            &self.device,
            &self.config,
            &self.camera_resource,
            &molecule.molecule,
        );

        self.probe_compute_pass
            .recreate_buffers(&self.device, &molecule.neighbor_atom_grid, &self.shared_resources);
        
        self.drf_compute_pass.recreate_df_texture(&self.device, &self.shared_resources, self.probe_compute_pass.get_grid_point_class_buffer());

        self.raymarch_pass
            .update_df_texture(&self.device, self.drf_compute_pass.get_df_texture());

        self.gui.compute_ses_once = true;
    }

    fn focus_camera(&mut self) {
        if let Some(molecule) = self.molecules.get(0) {
            let camera_eye: cgmath::Point3<f32> = molecule.molecule.calculate_centre().into();
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
            match crate::utils::wasm::update_canvas_size(&window) {
                None => {}
                Some(new_size) => self.resize(new_size),
            }
        }

        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Could not get surface texture");

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder: wgpu::CommandEncoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let depth_view = &self.depth_texture.view;

        // Render atoms
        if self.gui.render_spacefill {
            self.spacefill_pass
                .render(&view, depth_view, &mut encoder, &self.camera_resource);
        }

        // Compute SES surface
        if self.gui.compute_ses || self.gui.compute_ses_once {
            self.probe_compute_pass.execute(&mut encoder, &self.shared_resources);
            self.drf_compute_pass.execute(&mut encoder, &self.shared_resources);
            self.gui.compute_ses_once = false;
        }

        // Render SES surface
        if self.gui.render_ses_surface {
            self.raymarch_pass.render(
                &view,
                depth_view,
                &mut encoder,
                &self.camera_resource,
                self.gui.render_ses_surface,
            );
        }

        // Render GUI
        self.gui_pass
            .render(
                &view,
                &mut encoder,
                window,
                &self.device,
                &self.queue,
                &self.config,
                &mut self.gui,
            )
            .expect("Could not render GUI");

        // Submit commands to the GPU
        self.queue.submit(Some(encoder.finish()));

        // Draw a frame
        surface_texture.present();
    }

    pub fn update(&mut self, time_delta: std::time::Duration) {
        self.frame_count += 1;
        self.gui.frame_time = 0.9 * self.gui.frame_time + 0.1 * time_delta.as_secs_f32();

        self.camera_controller
            .update_camera(&mut self.camera, time_delta);

        self.camera_resource
            .update(&self.queue, &self.camera, &self.projection);

        self.update_molecules();

        if self.shared_resources.ses_grid.get_resolution() != self.gui.ses_resolution || self.shared_resources.ses_grid.probe_radius != self.gui.probe_radius {
            if self.shared_resources.ses_grid.probe_radius != self.gui.probe_radius {
                self.molecules.iter_mut().for_each(|molecule|
                    molecule.neighbor_atom_grid = NeighborAtomGrid::from_molecule(&molecule.molecule, self.gui.probe_radius)
                );
            }
            self.update_passes();
        }
    }
}
