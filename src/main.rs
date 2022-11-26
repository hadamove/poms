mod app;
mod compute;
mod gpu;
mod gui;
mod parser;
mod render;
mod render_pass;
mod shared;

use crate::app::App;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

fn main() {
    let event_loop = EventLoopBuilder::new().build();

    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Failed to create window");

    #[cfg(not(target_arch = "wasm32"))]
    {
        simple_logger::init_with_level(log::Level::Info).expect("Failed to initialize logger");
        futures::executor::block_on(run_loop(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        crate::shared::wasm::init_browser_window(&window);
        wasm_bindgen_futures::spawn_local(run_loop(event_loop, window));
    }
}

async fn run_loop(event_loop: EventLoop<()>, window: Window) {
    let mut app = App::new(&window).await;

    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        if app.gui.handle_winit_event(&event) {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        crate::shared::wasm::update_window_size_if_canvas_changed(&window, &mut app);

        app.input(&event);
        // TODO: Move this into app to make it more self-contained
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => app.resize(physical_size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    app.resize(*new_inner_size)
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let time_delta = now - last_render_time;
                last_render_time = now;
                app.update(time_delta);
                app.render(&window).unwrap();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
