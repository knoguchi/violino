//! Generate `demo.mid`: a legato phrase with expression (CC11) curves,
//! vibrato (CC1) on the held final note, and overlapping note-ons for
//! legato transitions. The file is plain SMF format 0 and also plays on
//! SWAM Violin with factory mapping — that is the point.

use std::io::Write;

fn vlq(mut value: u32) -> Vec<u8> {
    let mut stack = vec![(value & 0x7F) as u8];
    value >>= 7;
    while value > 0 {
        stack.push(0x80 | (value & 0x7F) as u8);
        value >>= 7;
    }
    stack.reverse();
    stack
}

struct Track {
    bytes: Vec<u8>,
}

impl Track {
    fn event(&mut self, delta: u32, msg: &[u8]) {
        self.bytes.extend(vlq(delta));
        self.bytes.extend_from_slice(msg);
    }

    fn cc(&mut self, delta: u32, controller: u8, value: u8) {
        self.event(delta, &[0xB0, controller, value]);
    }

    /// Emit a linear CC ramp over `ticks`, one event per step.
    fn cc_ramp(&mut self, controller: u8, from: u8, to: u8, ticks: u32, steps: u32) {
        for i in 1..=steps {
            let v = from as f32 + (to as f32 - from as f32) * i as f32 / steps as f32;
            self.cc(ticks / steps, controller, v as u8);
        }
    }
}

fn main() {
    const TPQ: u32 = 480; // ticks per quarter
    const OVERLAP: u32 = 20; // legato: next note starts before previous ends

    let mut t = Track { bytes: Vec::new() };
    // 100 BPM
    t.event(0, &[0xFF, 0x51, 0x03, 0x09, 0x27, 0xC0]);
    t.cc(0, 11, 70); // start mezzo-piano
    t.cc(0, 1, 0); // no vibrato yet

    // A4 - B4 - C#5 - D5, one quarter each, legato, with a small
    // swell (CC11) inside every note.
    let notes = [69u8, 71, 73, 74];
    for (i, &key) in notes.iter().enumerate() {
        t.event(0, &[0x90, key, 64]);
        if i > 0 {
            t.event(OVERLAP, &[0x80, notes[i - 1], 0]);
            t.cc_ramp(11, 78, 95, TPQ - OVERLAP, 8);
        } else {
            t.cc_ramp(11, 70, 95, TPQ, 8);
        }
    }

    // E5 held for 3 quarters: vibrato fades in (CC1), expression swells
    // then decays for the release.
    t.event(0, &[0x90, 76, 72]);
    t.event(OVERLAP, &[0x80, 74, 0]);
    t.cc_ramp(1, 0, 85, TPQ, 12);
    t.cc_ramp(11, 95, 112, TPQ, 8);
    t.cc_ramp(11, 112, 30, TPQ, 12);
    t.event(0, &[0x80, 76, 0]);
    t.event(0, &[0xFF, 0x2F, 0x00]); // end of track

    let mut file = Vec::new();
    file.extend_from_slice(b"MThd");
    file.extend_from_slice(&6u32.to_be_bytes());
    file.extend_from_slice(&0u16.to_be_bytes()); // format 0
    file.extend_from_slice(&1u16.to_be_bytes());
    file.extend_from_slice(&(TPQ as u16).to_be_bytes());
    file.extend_from_slice(b"MTrk");
    file.extend_from_slice(&(t.bytes.len() as u32).to_be_bytes());
    file.extend_from_slice(&t.bytes);

    std::fs::File::create("demo.mid").and_then(|mut f| f.write_all(&file)).expect("write demo.mid");
    println!("wrote demo.mid ({} bytes)", file.len());
}
