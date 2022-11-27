use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
};

#[derive(Debug, Default)]
pub struct Input {
    pub scroll: f32,
    pub mouse_pressed: bool,
    pub mouse_delta: (f64, f64),
}

impl Input {
    pub fn handle_winit_event<T>(&mut self, event: &Event<T>) {
        match event {
            Event::DeviceEvent { event, .. } => self.handle_device_event(event),
            Event::WindowEvent { event, .. } => self.handle_window_event(event),
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.mouse_delta = (0.0, 0.0);
        self.scroll = 0.0;
    }

    fn handle_device_event(&mut self, event: &DeviceEvent) {
        #[allow(clippy::single_match)]
        match event {
            DeviceEvent::MouseMotion { delta } => self.mouse_delta = *delta,
            _ => {}
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseWheel { delta, .. } => self.process_scroll(delta),
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => self.mouse_pressed = *state == ElementState::Pressed,
            _ => {}
        }
    }

    fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }
}
