use poms_common::limits::{MAX_ANIMATION_SPEED, MIN_ANIMATION_SPEED};

/// Controls the animation speed and state.
///
/// The user may select to upload multiple files at once, in which case the application will
/// periodically switch between rendering them (if the animation is active).
pub(crate) struct AnimationController {
    pub(crate) is_active: bool,
    /// The speed of the animation. The higher the value faster the animation.
    pub(crate) speed: u32,
    /// The current tick count. Each render frame increments this value. Do not confuse with animation frames.
    tick_count: u32,
}

const DEFAULT_ANIMATION_SPEED: u32 = 5;

impl Default for AnimationController {
    fn default() -> Self {
        Self::new(DEFAULT_ANIMATION_SPEED, false)
    }
}

impl AnimationController {
    /// Creates a new instance of `AnimationController`.
    pub fn new(speed: u32, is_active: bool) -> Self {
        assert!(
            (MIN_ANIMATION_SPEED..=MAX_ANIMATION_SPEED).contains(&speed),
            "Invalid animation speed"
        );
        Self {
            speed,
            is_active,
            tick_count: 0,
        }
    }

    /// Increments the tick count and returns `true` if the next frame is due.
    ///
    /// We need `TICKS_PER_ANIMATION_FRAME / self.speed` render frames (ticks) to advance to the next animation frame.
    pub(crate) fn advance_tick(&mut self) -> bool {
        const TICKS_PER_ANIMATION_FRAME: u32 = 40;
        if self.is_active {
            self.tick_count += 1;
            self.tick_count % (TICKS_PER_ANIMATION_FRAME / self.speed) == 0
        } else {
            false
        }
    }
}
