//! The player: MIDI events to per-sample bowing gestures.
//!
//! This layer owns everything a human player owns: dynamics, vibrato,
//! portamento, note transitions. The instrument (`violino-core`) only
//! vibrates. CC defaults follow SWAM Violin's factory MIDI mapping so the
//! same file can drive both engines:
//!
//! - CC11 expression -> bow velocity, with coupled bow pressure
//! - CC1  vibrato depth
//! - CC19 vibrato rate
//! - CC5  portamento time
//! - pitch bend, +/-2 semitones

use crate::timeline::TimedEvent;
use crate::smf::EventKind;
use violino_core::Violin;

pub struct Mapping {
    pub expression: u8,
    pub vibrato_depth: u8,
    pub vibrato_rate: u8,
    pub portamento_time: u8,
}

impl Default for Mapping {
    fn default() -> Self {
        Mapping { expression: 11, vibrato_depth: 1, vibrato_rate: 19, portamento_time: 5 }
    }
}

pub struct RenderOptions {
    pub sample_rate: f32,
    /// Seconds of silence-decay appended after the last event.
    pub tail: f32,
    pub mapping: Mapping,
    /// MIDI channel to play, or None for all channels (mono merge).
    pub channel: Option<u8>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions { sample_rate: 44100.0, tail: 1.5, mapping: Mapping::default(), channel: None }
    }
}

fn key_to_freq(key: u8) -> f32 {
    440.0 * ((key as f32 - 69.0) / 12.0).exp2()
}

/// One-pole smoother; time constant in seconds.
struct Smooth {
    value: f32,
    coeff: f32,
}

impl Smooth {
    fn new(initial: f32, tau: f32, sample_rate: f32) -> Self {
        Smooth { value: initial, coeff: (-1.0 / (tau * sample_rate)).exp() }
    }

    fn tick(&mut self, target: f32) -> f32 {
        self.value = target + (self.value - target) * self.coeff;
        self.value
    }
}

struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Render a merged, time-resolved event list to audio.
pub fn render(events: &[TimedEvent], opts: &RenderOptions) -> Vec<f32> {
    let sr = opts.sample_rate;
    let end = events.last().map_or(0.0, |e| e.time) as f32 + opts.tail;
    let total = (end * sr) as usize;

    let mut violin = Violin::new(sr);
    let mut noise = Noise(0x76696F6C); // "viol"

    // Raw controller state (0..1), updated at event boundaries.
    let mut expression: f32 = 0.75; // default so CC-less files still sound
    let mut vib_depth_raw: f32 = 0.0;
    let mut vib_rate_raw: f32 = 0.37; // ~5.3 Hz
    let mut portamento_raw: f32 = 0.0;
    let mut bend_semitones: f32 = 0.0;

    // Held notes, oldest first; last entry is the sounding note.
    let mut held: Vec<u8> = Vec::new();

    // Smoothed gesture signals.
    let mut f0 = Smooth::new(440.0, 0.02, sr);
    let mut velocity = Smooth::new(0.0, 0.015, sr);
    let mut pressure = Smooth::new(0.55, 0.03, sr);
    let mut vib_depth = Smooth::new(0.0, 0.08, sr);
    let mut vib_phase: f32 = 0.0;

    let mut out = vec![0.0f32; total];
    let mut ev = 0usize;
    for (i, sample) in out.iter_mut().enumerate() {
        let t = i as f64 / sr as f64;
        while ev < events.len() && events[ev].time <= t {
            match events[ev].kind {
                EventKind::NoteOn { channel, key, .. }
                    if opts.channel.is_none_or(|c| c == channel) =>
                {
                    held.retain(|&k| k != key);
                    held.push(key);
                }
                EventKind::NoteOff { channel, key }
                    if opts.channel.is_none_or(|c| c == channel) =>
                {
                    held.retain(|&k| k != key);
                }
                EventKind::ControlChange { channel, controller, value }
                    if opts.channel.is_none_or(|c| c == channel) =>
                {
                    let v = value as f32 / 127.0;
                    let m = &opts.mapping;
                    if controller == m.expression {
                        expression = v;
                    } else if controller == m.vibrato_depth {
                        vib_depth_raw = v;
                    } else if controller == m.vibrato_rate {
                        vib_rate_raw = v;
                    } else if controller == m.portamento_time {
                        portamento_raw = v;
                    }
                }
                EventKind::PitchBend { channel, value }
                    if opts.channel.is_none_or(|c| c == channel) =>
                {
                    bend_semitones = 2.0 * value as f32 / 8192.0;
                }
                _ => {}
            }
            ev += 1;
        }

        // Gesture targets from the current state. Real players raise bow
        // speed and pressure with register (a shorter string needs more of
        // both to lock into Helmholtz motion instead of surface sound an
        // octave up); without this, notes above ~C5 crack.
        let mut f0_target = f0.value; // bow lifted: pitch holds for the decay
        let (vel_target, pres_bonus) = match held.last() {
            Some(&key) => {
                let f = key_to_freq(key) * (bend_semitones / 12.0).exp2();
                // Schelleng: the minimum bow force for Helmholtz motion
                // rises both with register and with bow speed, so raise
                // pressure only — raising speed as well would move the
                // required force further away.
                let register = (f / 440.0).log2().max(0.0); // octaves above A4
                f0_target = f;
                (0.05 + 0.30 * expression, 0.25 * register)
            }
            None => (0.0, 0.0),
        };
        // Portamento widens the f0 smoothing time constant (20 ms..0.25 s).
        f0.coeff = (-1.0 / ((0.02 + 0.23 * portamento_raw) * sr)).exp();

        let vib_rate = 4.0 + 3.5 * vib_rate_raw;
        vib_phase += 2.0 * core::f32::consts::PI * vib_rate / sr;
        let vib = vib_depth.tick(0.012 * vib_depth_raw) * vib_phase.sin();

        let f = f0.tick(f0_target) * (1.0 + vib);
        let vel = velocity.tick(vel_target) * (1.0 + 0.008 * noise.next());
        let pres = pressure.tick((0.40 + 0.35 * expression + pres_bonus).min(0.95));
        *sample = violin.tick(f, vel, pres);
    }
    out
}
