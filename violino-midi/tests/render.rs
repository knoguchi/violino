//! End to end: bytes of a minimal SMF in, audio at the right pitch out.

use violino_midi::{parse, render, to_timeline, RenderOptions};

/// Format 0, 480 tpq: CC11=100, NoteOn A4, 2 beats, NoteOff, EOT.
fn minimal_smf() -> Vec<u8> {
    let track: &[u8] = &[
        0x00, 0xB0, 0x0B, 0x64, // CC11 = 100
        0x00, 0x90, 0x45, 0x40, // NoteOn A4 vel 64
        0x87, 0x40, 0x80, 0x45, 0x00, // delta 960, NoteOff
        0x00, 0xFF, 0x2F, 0x00, // end of track
    ];
    let mut f = Vec::new();
    f.extend_from_slice(b"MThd");
    f.extend_from_slice(&6u32.to_be_bytes());
    f.extend_from_slice(&0u16.to_be_bytes());
    f.extend_from_slice(&1u16.to_be_bytes());
    f.extend_from_slice(&480u16.to_be_bytes());
    f.extend_from_slice(b"MTrk");
    f.extend_from_slice(&(track.len() as u32).to_be_bytes());
    f.extend_from_slice(track);
    f
}

fn dominant_period(seg: &[f32], min_lag: usize, max_lag: usize) -> usize {
    let mut best = (min_lag, f32::MIN);
    for lag in min_lag..max_lag {
        let mut acc = 0.0;
        for i in 0..seg.len() - lag {
            acc += seg[i] * seg[i + lag];
        }
        if acc > best.1 {
            best = (lag, acc);
        }
    }
    best.0
}

#[test]
fn midi_note_renders_at_pitch() {
    let smf = parse(&minimal_smf()).expect("parse");
    let events = to_timeline(&smf);
    // note lasts 960 ticks at default 120 BPM = 1.0 s
    assert!((events.last().unwrap().time - 1.0).abs() < 1e-9);

    let opts = RenderOptions::default();
    let audio = render(&events, &opts);
    let sr = opts.sample_rate;

    // steady-state window inside the note
    let seg = &audio[(0.4 * sr) as usize..(0.9 * sr) as usize];
    let rms = (seg.iter().map(|x| x * x).sum::<f32>() / seg.len() as f32).sqrt();
    assert!(rms > 0.01, "no oscillation, rms = {rms}");

    let lag = dominant_period(seg, 40, 400);
    let measured = sr / lag as f32;
    assert!((measured - 440.0).abs() < 8.0, "expected ~440 Hz, measured {measured} Hz");

    // after the note ends the sound must decay
    let tail = &audio[(2.0 * sr) as usize..];
    let tail_rms = (tail.iter().map(|x| x * x).sum::<f32>() / tail.len() as f32).sqrt();
    assert!(tail_rms < rms * 0.1, "note did not decay: tail rms {tail_rms} vs {rms}");
}
