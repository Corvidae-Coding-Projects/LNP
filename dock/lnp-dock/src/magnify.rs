//! The magnification model.
//!
//! This deliberately does not copy the macOS Dock, for reasons that come
//! straight out of the pointing literature:
//!
//! * McGuffin and Balakrishnan ("Acquisition of expanding targets", and the
//!   later TOCHI paper) found that pointing performance follows a target's
//!   *final* size, and that expansion beginning as late as 90% of the way
//!   through the movement still delivers close to the full benefit.
//!
//! * The same work notes the catch: an expanding target can cover or displace
//!   its neighbours, and that penalty can cancel the gain. Gutwin's fisheye
//!   studies put numbers on it -- distortion that moves targets as the cursor
//!   approaches causes "hunting", costing both time and accuracy. His fix was
//!   speed-coupled flattening: damp the distortion by pointer velocity.
//!
//! Two design rules fall out, and this module implements both:
//!
//! 1. **Icon centres never move.** Slots are fixed width. Icons scale in place
//!    and are allowed to overlap visually. The target you aimed at is still
//!    exactly where it was when you started moving, so there is no hunting --
//!    but it is bigger when you arrive, which is where the Fitts benefit is.
//!
//! 2. **Magnification is damped while the pointer is travelling fast.** By the
//!    90% finding this costs almost nothing, and it keeps the dock visually
//!    still during the ballistic phase of the movement.

use std::time::{Duration, Instant};

/// Smoothstep easing: zero derivative at both ends, so magnification eases in
/// and out instead of arriving with a visible corner.
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Geometry and strength of the magnification effect.
#[derive(Debug, Clone, Copy)]
pub struct MagnifyParams {
    /// Width of one slot in logical pixels. Constant, and independent of the
    /// pointer -- this is what keeps centres fixed.
    pub slot_width: f32,
    /// Peak scale factor applied to an icon directly under the pointer.
    pub max_scale: f32,
    /// How far magnification reaches, in multiples of `slot_width`.
    pub radius_slots: f32,
}

impl Default for MagnifyParams {
    fn default() -> Self {
        Self {
            slot_width: 56.0,
            max_scale: 1.6,
            radius_slots: 2.0,
        }
    }
}

impl MagnifyParams {
    /// Centre of slot `index`, given the dock's left edge. Note this takes no
    /// pointer argument at all: slot centres are a pure function of geometry,
    /// which is the property that eliminates the hunting effect.
    pub fn slot_center(&self, left: f32, index: usize) -> f32 {
        left + self.slot_width * (index as f32 + 0.5)
    }

    /// Scale for a slot whose centre is at `slot_center`.
    ///
    /// `pointer` is `None` when the pointer is not over the dock, in which
    /// case everything rests at 1.0. `damping` in 0.0..=1.0 comes from
    /// [`SpeedDamping`] and scales the whole effect.
    pub fn scale_at(&self, slot_center: f32, pointer: Option<f32>, damping: f32) -> f32 {
        let Some(px) = pointer else { return 1.0 };

        let radius = self.radius_slots * self.slot_width;
        if radius <= 0.0 || self.max_scale <= 1.0 {
            return 1.0;
        }

        let distance = (px - slot_center).abs();
        let influence = smoothstep(1.0 - distance / radius);

        1.0 + (self.max_scale - 1.0) * influence * damping.clamp(0.0, 1.0)
    }
}

/// Speed-coupled damping, after Gutwin's speed-coupled flattening.
///
/// Tracks horizontal pointer velocity and produces a damping factor: 1.0 when
/// the pointer is settled (magnify fully), falling to 0.0 while it is moving
/// quickly (stay flat and still). The output is smoothed so the dock does not
/// visibly snap between the two states.
#[derive(Debug)]
pub struct SpeedDamping {
    last: Option<(f32, Instant)>,
    /// Smoothed damping value actually applied.
    damping: f32,
    /// At or below this speed (logical px per ms) magnification is full.
    pub settled_speed: f32,
    /// At or above this speed magnification is fully suppressed.
    pub flat_speed: f32,
    /// Time constant of the smoothing, in milliseconds.
    pub smoothing_ms: f32,
}

impl Default for SpeedDamping {
    fn default() -> Self {
        Self {
            last: None,
            // Start settled: if the pointer enters and stops, magnify at once.
            damping: 1.0,
            settled_speed: 0.35,
            flat_speed: 1.8,
            smoothing_ms: 90.0,
        }
    }
}

impl SpeedDamping {
    /// Feed a pointer sample. Returns the damping factor to use right now.
    pub fn update(&mut self, x: f32, now: Instant) -> f32 {
        let target = match self.last {
            Some((last_x, last_t)) => {
                let dt = now.duration_since(last_t).as_secs_f32() * 1000.0;
                if dt <= 0.0 {
                    // Two samples in the same instant carry no velocity
                    // information; keep whatever we last decided.
                    self.damping
                } else {
                    let speed = (x - last_x).abs() / dt;
                    Self::damping_for_speed(speed, self.settled_speed, self.flat_speed)
                }
            }
            // First sample after entering: no velocity yet, assume settled.
            None => 1.0,
        };

        let dt_ms = self
            .last
            .map(|(_, t)| now.duration_since(t).as_secs_f32() * 1000.0)
            .unwrap_or(0.0);

        // Exponential smoothing toward the target damping.
        let alpha = if self.smoothing_ms <= 0.0 {
            1.0
        } else {
            (dt_ms / self.smoothing_ms).clamp(0.0, 1.0)
        };
        self.damping += (target - self.damping) * alpha;
        self.damping = self.damping.clamp(0.0, 1.0);

        self.last = Some((x, now));
        self.damping
    }

