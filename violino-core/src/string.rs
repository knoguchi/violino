//! Digital waveguide bowed string (Smith 1992).
//!
//! Two delay lines carry the transverse velocity waves on either side of the
//! bow point; the bow injects velocity through the friction nonlinearity;
//! nut and bridge reflect with sign inversion, the bridge side through a
//! lossy one-pole (lumped string losses).

use crate::bow::bow_friction;
use crate::dsp::{Delay, OnePole};

/// Loop delay contributed by the interpolation and the loop filter,
/// subtracted from the total so the pitch lands on `f0`.
const LOOP_DELAY_COMPENSATION: f32 = 4.0;

pub struct BowedString
{
    sample_rate: f32,
    /// Bow position as a fraction of string length from the bridge.
    /// Physical meaning: sul ponticello (small) .. sul tasto (large).
    pub beta: f32,
    neck: Delay,
    bridge: Delay,
    string_filter: OnePole,
}

impl BowedString
{
    pub fn new(sample_rate: f32) -> Self
    {
        BowedString
        {
            sample_rate,
            beta: 0.127,
            neck: Delay::new(4096),
            bridge: Delay::new(4096),
            string_filter: OnePole::new(0.6, 0.95),
        }
    }

    /// One sample. `f0` in Hz, `bow_velocity` roughly 0..0.4,
    /// `bow_pressure` 0..1. Returns bridge output (pre-body).
    pub fn tick(&mut self, f0: f32, bow_velocity: f32, bow_pressure: f32) -> f32
    {
        let total = (self.sample_rate / f0 - LOOP_DELAY_COMPENSATION).max(4.0);
        let bridge_len = (total * self.beta).max(2.0);
        let neck_len = (total * (1.0 - self.beta)).max(2.0);
        let slope = 5.0 - 4.0 * bow_pressure;

        let bridge_refl = -self.string_filter.tick(self.bridge.out);
        let nut_refl = -self.neck.out;
        let string_vel = bridge_refl + nut_refl;
        let dv = bow_velocity - string_vel;
        let new_vel = dv * bow_friction(dv, slope);

        self.neck.tick(bridge_refl + new_vel, neck_len);
        self.bridge.tick(nut_refl + new_vel, bridge_len);
        self.bridge.out
    }
}
