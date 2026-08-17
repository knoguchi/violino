# violino

A physical model of bowed strings, built as a library: MIDI-shaped control
signals in, audio samples out. Headless by design — no UI, no plugin
framework required, dependency-free DSP core in Rust.

**Status: early prototype.** The waveguide string with friction-curve bowing
works (it locks into Helmholtz motion, plays legato phrases with vibrato and
dynamics); everything that makes a violin sound like a *good* violin is on
the roadmap below.

## Why

Expressive bowed-string synthesis is dominated by one excellent proprietary
product (Audio Modeling's SWAM) built on theory that has been public for
decades. Between textbook implementations (STK, Faust `physmodels`) and
research code there is a gap: no polished, open, *library-first* bowed-string
instrument that a program can play. violino aims at that gap.

The design bet: the instrument is a pure function. Player gestures (bow
velocity, bow pressure, bow position, fingering) go in as control signals;
the model does not decide how to play, only how to vibrate. The "player" —
the layer that turns MIDI notes and CC curves into plausible bowing gestures
— is a separate, first-class component.

## Layout

- `violino-core` — the instrument: waveguide string, MSW-style friction bow,
  modal body. Zero dependencies, no allocation on the audio path.
- `violino-py` — PyO3 bindings (build with `maturin develop`), for research
  iteration, calibration, and analysis-by-synthesis experiments.
- `violino-midi` — Standard MIDI File player: notes + CC streams to bowing
  gestures. CC defaults follow SWAM Violin's factory mapping (CC11
  expression, CC1 vibrato depth, CC19 vibrato rate, CC5 portamento, pitch
  bend), so the same MIDI file can drive both violino and SWAM for A/B
  comparison. Zero-dependency SMF parser included.

## Quick start

```bash
cargo test                      # Helmholtz-motion and MIDI end-to-end checks
cargo run --example phrase      # renders violino_phrase.wav (7 s legato demo)

cargo run -p violino-midi --example gen_demo   # writes demo.mid
cargo run -p violino-midi -- demo.mid demo.wav # render any SMF to WAV
```

## Roadmap

1. ~~**MIDI file renderer** (`violino-midi`)~~ — done (minimal): SMF
   format 0/1, tempo map, mono legato player, SWAM-compatible CC defaults.
   Remaining: bow position / bow noise CCs, key switches, MPE.
2. **Measured body**: replace the hand-tuned mode table with modes fitted to
   published violin bridge-admittance / impulse-response measurements. The
   largest realism win per unit of effort.
3. **Elasto-plastic friction** (after Willemsen 2021): better bow-change and
   attack transients than the memoryless friction curve.
4. **Calibration pipeline**: analysis-by-synthesis against real violin
   recordings — optimize control curves per recording, then model parameters
   across recordings. The optimizer plays the role of the skilled player.
   First piece exists: `tools/compare_real.py` renders the ground-truth
   notes of a Bach10 piece (real performance timing) and reports log-mel
   spectral distance and envelope correlation against the real violin stem.
   Baseline on piece 01 (no expression data, hand-tuned body):
   LSD 1.354, envelope correlation 0.300 — the numbers to beat.
5. Pizzicato (excitation swap), harmonics (touch-point reflection),
   torsional waves, sympathetic strings, other instruments of the family
   (viola, cello, bass are parameter sets, not new code).

## Theory and credits

This is an independent implementation from the published literature — a
sibling of STK and Faust's `physmodels`, not a fork. Constants and loop
topology were initially referenced against STK's `Bowed` by Perry Cook and
Gary Scavone.

- M. E. McIntyre, R. T. Schumacher, J. Woodhouse,
  "On the oscillations of musical instruments",
  *J. Acoust. Soc. Am.* 74(5), 1983.
- J. O. Smith III, "Physical modeling using digital waveguides",
  *Computer Music Journal* 16(4), 1992.
- S. Serafin, "The sound of friction: real-time models, playability and
  musical applications", PhD thesis, Stanford University, 2004.
- M. Demoucron, "On the control of virtual violins", PhD thesis,
  UPMC / KTH, 2008.
- J. Woodhouse, "The acoustics of the violin: a review",
  *Reports on Progress in Physics* 77, 2014.
- S. Willemsen, "The emulated ensemble: real-time simulation of musical
  instruments using finite-difference time-domain methods", PhD thesis,
  Aalborg University, 2021.

## License

MIT OR Apache-2.0, at your option.
