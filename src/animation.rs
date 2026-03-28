//! Animation engine: easing functions, keyframe animation, and timelines.
//!
//! Equivalent to LVGL's `lv_anim` subsystem.  All values are represented as
//! `i32` so the same engine drives position, size, color-channel, and opacity
//! changes.  Time is expressed in milliseconds (u32).

// ─── Easing functions ─────────────────────────────────────────────────────────

/// An easing function maps a normalised time `t` (0 = start, 255 = end) to a
/// normalised progress value (0 = start value, 255 = end value).
pub type EasingFn = fn(t: u8) -> u8;

/// Built-in easing functions.
pub mod easing {
    /// No easing: constant rate of change.
    pub fn linear(t: u8) -> u8 {
        t
    }

    /// Quadratic ease-in: accelerate from zero.
    pub fn ease_in(t: u8) -> u8 {
        ((t as u16 * t as u16) / 255) as u8
    }

    /// Quadratic ease-out: decelerate to zero.
    pub fn ease_out(t: u8) -> u8 {
        let inv = 255u16 - t as u16;
        (255 - (inv * inv / 255)) as u8
    }

    /// Quadratic ease-in-out: accelerate then decelerate.
    pub fn ease_in_out(t: u8) -> u8 {
        if t < 128 {
            ((2u16 * t as u16 * t as u16) / 255) as u8
        } else {
            let t = t as u16;
            let v = 2 * (255 - t) * (255 - t) / 255;
            (255 - v) as u8
        }
    }

    /// Cubic ease-in.
    pub fn ease_in_cubic(t: u8) -> u8 {
        let t16 = t as u32;
        (t16 * t16 * t16 / (255 * 255)) as u8
    }

    /// Cubic ease-out.
    pub fn ease_out_cubic(t: u8) -> u8 {
        let inv = 255u32 - t as u32;
        (255 - inv * inv * inv / (255 * 255)) as u8
    }

    /// Sine-based ease-in-out (integer approximation using a 16-point table).
    pub fn ease_sine(t: u8) -> u8 {
        // 16-entry quarter-sine table scaled to [0, 255].
        const QTABLE: [u8; 17] = [
            0, 25, 49, 74, 98, 121, 142, 162, 180, 196, 210, 222, 231, 239, 244, 248, 255,
        ];
        // Map t (0–255) → index into the half-sine (0–32 ≈ 0–180°).
        let idx = (t as usize * 16 / 255).min(16);
        QTABLE[idx]
    }
}

// ─── Animation ────────────────────────────────────────────────────────────────

/// Repeat behaviour for an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    /// Play once and stop.
    Once,
    /// Play `n` times and stop.
    Count(u16),
    /// Loop forever.
    Infinite,
}

/// The current playback state of an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimState {
    Idle,
    Running,
    Paused,
    Completed,
}

/// A single-value animation from `start_value` to `end_value` over `duration`
/// milliseconds.
///
/// Call `tick(dt_ms)` each frame to advance the animation and read the current
/// interpolated value from `current_value()`.
pub struct Animation {
    pub start_value: i32,
    pub end_value: i32,
    pub duration: u32,
    pub elapsed: u32,
    pub easing: EasingFn,
    pub repeat: Repeat,
    pub state: AnimState,
    /// Number of full repetitions completed.
    pub repeat_count: u16,
    /// When `true` the animation plays forward then backward (ping-pong).
    pub auto_reverse: bool,
}

impl Animation {
    /// Create a new animation.
    pub fn new(start_value: i32, end_value: i32, duration: u32) -> Self {
        Self {
            start_value,
            end_value,
            duration,
            elapsed: 0,
            easing: easing::linear,
            repeat: Repeat::Once,
            state: AnimState::Idle,
            repeat_count: 0,
            auto_reverse: false,
        }
    }

    /// Set the easing function.
    pub fn with_easing(mut self, easing: EasingFn) -> Self {
        self.easing = easing;
        self
    }

    /// Set the repeat behaviour.
    pub fn with_repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    /// Enable ping-pong (forward → backward) looping.
    pub fn with_auto_reverse(mut self, enabled: bool) -> Self {
        self.auto_reverse = enabled;
        self
    }

    /// Start (or restart) the animation.
    pub fn start(&mut self) {
        self.elapsed = 0;
        self.repeat_count = 0;
        self.state = AnimState::Running;
    }

    /// Pause a running animation.
    pub fn pause(&mut self) {
        if self.state == AnimState::Running {
            self.state = AnimState::Paused;
        }
    }

    /// Resume a paused animation.
    pub fn resume(&mut self) {
        if self.state == AnimState::Paused {
            self.state = AnimState::Running;
        }
    }

