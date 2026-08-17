//! CLI: render a Standard MIDI File to WAV.
//!
//! Usage: violino-midi <input.mid> [output.wav]

use violino_midi::{parse, render, to_timeline, wav, RenderOptions};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = args.get(1).unwrap_or_else(|| {
        eprintln!("usage: violino-midi <input.mid> [output.wav]");
        std::process::exit(2);
    });
    let default_out = input.trim_end_matches(".mid").trim_end_matches(".midi").to_string() + ".wav";
    let output = args.get(2).unwrap_or(&default_out);

    let data = std::fs::read(input).unwrap_or_else(|e| {
        eprintln!("cannot read {input}: {e}");
        std::process::exit(1);
    });
    let smf = parse(&data).unwrap_or_else(|e| {
        eprintln!("cannot parse {input}: {e}");
        std::process::exit(1);
    });
    let events = to_timeline(&smf);
    let opts = RenderOptions::default();
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
