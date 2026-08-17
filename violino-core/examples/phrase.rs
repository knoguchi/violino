//! Render a short legato phrase (A3-B3-C#4-D4-E4) to `violino_phrase.wav`.
//!
//! The control curves below are the "player": hand-written bow velocity,
//! pressure, and vibrato envelopes. Replacing this file with a MIDI-driven
//! player is the top roadmap item.

use violino_core::{Curve, Violin};

const SR: f32 = 44100.0;

/// xorshift32; deterministic bow noise without pulling in a dependency.
struct Noise(u32);

impl Noise
{
    fn next(&mut self) -> f32
    {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn write_wav(path: &str, sample_rate: u32, samples: &[f32]) -> std::io::Result<()>
{
    use std::io::Write;
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for &s in samples
    {
        pcm.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    let data_len = pcm.len() as u32;
    let mut f = std::fs::File::create(path)?;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVE")?;
    f.write_all(b"fmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&1u16.to_le_bytes())?; // mono
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&(sample_rate * 2).to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    f.write_all(&pcm)
}

fn main()
{
    let (a3, b3, cs4, d4, e4) = (220.0, 246.94, 277.18, 293.66, 329.63);
    let dur = 7.0;
    let g = 0.05; // legato glide time

    let f0 = Curve::new(vec![
        (0.0, a3),
        (1.0, a3),
        (1.0 + g, b3),
        (1.8, b3),
        (1.8 + g, cs4),
        (2.6, cs4),
        (2.6 + g, d4),
        (3.4, d4),
        (3.4 + g, e4),
        (dur, e4),
    ]);
    // Bow velocity: the phrase dynamics, with a swell and release at the end.
    let velocity = Curve::new(vec![
        (0.0, 0.0),
        (0.08, 0.22),
        (1.0, 0.25),
        (1.8, 0.24),
        (2.6, 0.27),
        (3.4, 0.26),
        (4.6, 0.36),
        (5.8, 0.30),
        (6.6, 0.10),
        (dur, 0.0),
    ]);
    let pressure = Curve::new(vec![
        (0.0, 0.55),
        (3.4, 0.55),
        (4.6, 0.70),
        (6.2, 0.45),
        (dur, 0.30),
    ]);
    // Vibrato restarts on each note and deepens on the held final note.
    let vib_depth = Curve::new(vec![
        (0.0, 0.0),
        (0.3, 0.0),
        (0.9, 0.004),
        (1.05, 0.0),
        (1.6, 0.003),
        (1.85, 0.0),
        (2.4, 0.003),
        (2.65, 0.0),
        (3.2, 0.003),
        (3.45, 0.0),
        (4.2, 0.008),
        (5.5, 0.011),
        (dur, 0.008),
    ]);

    let n = (SR * dur) as usize;
    let mut violin = Violin::new(SR);
    let mut noise = Noise(0x20260816);
    let mut out = vec![0.0f32; n];
    let mut vib_phase = 0.0f32;
    for (i, sample) in out.iter_mut().enumerate()
    {
        let t = i as f32 / SR;
        let vib_rate = 5.3 + 0.15 * (2.0 * core::f32::consts::PI * 0.7 * t).sin();
        vib_phase += 2.0 * core::f32::consts::PI * vib_rate / SR;
        let f = f0.at(t) * (1.0 + vib_depth.at(t) * vib_phase.sin());
        let vel = velocity.at(t) * (1.0 + 0.008 * noise.next());
        *sample = violin.tick(f, vel, pressure.at(t));
    }

    let peak = out.iter().fold(0.0f32, |m, &x| m.max(x.abs()));
    for s in &mut out
    {
        *s *= 0.7 / peak;
    }
    let path = "violino_phrase.wav";
    write_wav(path, SR as u32, &out).expect("failed to write wav");
    println!("wrote {path}: {dur}s, peak normalized to 0.7");
}
