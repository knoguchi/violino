# violino — project context

Physical model of bowed strings as a Rust library. Read README.md first; it
is the source of truth for roadmap and citations. This file holds context
that is not derivable from the code.

## Decisions already made (do not relitigate)

- **Independent implementation from papers, not a fork.** Code is written
  from the published equations (MSW 1983, Smith 1992, Serafin 2004,
  Willemsen 2021). Do not copy code from STK/Faust; referencing their
  constants is fine and must be credited in comments
  (e.g. `// after STK Bowed`).
- **Instrument vs player separation.** `violino-core` is the instrument: a
  pure function from per-sample gesture signals (f0, bow velocity, bow
  pressure, bow position) to audio. Vibrato, portamento, note transitions,
  and CC-to-gesture mapping belong to the player layer (`violino-midi`,
  planned), never inside the core.
- **Zero dependencies in `violino-core`**, no allocation on the audio path.
  Mirrors the structure of the author's dx7 project
  (github.com/knoguchi/dx7: dx7-core / dx7-app / dx7-midi, embedded targets
  excluded from the workspace); keep that door open here.
- **Offline rendering first**; real-time and plugin formats are explicitly
  deferred.
- **Name**: "violino" (checked available on crates.io and PyPI, 2026-08-16).
  Publish only when there is real content; no name-squatting.

## Goal that shapes the roadmap

The user wants to drive SWAM Violin and violino with the **same MIDI file**
and compare outputs — player held constant, engines differing. Hence the
`violino-midi` SMF renderer with a SWAM-compatible CC mapping is the top
roadmap item, and hand-written curves in `examples/phrase.rs` are a stopgap.

## Calibration strategy (decided 2026-08-16)

The chosen path is: keep the physical engine, learn the **player**.

- **Phase 1 (next up)**: gradient-free optimization (CMA-ES or
  Nelder-Mead) of per-note control parameters (expression, pressure
  bonus, vibrato depth/rate, timing offsets) against a real recording,
  using the Rust engine as a black box and the log-mel spectral distance
  from `tools/compare_real.py` as the cost. Purpose: measure how far the
  current engine can go with a good player — the engine's ceiling.
- **Phase 2**: port the tick loop to PyTorch/JAX (~50 lines, everything
  is differentiable: linear-interp delays, smooth friction, biquads).
  Then (a) fit control curves by gradient, (b) train a small
  MIDI-to-gesture "player network" end-to-end on Bach10/URMP pairs, and
  (c) calibrate physical constants (friction shape, body modes) by
  gradient. Train on short segments (per-note), not whole pieces (BPTT
  over 1M steps is impractical).
- Explicitly rejected: replacing the instrument with a neural net
  (DDSP/MIDI-DDSP) — keep those as benchmarks only; NN residual
  correction on the output — only to be considered after Phase 2.

## Quality bar and evaluation

Do not evaluate sound quality by ear alone. The calibration pipeline
(analysis-by-synthesis against real recordings, optimizer as the "player")
is the intended quality metric. Known current weaknesses, in order of
impact: placeholder body modes (hand-tuned table in `body.rs`), memoryless
friction curve, no torsional waves.

## Workflow notes

- The user communicates in Japanese; code, comments, and docs are English.
- `cargo test` runs a Helmholtz-motion check (`tests/pitch.rs`): steady bow
  must lock to the target f0. Keep this test passing; extend it when adding
  physics.
- Python bindings build: `cd violino-py && maturin develop` (needs maturin;
  pyo3 0.23, abi3-py39). `violino-py` is excluded from workspace
  default-members so plain `cargo test` doesn't need pyo3.
