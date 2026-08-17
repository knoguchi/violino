//! violino-core: a physical model of bowed strings.
//!
//! Headless, dependency-free, sample-by-sample. The instrument is a pure
//! function: control signals in, audio samples out. No UI, no plugin
//! framework, no allocation on the audio path after construction.
//!
//! Structure:
//!
//! ```text
//! excitation (bow friction) -> string (waveguide) -> body (modal resonators)
//! ```
//!
//! Theory this implementation is built on (see README for full citations):
//! - McIntyre / Schumacher / Woodhouse (1983): stick-slip friction excitation
//! - Smith (1992): digital waveguide string
//! - Serafin (2004): real-time bowed-string models
//! - Initial constants reference STK's `Bowed` (Cook & Scavone)

pub mod body;
pub mod bow;
pub mod curve;
pub mod dsp;
pub mod string;

pub use body::Body;
pub use bow::bow_friction;
pub use curve::Curve;
pub use dsp::{Biquad, Delay, OnePole};
pub use string::BowedString;

/// A complete violin voice: bowed string plus body resonators.
pub struct Violin
{
    pub string: BowedString,
    pub body: Body,
}

impl Violin
{
    pub fn new(sample_rate: f32) -> Self
    {
        Violin
        {
            string: BowedString::new(sample_rate),
            body: Body::violin(sample_rate),
        }
    }

    /// One sample of output for the given control inputs.
    ///
    /// `f0` is the target fundamental in Hz (vibrato and portamento are the
    /// caller's job: they are properties of the player, not the instrument).
    /// `bow_velocity` is roughly 0..0.4, `bow_pressure` 0..1.
    pub fn tick(&mut self, f0: f32, bow_velocity: f32, bow_pressure: f32) -> f32
    {
        let raw = self.string.tick(f0, bow_velocity, bow_pressure);
        self.body.tick(raw)
    }
}
