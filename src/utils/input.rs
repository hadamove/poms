use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
};

#[derive(Debug, Default)]
pub struct Input {
    pub scroll: f32,
    pub mouse_pressed: bool,
    pub last_mouse_position: (f64, f64),
    pub mouse_delta: (f64, f64),
}

impl Input {
    pub fn handle_winit_event<T>(&mut self, event: &Event<T>) {
        if let Event::WindowEvent { event, .. } = event {
            self.handle_window_event(event);
        }
    }

    pub fn reset(&mut self) {
        self.mouse_delta = (0.0, 0.0);
        self.scroll = 0.0;
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseWheel { delta, .. } => self.process_scroll(delta),
            WindowEvent::CursorMoved { position, .. } => self.process_cursor(*position),
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
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 20.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    fn process_cursor(&mut self, position: PhysicalPosition<f64>) {
        let delta = (
            position.x - self.last_mouse_position.0,
            position.y - self.last_mouse_position.1,
        );
        if delta != (0.0, 0.0) {
            self.mouse_delta = delta;
        }
        self.last_mouse_position = position.into();
    }
}
