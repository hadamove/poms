mod compute;
mod gui;
mod render;
mod state;
mod utils;

use crate::gui::egui;
use crate::state::State;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    let event_loop = winit::event_loop::EventLoop::with_user_event();
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
        wasm::init_browser_window(&window);
        wasm_bindgen_futures::spawn_local(run_loop(event_loop, window));
    }
}

async fn run_loop(event_loop: EventLoop<egui::Event>, window: Window) {
    let event_loop_proxy = event_loop.create_proxy();
    let mut state = State::new(&window, event_loop_proxy).await;

    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => {
            if !state.input(&event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => state.resize(physical_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(*new_inner_size)
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
            state.render(&window);
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            if state.camera_controller.is_mouse_pressed() {
                state.camera_controller.process_mouse(delta.0, delta.1)
            }
        }
        _ => {}
    });
}
