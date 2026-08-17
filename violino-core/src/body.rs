//! Instrument body as a bank of parallel resonant modes plus a dry path.
//!
//! The mode table below is hand-tuned placeholder data: one air mode around
//! 275 Hz and a handful of wood modes. The roadmap replaces this with modes
//! fitted to measured violin bridge admittance / impulse responses, which is
//! where most of the realism budget lives.

use crate::dsp::Biquad;

pub struct Body
{
    modes: Vec<(Biquad, f32)>,
    dry: f32,
}

impl Body
{
    pub fn violin(sample_rate: f32) -> Self
    {
        let table: &[(f32, f32, f32)] = &[
            // (freq Hz, Q, gain)
            (275.0, 11.0, 1.5),  // A0 air mode
            (460.0, 14.0, 1.1),
            (700.0, 9.0, 0.7),
            (1050.0, 8.0, 0.6),
            (1600.0, 7.0, 0.35),
            (2800.0, 6.0, 0.2),
        ];
        let modes = table
            .iter()
            .map(|&(f, q, g)| (Biquad::bandpass(sample_rate, f, q), g))
            .collect();
        Body { modes, dry: 0.25 }
    }

    pub fn tick(&mut self, x: f32) -> f32
    {
        let mut y = self.dry * x;
        for (filter, gain) in &mut self.modes
        {
            y += *gain * filter.tick(x);
        }
        y
    }
}
