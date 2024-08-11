use super::events::UserEvent;

/// Struct that represents an error message that should be displayed to the user.
pub struct ErrorMessage {
    pub id: uuid::Uuid,
    pub message: String,
}

/// Struct that holds current state of the UI.
/// Also used to store dispatched events that are collected by the main app loop.
#[derive(Default)]
pub struct UIState {
    /// Resolution of the distance field used for molecular surface rendering.
    pub target_resolution: u32,
    /// Probe radius used for molecular surface rendering.
    pub probe_radius: f32,
    /// Flag that indicates if spacefill pass should be rendered.
    pub render_spacefill: bool,
    /// Flag that indicates if molecular surface pass should be rendered.
    pub render_molecular_surface: bool,
    /// Flag that indicates if animation is active.
    pub is_animation_active: bool,
    /// Speed of the animation.
    pub animation_speed: u32,
    /// List of error messages that should be displayed.
    pub error_messages: Vec<ErrorMessage>,

    /// List of events that were dispatched by the UI.
    pub events: Vec<UserEvent>,
}

impl UIState {
    /// Dispatches an event to the UI.
    pub fn dispatch_event(&mut self, event: UserEvent) {
        self.events.push(event);
    }

    /// Collects dispatched events and clears the list of events.
    /// Call this method at the end of the frame to collect all events that were dispatched during the frame.
    pub fn collect_events(&mut self) -> Vec<UserEvent> {
        self.events.drain(..).collect()
    }

    /// Adds a new error message to the list of error messages.
    /// Call this method to display an error message to the user.
    pub fn open_error_message(&mut self, message: String) {
        self.error_messages.push(ErrorMessage {
            id: uuid::Uuid::new_v4(),
            message,
        });
    }
}