    /// Called when the pointer stops producing motion events. Without this the
    /// dock would stay flat after a fast flick that ends on the dock, which is
    /// precisely the case where the user has arrived and wants the target big.
    pub fn settle(&mut self, now: Instant) -> f32 {
        if let Some((x, _)) = self.last {
            self.last = Some((x, now));
        }
        self.damping += (1.0 - self.damping) * 0.5;
        self.damping = self.damping.clamp(0.0, 1.0);
        self.damping
    }

    /// Pointer left the dock entirely.
    pub fn reset(&mut self) {
        self.last = None;
        self.damping = 1.0;
    }

    pub fn damping(&self) -> f32 {
        self.damping
    }

    fn damping_for_speed(speed: f32, settled: f32, flat: f32) -> f32 {
        if speed <= settled {
            1.0
        } else if speed >= flat {
            0.0
        } else {
            1.0 - smoothstep((speed - settled) / (flat - settled))
        }
    }
}

/// How long to wait with no motion before treating the pointer as settled.
pub const SETTLE_AFTER: Duration = Duration::from_millis(60);

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> MagnifyParams {
        MagnifyParams {
            slot_width: 50.0,
            max_scale: 2.0,
            radius_slots: 2.0,
        }
    }

    /// The property the whole design rests on: slot centres are independent of
    /// where the pointer is. If this ever fails, the dock has become a fisheye
    /// and has reintroduced the hunting effect.
    #[test]
    fn slot_centres_do_not_depend_on_pointer() {
        let p = params();
        let before: Vec<f32> = (0..5).map(|i| p.slot_center(0.0, i)).collect();
        // Nothing in the API even permits the pointer to influence this, but
        // assert the values explicitly so a refactor cannot quietly change it.
        assert_eq!(before, vec![25.0, 75.0, 125.0, 175.0, 225.0]);
    }

    #[test]
    fn hovered_slot_reaches_max_scale() {
        let p = params();
        let c = p.slot_center(0.0, 2);
        assert!((p.scale_at(c, Some(c), 1.0) - 2.0).abs() < 1e-4);
    }

    #[test]
    fn distant_slot_is_unscaled() {
        let p = params();
        let c = p.slot_center(0.0, 0);
        // Pointer four slots away, radius is two.
        assert!((p.scale_at(c, None, 1.0) - 1.0).abs() < 1e-6);
        assert!((p.scale_at(c, Some(c + 4.0 * 50.0), 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn scale_decreases_monotonically_with_distance() {
        let p = params();
        let c = 100.0;
        let mut prev = f32::INFINITY;
        for step in 0..20 {
            let s = p.scale_at(c, Some(c + step as f32 * 5.0), 1.0);
            assert!(s <= prev + 1e-6, "scale increased with distance");
            prev = s;
        }
    }

    #[test]
    fn damping_suppresses_magnification() {
        let p = params();
        let c = p.slot_center(0.0, 1);
        let hot = p.scale_at(c, Some(c), 1.0);
        let cold = p.scale_at(c, Some(c), 0.0);
        assert!(hot > cold);
        assert!((cold - 1.0).abs() < 1e-6, "zero damping must mean no magnification");
    }

    #[test]
    fn fast_pointer_damps_toward_flat() {
        // A fast flick across the dock should drive damping down.
        let mut d = SpeedDamping::default();
        let t0 = Instant::now();
        d.update(0.0, t0);
        let mut t = t0;
        let mut x = 0.0;
        for _ in 0..20 {
            t += Duration::from_millis(8);
            x += 40.0; // 5 px/ms, well above flat_speed
            d.update(x, t);
        }
        assert!(d.damping() < 0.2, "damping was {}", d.damping());
    }

    #[test]
    fn slow_pointer_keeps_magnification() {
        let mut d = SpeedDamping::default();
        let t0 = Instant::now();
        d.update(0.0, t0);
        let mut t = t0;
        let mut x = 0.0;
        for _ in 0..20 {
            t += Duration::from_millis(16);
            x += 1.0; // 0.0625 px/ms, well below settled_speed
            d.update(x, t);
        }
        assert!(d.damping() > 0.9, "damping was {}", d.damping());
    }

    #[test]
    fn settling_after_a_flick_restores_magnification() {
        let mut d = SpeedDamping::default();
        let t0 = Instant::now();
        d.update(0.0, t0);
        let mut t = t0;
        let mut x = 0.0;
        for _ in 0..20 {
            t += Duration::from_millis(8);
            x += 40.0;
            d.update(x, t);
        }
        assert!(d.damping() < 0.2);

        // Pointer stops. This is the arrival case -- the user is now on the
        // dock and wants the target large.
        for _ in 0..8 {
            t += SETTLE_AFTER;
            d.settle(t);
        }
        assert!(d.damping() > 0.9, "damping was {}", d.damping());
    }
}
