mod app;
mod gpu_context;

use std::sync::Arc;

use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use app::App;
use gpu_context::GpuContext;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window = WindowBuilder::new()
        .with_title("POMS")
        .build(&event_loop)
        .expect("Failed to create window");

    #[cfg(not(target_arch = "wasm32"))]
    {
        simple_logger::init_with_level(log::Level::Info).expect("Failed to initialize logger");
        futures::executor::block_on(run_loop(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm::init_browser_window(&window);
        wasm_bindgen_futures::spawn_local(run_loop(event_loop, window));
    }
}

async fn run_loop(event_loop: EventLoop<()>, window: Window) {
    let window = Arc::new(window);
    let context = GpuContext::initialize(window.clone()).await;
    let mut app = App::new(context);

    event_loop
        .run(|event, elwt| {
            #[cfg(target_arch = "wasm32")]
            wasm::resize_app_if_canvas_changed(&window, &mut app);

            match event {
                Event::WindowEvent { event, .. } if !app.handle_window_event(&event) => match event
                {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => app.resize(physical_size),
                    WindowEvent::RedrawRequested => {
                        app.redraw();
                    }
                    _ => {}
                },
                Event::DeviceEvent { event, .. } => app.handle_device_event(&event),
                Event::AboutToWait => window.request_redraw(),
                _ => {}
            }
        })
        .expect("Failed to run event loop");
}

/// When compiling for browsers, the `wasm32` target is used.
/// This module contains utilities specific to the browser environment.
#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::application::App;

    use winit::platform::web::WindowExtWebSys;
    use winit::window::Window;

    pub fn init_browser_window(window: &Window) {
        // Log detailed error info to browser's dev console
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_logger::init(wasm_logger::Config::default());

        // Append window to document body
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
    }

    pub fn resize_app_if_canvas_changed(window: &Window, app: &mut App) {
        let canvas = window.canvas();
        let (width, height) = (canvas.client_width(), canvas.client_height());

        let logical_size = winit::dpi::LogicalSize { width, height };
        let scale_factor = window.scale_factor();
        let canvas_size = logical_size.to_physical(scale_factor);

        if canvas_size != window.inner_size() {
            canvas.set_width(canvas_size.width);
            canvas.set_height(canvas_size.height);
            application.resize(canvas_size);
        }
    }
}
