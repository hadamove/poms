pub struct AnimationController {
    pub is_active: bool,
    pub speed: u32,
    tick_count: u32,
}

impl AnimationController {
    pub fn new(speed: u32, is_active: bool) -> Self {
        Self {
            speed,
            is_active,
            tick_count: 0,
        }
    }

    /// Increments the tick count and returns `true` if the next frame is due.
    pub fn advance_tick(&mut self) -> bool {
        if self.is_active {
            self.tick_count += 1;
            self.is_frame_due()
        } else {
            false
        }
    }

    /// Checks if the current tick is due for advancement based on the speed.
    pub fn is_frame_due(&self) -> bool {
        const TICKS_PER_ANIMATION_FRAME: u32 = 40;
        self.is_active && self.tick_count % (TICKS_PER_ANIMATION_FRAME / self.speed) == 0
    }
}
