//! Merge SMF tracks and resolve the tempo map into absolute seconds.

use crate::smf::{EventKind, Smf};

#[derive(Debug, Clone, Copy)]
pub struct TimedEvent {
    /// Absolute time in seconds from the start of the file.
    pub time: f64,
    pub kind: EventKind,
}

/// Flatten all tracks into one event list with absolute times in seconds,
/// applying tempo changes (default 120 BPM until the first tempo event).
pub fn to_timeline(smf: &Smf) -> Vec<TimedEvent> {
    // (absolute tick, track index, event index, kind) — indexes keep the
    // sort stable so simultaneous events preserve file order.
    let mut merged: Vec<(u64, usize, usize, EventKind)> = Vec::new();
    for (ti, track) in smf.tracks.iter().enumerate() {
        let mut tick: u64 = 0;
        for (ei, ev) in track.iter().enumerate() {
            tick += ev.delta as u64;
            merged.push((tick, ti, ei, ev.kind));
        }
    }
    merged.sort_by_key(|&(tick, ti, ei, _)| (tick, ti, ei));

    let mut out = Vec::with_capacity(merged.len());
    let mut us_per_quarter: f64 = 500_000.0;
    let mut prev_tick: u64 = 0;
    let mut time: f64 = 0.0;
    let tpq = smf.ticks_per_quarter as f64;
    for (tick, _, _, kind) in merged {
        time += (tick - prev_tick) as f64 * us_per_quarter / tpq / 1e6;
        prev_tick = tick;
        if let EventKind::Tempo { microseconds_per_quarter } = kind {
            us_per_quarter = microseconds_per_quarter as f64;
        }
        out.push(TimedEvent { time, kind });
    }
    out
}
