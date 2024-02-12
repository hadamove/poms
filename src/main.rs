mod app;
mod context;
mod gui;
mod parser;
mod passes;
mod utils;

use std::sync::Arc;

use app::App;

use context::Context;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

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
        crate::utils::wasm::init_browser_window(&window);
        wasm_bindgen_futures::spawn_local(run_loop(event_loop, window));
    }
}

async fn run_loop(event_loop: EventLoop<()>, window: Window) {
    let context = Context::initialize(Arc::new(window)).await;
    let mut app = App::new(context);

    event_loop
        .run(|event, elwt| {
            #[cfg(target_arch = "wasm32")]
            utils::wasm::resize_app_if_canvas_changed(&window, &mut app);

            match event {
                Event::WindowEvent { event, .. } if !app.handle_event(&event) => {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => app.resize(physical_size),
                        // TODO: WindowEvent::ScaleFactorChanged
                        WindowEvent::RedrawRequested => {
                            app.redraw();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .expect("Failed to run event loop");
}
