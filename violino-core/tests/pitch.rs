//! The one property that separates a bowed-string model from a noise
//! generator: steady bowing must lock into periodic stick-slip (Helmholtz)
//! motion at the target fundamental.

use violino_core::BowedString;

fn dominant_period(seg: &[f32], min_lag: usize, max_lag: usize) -> usize
{
    let mut best = (min_lag, f32::MIN);
    for lag in min_lag..max_lag
    {
        let mut acc = 0.0;
        for i in 0..seg.len() - lag
        {
            acc += seg[i] * seg[i + lag];
        }
        if acc > best.1
        {
            best = (lag, acc);
        }
    }
    best.0
}

#[test]
fn steady_bow_oscillates_at_f0()
{
    let sr = 44100.0;
    let f0 = 220.0;
    let mut string = BowedString::new(sr);
    let n = sr as usize;
    let mut out = vec![0.0f32; n];
    for (i, sample) in out.iter_mut().enumerate()
    {
        // ramp the bow in over ~45 ms, then hold
        let vel = 0.25 * (i as f32 / 2000.0).min(1.0);
        *sample = string.tick(f0, vel, 0.55);
    }

    let tail = &out[n / 2..];
    let rms = (tail.iter().map(|x| x * x).sum::<f32>() / tail.len() as f32).sqrt();
    assert!(rms > 0.01, "string did not build up oscillation, rms = {rms}");

    let lag = dominant_period(tail, 40, 400);
    let measured = sr / lag as f32;
    assert!(
        (measured - f0).abs() < 5.0,
        "expected ~{f0} Hz, measured {measured} Hz (lag {lag})"
    );
}
