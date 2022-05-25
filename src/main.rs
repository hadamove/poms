use camera::{Camera, CameraController};
use cgmath::Vector3;
use renderer::atom_pass::AtomRenderPass;
use renderer::camera::CameraRender;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod camera;
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
    camera_render: CameraRender,
    camera_controller: CameraController,
    atom_render_pass: AtomRenderPass,
}

impl State {
    async fn new(window: &Window) -> Self {
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
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let molecule = parser::parse_pdb_file().await;

        let camera = Camera::new(
            molecule.calculate_centre().into(),
            Vector3::new(0.0, 0.0, 70.0),
            config.width as f32 / config.height as f32,
        );

        let camera_controller = CameraController::new(2.0);
        let camera_render = CameraRender::new(&device);
        let atom_render_pass = AtomRenderPass::new(&device, &config, &camera_render, &molecule);

        Self {
            surface,
            device,
            queue,
            config,

            camera,
            camera_render,
            camera_controller,
            atom_render_pass,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        self.camera.resize(new_size.width, new_size.height);
        self.atom_render_pass.resize(&self.device, &self.config);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_render.update(&self.queue, &self.camera);
    }

    fn render(&mut self, frame: &wgpu::SurfaceTexture) {
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        self.atom_render_pass
            .render(&view, &mut encoder, &self.camera_render);

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

async fn run_loop(event_loop: EventLoop<()>, window: Window) {
    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            #[cfg(target_arch = "wasm32")]
            {
                // Dynamically change the size of the canvas in the browser window
                match web_utils::update_canvas_size(&window) {
                    None => {}
                    Some(new_size) => state.resize(new_size),
                }
            }

            let frame = state.surface.get_current_texture().unwrap();
            state.render(&frame);
            frame.present();
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}

fn main() {
    let event_loop = EventLoop::new();
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
