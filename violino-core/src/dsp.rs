//! Small DSP building blocks: fractional delay, one-pole lowpass, biquad.

/// Fractional delay line with linear interpolation.
pub struct Delay
{
    buf: Vec<f32>,
    w: usize,
    /// Most recent interpolated output.
    pub out: f32,
}

impl Delay
{
    pub fn new(max_len: usize) -> Self
    {
        Delay { buf: vec![0.0; max_len], w: 0, out: 0.0 }
    }

    /// Push one sample and read `delay` samples into the past.
    pub fn tick(&mut self, x: f32, delay: f32) -> f32
    {
        let n = self.buf.len();
        self.buf[self.w] = x;
        let mut r = self.w as f32 - delay;
        while r < 0.0
        {
            r += n as f32;
        }
        let mut i = r as usize;
        if i >= n
        {
            i -= n;
        }
        let frac = r - r.floor();
        let j = if i + 1 >= n { 0 } else { i + 1 };
        self.out = self.buf[i] * (1.0 - frac) + self.buf[j] * frac;
        self.w = (self.w + 1) % n;
        self.out
    }
}

/// One-pole lowpass; models the lumped losses of the string per round trip.
pub struct OnePole
{
    b0: f32,
    a1: f32,
    y1: f32,
}

impl OnePole
{
    pub fn new(pole: f32, gain: f32) -> Self
    {
        OnePole { b0: gain * (1.0 - pole), a1: -pole, y1: 0.0 }
    }

    pub fn tick(&mut self, x: f32) -> f32
    {
        let y = self.b0 * x - self.a1 * self.y1;
        self.y1 = y;
        y
    }
}

/// Direct-form-I biquad. Constructor provided for RBJ constant-peak bandpass,
/// used as one resonant mode of the instrument body.
pub struct Biquad
{
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad
{
    /// RBJ cookbook bandpass, constant peak gain.
    pub fn bandpass(sample_rate: f32, freq: f32, q: f32) -> Self
    {
        let w0 = 2.0 * core::f32::consts::PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let a0 = 1.0 + alpha;
        Biquad
        {
            b0: alpha / a0,
            b1: 0.0,
            b2: -alpha / a0,
            a1: -2.0 * w0.cos() / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn tick(&mut self, x: f32) -> f32
    {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}
