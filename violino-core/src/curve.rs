//! Breakpoint control curves: piecewise-linear (time, value) envelopes.
//!
//! These describe the player, not the instrument: bow velocity, bow
//! pressure, vibrato depth over the course of a phrase. The MIDI layer
//! (roadmap) generates the same kind of signal from note and CC streams.

pub struct Curve
{
    points: Vec<(f32, f32)>,
}

impl Curve
{
    /// `points` are (time seconds, value), sorted by time.
    pub fn new(points: Vec<(f32, f32)>) -> Self
    {
        Curve { points }
    }

    /// Linearly interpolated value at time `t`; clamps at both ends.
    pub fn at(&self, t: f32) -> f32
    {
        match self.points.iter().position(|&(pt, _)| pt > t)
        {
            None => self.points.last().map_or(0.0, |&(_, v)| v),
            Some(0) => self.points[0].1,
            Some(i) =>
            {
                let (t0, v0) = self.points[i - 1];
                let (t1, v1) = self.points[i];
                v0 + (v1 - v0) * (t - t0) / (t1 - t0)
            }
        }
    }
}
