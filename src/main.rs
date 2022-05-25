use camera::{Camera, CameraController};
use cgmath::{Point3, Vector3};
use renderer::camera::CameraRender;
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod camera;
mod parser;
mod renderer;
mod texture;

// Change this to see other molecules
const MOLECULE_FILE: &str = "./molecules/1aon.pdb";

const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.03,
    g: 0.03,
    b: 0.04,
    a: 1.00,
};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,

    atoms_bind_group: wgpu::BindGroup,
    num_vertices: u32,

    camera: Camera,
    camera_render: CameraRender,
    camera_controller: CameraController,

    depth_texture: texture::Texture,
    clear_color: wgpu::Color,
}

// TODO: clean up this class, chop it up into smaller components
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

        let molecule = parser::parse_pdb_file(MOLECULE_FILE.to_string()).await;

        let atoms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atoms Buffer"),
            contents: bytemuck::cast_slice(&molecule.atoms),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let atoms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Atoms Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let atoms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atoms Bind Group"),
            layout: &atoms_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 1,
                resource: atoms_buffer.as_entire_binding(),
            }],
        });

        let camera = Camera::new(
            molecule.calculate_centre().into(),
            Vector3::new(0.0, 0.0, 70.0),
            config.width as f32 / config.height as f32,
        );

        let camera_controller = CameraController::new(2.0);
        let camera_render = CameraRender::new(&device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_render.get_bind_group_layout(),
                    &atoms_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(&include_wgsl!("shaders/atom.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let num_vertices = molecule.atoms.len() as u32 * 6;
        let depth_texture = texture::Texture::create_depth_texture(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,

            render_pipeline,

            num_vertices,
            atoms_bind_group,

            camera,
            camera_render,
            camera_controller,

            depth_texture,
            clear_color: CLEAR_COLOR,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        self.camera.resize(new_size.width, new_size.height);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config)
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_render.update(&self.queue, &self.camera);
    }

    fn render(&mut self, frame: &wgpu::SurfaceTexture) -> Result<(), wgpu::SurfaceError> {
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_render.get_bind_group(), &[]);
            render_pass.set_bind_group(1, &self.atoms_bind_group, &[]);
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
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
            #[cfg(target_arch = "wasm32")]
            {
                use winit::platform::web::WindowExtWebSys;

                let canvas = window.canvas();
                let (width, height) = (canvas.client_width(), canvas.client_height());
                let factor = window.scale_factor();

                let logical = winit::dpi::LogicalSize { width, height };
                let new_size = logical.to_physical(factor);

                // Dynamically change the size of the canvas in the browser window
                if new_size != state.size {
                    canvas.set_width(new_size.width);
                    canvas.set_height(new_size.height);
                    state.resize(new_size);
                }
            }
            state.update();
            let frame = state.surface.get_current_texture().unwrap();
            match state.render(&frame) {
                Ok(_) => frame.present(),
                // Reconfigure surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(window.inner_size()),
                // OOM, exit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
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
        // Log detailed error info to browser's dev console
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_logger::init(wasm_logger::Config::default());

        // Append window to document body
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                let canvas = window.canvas();
                let style = canvas.style();
                // Set canvas to fill the whole window
                style.set_property("width", "100%").unwrap();
                style.set_property("height", "100%").unwrap();
                body.append_child(&web_sys::Element::from(canvas)).ok()
            })
            .expect("Failed to append canvas to body");
        wasm_bindgen_futures::spawn_local(run_loop(event_loop, window));
    }
}