    /// Advance the animation by `dt_ms` milliseconds.
    ///
    /// Returns the current interpolated value.
    pub fn tick(&mut self, dt_ms: u32) -> i32 {
        if self.state != AnimState::Running || self.duration == 0 {
            return self.current_value();
        }

        self.elapsed += dt_ms;

        if self.elapsed >= self.duration {
            // One full cycle completed.
            self.repeat_count += 1;

            let done = match self.repeat {
                Repeat::Once => true,
                Repeat::Count(n) => self.repeat_count >= n,
                Repeat::Infinite => false,
            };

            if done {
                // Pin elapsed at duration so current_value() always returns end_value.
                self.elapsed = self.duration;
                self.state = AnimState::Completed;
                return if self.auto_reverse { self.start_value } else { self.end_value };
            }

            // Start the next cycle.
            self.elapsed = self.elapsed.saturating_sub(self.duration);
        }

        self.current_value()
    }

    /// Compute the current interpolated value without advancing the clock.
    pub fn current_value(&self) -> i32 {
        if self.duration == 0 {
            return self.end_value;
        }

        // Normalise elapsed to 0–255.
        let t_raw = ((self.elapsed as u64 * 255 / self.duration as u64).min(255)) as u8;

        // Apply ping-pong if requested: first half forward, second half backward.
        let t = if self.auto_reverse {
            if t_raw < 128 {
                t_raw * 2
            } else {
                255 - (t_raw - 128) * 2
            }
        } else {
            t_raw
        };

        let progress = (self.easing)(t);
        let range = self.end_value - self.start_value;
        self.start_value + (range * progress as i32) / 255
    }

    /// `true` when the animation has finished all repetitions.
    pub fn is_completed(&self) -> bool {
        self.state == AnimState::Completed
    }
}

// ─── Timeline ─────────────────────────────────────────────────────────────────

/// A simple animation timeline that runs up to 4 animations in parallel.
pub struct Timeline {
    animations: heapless::Vec<Animation, 4>,
}

impl Timeline {
    pub fn new() -> Self {
        Self { animations: heapless::Vec::new() }
    }

    /// Add an animation to the timeline. Returns `Err(())` when full.
    pub fn add(&mut self, anim: Animation) -> Result<(), ()> {
        self.animations.push(anim).map_err(|_| ())
    }

    /// Tick all animations and return a slice reference to them.
    pub fn tick(&mut self, dt_ms: u32) {
        for anim in self.animations.iter_mut() {
            anim.tick(dt_ms);
        }
    }

    /// Start all animations.
    pub fn start_all(&mut self) {
        for anim in self.animations.iter_mut() {
            anim.start();
        }
    }

    /// `true` when every animation in the timeline has completed.
    pub fn all_completed(&self) -> bool {
        self.animations.iter().all(|a| a.is_completed())
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Re-export the `easing` module so callers can do `ferrogl::animation::easing::linear`.
pub use easing::linear as linear_easing;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_easing() {
        assert_eq!(easing::linear(0), 0);
        assert_eq!(easing::linear(128), 128);
        assert_eq!(easing::linear(255), 255);
    }

    #[test]
    fn test_ease_in_monotone() {
        let mut prev = 0u8;
        for t in 0u8..=255 {
            let v = easing::ease_in(t);
            assert!(v >= prev, "ease_in should be non-decreasing at t={t}");
            prev = v;
        }
    }

    #[test]
    fn test_ease_out_monotone() {
        let mut prev = 0u8;
        for t in 0u8..=255 {
            let v = easing::ease_out(t);
            assert!(v >= prev, "ease_out should be non-decreasing at t={t}");
            prev = v;
        }
    }

    #[test]
    fn test_animation_start_end_values() {
        let mut anim = Animation::new(0, 100, 1000).with_easing(easing::linear);
        anim.start();
        // At t=0 → start value.
        assert_eq!(anim.current_value(), 0);
        // Advance to end.
        anim.tick(1000);
        assert_eq!(anim.current_value(), 100);
        assert!(anim.is_completed());
    }

    #[test]
    fn test_animation_midpoint() {
        let mut anim = Animation::new(0, 200, 1000).with_easing(easing::linear);
        anim.start();
        let v = anim.tick(500);
        // Linear at 50% → ~100.
        assert!((v - 100).abs() <= 2, "Expected ~100 at midpoint, got {v}");
    }

    #[test]
    fn test_animation_repeat_count() {
        let mut anim =
            Animation::new(0, 10, 100).with_repeat(Repeat::Count(3)).with_easing(easing::linear);
        anim.start();
        // After 3 full cycles the animation should be done.
        anim.tick(100); // cycle 1
        anim.tick(100); // cycle 2
        anim.tick(100); // cycle 3
        assert!(anim.is_completed());
    }

    #[test]
    fn test_timeline_all_completed() {
        let mut tl = Timeline::new();
        let mut a = Animation::new(0, 10, 100);
        let mut b = Animation::new(0, 20, 200);
        a.start();
        b.start();
        tl.add(a).unwrap();
        tl.add(b).unwrap();

        tl.tick(200);
        // Both animations should have completed (200ms ≥ both durations).
        assert!(tl.all_completed());
    }
}
