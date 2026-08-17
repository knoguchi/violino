//! CLI: render a Standard MIDI File to WAV.
//!
//! Usage: violino-midi <input.mid> [output.wav] [--channel N]

use violino_midi::{parse, render, to_timeline, wav, RenderOptions};

fn main() {
    let mut channel: Option<u8> = None;
    let mut positional: Vec<String> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--channel" {
            let v = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                eprintln!("--channel needs a number 0-15");
                std::process::exit(2);
            });
            channel = Some(v);
        } else {
            positional.push(a);
        }
    }
    let input = positional.first().unwrap_or_else(|| {
        eprintln!("usage: violino-midi <input.mid> [output.wav] [--channel N]");
        std::process::exit(2);
    });
    let default_out = input.trim_end_matches(".mid").trim_end_matches(".midi").to_string() + ".wav";
    let output = positional.get(1).unwrap_or(&default_out);

    let data = std::fs::read(input).unwrap_or_else(|e| {
        eprintln!("cannot read {input}: {e}");
        std::process::exit(1);
    });
    let smf = parse(&data).unwrap_or_else(|e| {
        eprintln!("cannot parse {input}: {e}");
        std::process::exit(1);
    });
    let events = to_timeline(&smf);
    let opts = RenderOptions { channel, ..RenderOptions::default() };
    let mut audio = render(&events, &opts);

    let peak = audio.iter().fold(0.0f32, |m, &x| m.max(x.abs()));
    if peak > 0.0 {
        for s in &mut audio {
            *s *= 0.7 / peak;
        }
    }
    wav::write_wav(output, opts.sample_rate as u32, &audio).unwrap_or_else(|e| {
        eprintln!("cannot write {output}: {e}");
        std::process::exit(1);
    });
    println!(
        "rendered {} events, {:.1}s -> {output}",
        events.len(),
        audio.len() as f32 / opts.sample_rate
    );
}
