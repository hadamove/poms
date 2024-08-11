pub const MAX_DISTANCE_FIELD_RESOLUTION: u32 = 256;
pub const MAX_NUM_GRID_POINTS: usize = u32::pow(MAX_DISTANCE_FIELD_RESOLUTION, 3) as usize;

pub const MIN_PROBE_RADIUS: f32 = 1.4;
pub const MAX_PROBE_RADIUS: f32 = 5.0;

pub const MAX_NUM_ATOMS: usize = 1_000_000;
