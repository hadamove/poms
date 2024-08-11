use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
};

/// Represents mouse input, including scroll, button presses, and cursor movement.
#[derive(Debug, Default)]
pub struct MouseInput {
    pub scroll: f32,
    pub mouse_pressed: bool,
    pub mouse_delta: (f64, f64),
}

impl MouseInput {
    const ROTATION_SPEED: f64 = 3.0;
    const LINE_SCROLL_SPEED: f32 = 20.0;

    /// Instead of resetting the mouse input back to zero each frame,
    /// slowly decay the input values to zero to allow for smoother camera movement free of jitter.
    pub fn decay_input(&mut self) {
        let smooth = 0.8;
        self.scroll *= smooth as f32;
        self.mouse_delta = (self.mouse_delta.0 * smooth, self.mouse_delta.1 * smooth);
    }

    /// Handles window-related events, such as mouse input and scrolling.
    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel { delta, .. } => self.process_scroll(delta),
            // Updates mouse pressed state when the left mouse button is pressed/released
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => self.mouse_pressed = *state == ElementState::Pressed,
            _ => {}
        }
        false
    }

    /// Handles raw device events, such as direct mouse motion.
    ///
    /// We use `DeviceEvent` here because it provides raw, unfiltered data from the
    /// input device, which is crucial for precise control, such as rotating a 3D camera.
    ///
    /// Using `WindowEvent` for camera rotation is not ideal because `WindowEvent::CursorMoved`
    /// is influenced by the OS, including cursor acceleration and screen boundaries, which
    /// can lead to inconsistent and less precise camera movements.
    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            // Apply rotation speed scaling to raw mouse movement for camera control
            self.mouse_delta = (
                delta.0 * Self::ROTATION_SPEED,
                delta.1 * Self::ROTATION_SPEED,
            );
        }
    }

    /// Processes scroll input and adjusts the scroll amount based on delta.
    fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // Line scroll (e.g., using a mouse wheel)
            MouseScrollDelta::LineDelta(_, scroll) => scroll * Self::LINE_SCROLL_SPEED,
            // Pixel scroll (e.g., using a touchpad)
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }
}
