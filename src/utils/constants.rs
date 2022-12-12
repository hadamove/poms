pub const DEFAULT_SES_RESOLUTION: u32 = 64;
pub const MAX_SES_RESOLUTION: u32 = 256;

pub const MAX_NUM_ATOMS: usize = 1_000_000;
pub const MAX_NUM_GRID_POINTS: usize = u32::pow(MAX_SES_RESOLUTION, 3) as usize;

pub const DEFAULT_PROBE_RADIUS: f32 = 1.4;
pub const MAX_PROBE_RADIUS: f32 = 5.0;

pub const DEFAULT_LIGHT_DIRECTION: [f32; 3] = [0.0, 0.0, 1.0];
pub const DEFAULT_LIGHT_COLOR: [f32; 3] = [1.0, 0.7, 0.7];

pub const ANIMATION_ACTIVE_BY_DEFAULT: bool = true;
pub const DEFAULT_ANIMATION_SPEED: u32 = 5;

pub enum ColorTheme {
    Light,
    Dark,
}
