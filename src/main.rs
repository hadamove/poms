mod app;
mod context;
mod gui;
mod parser;
mod passes;
mod utils;

use app::App;

use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window = WindowBuilder::new()
        .with_title("POMS")
        .build(&event_loop)
        .expect("Failed to create window");

    window.set_title("POMS");

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
    let mut app = App::new(&window).await;

    event_loop
        .run(|event, elwt| {
            #[cfg(target_arch = "wasm32")]
            utils::wasm::resize_app_if_canvas_changed(&window, &mut app);

            match event {
                Event::WindowEvent { event, .. } if !app.handle_event(&window, &event) => {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => app.resize(physical_size),
                        // TODO: Fix Scale Factor
                        // WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        //     app.resize(*new_inner_size)
                        // }
                        WindowEvent::RedrawRequested => {
                            app.redraw(&window);
                        }
                        _ => {}
                    }
                }
                // TODO: Is this the right event?
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Failed to run event loop");
}
