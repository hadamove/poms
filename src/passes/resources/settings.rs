use crate::utils::constants::{DEFAULT_PROBE_RADIUS, DEFAULT_SES_RESOLUTION};

pub struct SesSettings {
    pub probe_radius: f32,
    pub resolution: u32,
}

impl Default for SesSettings {
    fn default() -> Self {
        Self {
            probe_radius: DEFAULT_PROBE_RADIUS,
            resolution: DEFAULT_SES_RESOLUTION,
        }
    }
}
