use camera::{Camera, CameraController, Projection};
use cgmath::Vector3;
use compute::probe_pass::ProbeComputePass;
use gui::egui;
use renderer::camera::CameraRender;
use renderer::{atom_pass::AtomRenderPass, ses_pass::SESRenderPass};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
};

mod camera;
mod compute;
mod grid;
mod gui;
mod parser;
mod renderer;
mod texture;
mod web_utils;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    camera: Camera,
    projection: Projection,
    camera_render: CameraRender,
    camera_controller: CameraController,

    atom_render_pass: AtomRenderPass,
    ses_render_pass: SESRenderPass,
    probe_compute_pass: ProbeComputePass,

    depth_texture: texture::Texture,

    gui: egui::Gui,
}

impl State {
    async fn new(window: &Window, event_loop_proxy: EventLoopProxy<egui::Event>) -> Self {
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

        // TODO: remove feature requirement
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
                    limits: adapter.limits(),
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let molecule = parser::parse_pdb_file(&"./molecules/1cqw.pdb".to_string());

        let camera_eye: cgmath::Point3<f32> = molecule.calculate_centre().into();
        let offset = Vector3::new(0.0, 0.0, 50.0);

        let camera = camera::Camera::new(camera_eye + offset, cgmath::Deg(-90.0), cgmath::Deg(0.0));

        let projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 1000.0);

        let camera_controller = camera::CameraController::new(100.0, 0.3);

        let camera_render = CameraRender::new(&device);
        let atom_render_pass = AtomRenderPass::new(&device, &config, &camera_render, &molecule);
        let ses_render_pass = SESRenderPass::new(&device, &config, &camera_render, &molecule);
        let probe_compute_pass = ProbeComputePass::new(&device, &config, &camera_render, &molecule);

        let depth_texture = texture::Texture::create_depth_texture(&device, &config);

        let gui = egui::Gui::new(&window, &device, &config, event_loop_proxy);

        Self {
            surface,
            device,
            queue,
            config,

            camera,
            projection,
            camera_render,
            camera_controller,

            atom_render_pass,
            ses_render_pass,
            probe_compute_pass,
            depth_texture,

            gui,
        }
    }

    fn update_molecule(&mut self) {
        if let Some(path) = &self.gui.my_app.file_to_load {
            let molecule = parser::parse_pdb_file(path);
            self.atom_render_pass =
                AtomRenderPass::new(&self.device, &self.config, &self.camera_render, &molecule);
            self.gui.my_app.file_to_load = None;

            let camera_eye: cgmath::Point3<f32> = molecule.calculate_centre().into();
            let offset = Vector3::new(0.0, 0.0, 50.0);

            self.camera =
                camera::Camera::new(camera_eye - offset, cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        self.projection.resize(new_size.width, new_size.height);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config)
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event) || self.gui.handle_events(event)
    }

    fn update(&mut self, time_delta: std::time::Duration) {
        self.camera_controller
            .update_camera(&mut self.camera, time_delta);

        self.camera_render
            .update(&self.queue, &self.camera, &self.projection);
        self.update_molecule();
    }
}

async fn run_loop(event_loop: EventLoop<egui::Event>, window: Window) {
    let event_loop_proxy = event_loop.create_proxy();
    let mut state = State::new(&window, event_loop_proxy).await;

    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                if !state.input(&event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(*new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let time_delta = now - last_render_time;
                last_render_time = now;
                state.update(time_delta);

                #[cfg(target_arch = "wasm32")]
                {
                    // Dynamically change the size of the canvas in the browser window
                    match web_utils::update_canvas_size(&window) {
                        None => {}
                        Some(new_size) => state.resize(new_size),
                    }
                }

                let surface_texture = state.surface.get_current_texture().unwrap();

                let view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder: wgpu::CommandEncoder = state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let depth_view = state
                    .depth_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                state.probe_compute_pass.execute(&mut encoder);
                state.probe_compute_pass.render(
                    &view,
                    &depth_view,
                    &mut encoder,
                    &state.camera_render,
                );

                // Copy data from the probe compute pass to cpu

                // Render atoms
                state.atom_render_pass.render(
                    &view,
                    &depth_view,
                    &mut encoder,
                    &state.camera_render,
                );

                // state.ses_render_pass.render(
                //     &view,
                //     &depth_view,
                //     &mut encoder,
                //     &state.camera_render,
                // );

                // Render GUI
                state.gui.render(
                    &view,
                    &mut encoder,
                    &window,
                    &state.device,
                    &state.queue,
                    &state.config,
                );

                // Submit commands to the GPU
                state.queue.submit(Some(encoder.finish()));

                // Draw a frame
                surface_texture.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if state.camera_controller.is_mouse_pressed() {
                state.camera_controller.process_mouse(delta.0, delta.1)
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::with_user_event();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        simple_logger::init_with_level(log::Level::Info).unwrap();
        futures::executor::block_on(run_loop(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_utils::init_browser_window(&window);
        wasm_bindgen_futures::spawn_local(run_loop(event_loop, window));
    }
}
