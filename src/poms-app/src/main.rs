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

    let context = match GpuContext::initialize(window.clone()).await {
        Ok(ctx) => ctx,
        Err(e) => {
            log::error!("Failed to initialize GPU context: {:?}", e);
            #[cfg(target_arch = "wasm32")]
            wasm::show_webgpu_error();
            return;
        }
    };

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
    use super::app::App;
    use winit::platform::web::WindowExtWebSys;
    use winit::window::Window;

    /// Initialize the browser window by setting up logging and creating a canvas element.
    pub(crate) fn init_browser_window(window: &Window) {
        // Log detailed error info to browser's dev console
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_logger::init(wasm_logger::Config::default());

        // Create canvas that will be used for rendering
        let canvas = window.canvas().unwrap();
        // Set canvas style to fill the window
        canvas
            .set_attribute("style", "width: 100%; height: 100%;")
            .unwrap();

        append_node_to_body(&web_sys::Element::from(canvas));

        log::info!("Initialized browser window");
    }

    /// In case browser window is resized, adjust the canvas and app size (e.g. texture size) accordingly.
    pub(crate) fn resize_app_if_canvas_changed(window: &Window, app: &mut App) {
        let canvas = window.canvas().unwrap();
        let (width, height) = (canvas.client_width(), canvas.client_height());

        let logical_size = winit::dpi::LogicalSize { width, height };
        let scale_factor = window.scale_factor();
        let canvas_size = logical_size.to_physical(scale_factor);

        let window_size: winit::dpi::PhysicalSize<u32> = window.inner_size();

        // When converting to physical size, there might be a small difference due to rounding. Use a tolerance value to ignore it.
        const TOLERANCE_IN_PX: u32 = 1;

        fn abs_diff(a: u32, b: u32) -> u32 {
            a.max(b) - a.min(b)
        }

        if abs_diff(canvas_size.width, window_size.width) > TOLERANCE_IN_PX
            || abs_diff(canvas_size.height, window_size.height) > TOLERANCE_IN_PX
        {
            canvas.set_width(canvas_size.width);
            canvas.set_height(canvas_size.height);
            app.resize(canvas_size);
        }
    }

    /// Show an error message in the browser window if WebGPU is not supported or some other error occurred.
    pub(crate) fn show_webgpu_error() {
        let error_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();

        let compatibility_link =
            "https://developer.mozilla.org/en-US/docs/Web/API/GPU#browser_compatibility";

        error_element.set_inner_html(&format!(
            r#"<h1>Error.</h1>
            <a href='{}' style='text-decoration: underline;'>Please make sure your browser supports WebGPU.</a>
            <p>See console for more details.</p>"#,
            compatibility_link
        ));

        error_element
            .set_attribute(
                "style",
                r#"position: absolute; 
                top: 50%; 
                left: 50%; 
                transform: translate(-50%, -50%); 
                background-color: #f0f0f0; 
                color: #333; 
                border: 1px solid #333; 
                padding: 20px; 
                z-index: 10; 
                text-align: center; 
                font-family: monospace;"#,
            )
            .unwrap();

        append_node_to_body(&error_element);
    }

    fn append_node_to_body(node: &web_sys::Node) {
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| body.append_child(node).ok())
            .expect("Failed to append element to body");
    }
}
