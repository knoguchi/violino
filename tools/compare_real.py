#!/usr/bin/env python3
"""Compare a violino rendering against a real recording (Bach10 layout).

Pipeline:
  1. read Bach10 ground-truth notes (onset_ms, offset_ms, midi_pitch, source),
     which are aligned to the real performance,
  2. write a MIDI file for one instrument from those notes,
  3. render it with the violino-midi CLI,
  4. compare against the real instrument stem: log-mel spectrograms,
     log-spectral distance, and envelope correlation, saved as a PNG.

The rendered part has no expression data (the ground truth is notes only),
so this measures the combination "violino + naive player" against
"real violin + real player". Closing that gap with optimized gesture
curves is the analysis-by-synthesis roadmap item.

Usage:
  compare_real.py <piece_dir> [--instrument 1] [--out-prefix out/name]

Dependencies: numpy, matplotlib, mido.
"""

import argparse
import pathlib
import subprocess
import sys
import wave

import matplotlib
import mido
import numpy as np

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402


def read_notes(txt_path, instrument):
    notes = []
    for line in pathlib.Path(txt_path).read_text().splitlines():
        parts = line.split()
        if len(parts) != 4:
            continue
        onset_ms, offset_ms, pitch, source = (int(float(x)) for x in parts)
        if source == instrument:
            notes.append((onset_ms / 1000.0, offset_ms / 1000.0, pitch))
    notes.sort()
    return notes


def notes_to_midi(notes, path):
    mid = mido.MidiFile(ticks_per_beat=480)
    track = mido.MidiTrack()
    mid.tracks.append(track)
    tempo = 500000  # 120 BPM: 1 tick = 1/960 s
    track.append(mido.MetaMessage("set_tempo", tempo=tempo, time=0))
    events = []
    for onset, offset, pitch in notes:
        events.append((onset, 1, pitch))
        events.append((offset, 0, pitch))
    # sort by time; note-offs first on ties so legato overlaps stay sane
    events.sort(key=lambda e: (e[0], e[1]))
    prev_tick = 0
    for time_s, kind, pitch in events:
        tick = round(mido.second2tick(time_s, mid.ticks_per_beat, tempo))
        msg = "note_on" if kind else "note_off"
        track.append(mido.Message(msg, note=pitch, velocity=64, time=tick - prev_tick))
        prev_tick = tick
    mid.save(path)


def load_wav(path):
    with wave.open(str(path)) as w:
        assert w.getsampwidth() == 2, "expected 16-bit PCM"
        sr = w.getframerate()
        raw = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16)
        if w.getnchannels() > 1:
            raw = raw.reshape(-1, w.getnchannels()).mean(axis=1)
        return raw.astype(np.float32) / 32768.0, sr


def log_mel(sig, sr, n_fft=2048, hop=512, n_mels=80, fmax=8000.0):
    frames = 1 + (len(sig) - n_fft) // hop
    window = np.hanning(n_fft)
    spec = np.empty((n_fft // 2 + 1, frames))
    for i in range(frames):
        seg = sig[i * hop : i * hop + n_fft] * window
        spec[:, i] = np.abs(np.fft.rfft(seg))
    freqs = np.fft.rfftfreq(n_fft, 1.0 / sr)

    def mel(f):
        return 2595.0 * np.log10(1.0 + f / 700.0)

    mel_points = np.linspace(mel(0.0), mel(fmax), n_mels + 2)
    hz_points = 700.0 * (10.0 ** (mel_points / 2595.0) - 1.0)
    fb = np.zeros((n_mels, len(freqs)))
    for m in range(n_mels):
        lo, mid, hi = hz_points[m : m + 3]
        fb[m] = np.clip(
            np.minimum((freqs - lo) / (mid - lo + 1e-9), (hi - freqs) / (hi - mid + 1e-9)),
            0.0,
            None,
        )
    return np.log10(fb @ spec + 1e-6)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("piece_dir")
    ap.add_argument("--instrument", type=int, default=1, help="Bach10 source index, 1 = violin")
    ap.add_argument("--out-prefix", default=None)
    args = ap.parse_args()

    piece = pathlib.Path(args.piece_dir)
    name = piece.name
    stem_suffix = {1: "violin", 2: "clarinet", 3: "saxphone", 4: "bassoon"}[args.instrument]
    real_wav = piece / f"{name}-{stem_suffix}.wav"
    prefix = pathlib.Path(args.out_prefix or f"tmp/{name}-{stem_suffix}")
    prefix.parent.mkdir(parents=True, exist_ok=True)

    notes = read_notes(piece / f"{name}.txt", args.instrument)
    print(f"{len(notes)} ground-truth notes for instrument {args.instrument} ({stem_suffix})")
    midi_path = f"{prefix}.mid"
    synth_path = f"{prefix}.violino.wav"
    notes_to_midi(notes, midi_path)

    repo = pathlib.Path(__file__).resolve().parent.parent
    subprocess.run(
        ["cargo", "run", "--release", "-q", "-p", "violino-midi", "--", midi_path, synth_path],
        cwd=repo,
        check=True,
    )

    real, sr_r = load_wav(real_wav)
    synth, sr_s = load_wav(synth_path)
    assert sr_r == sr_s, f"sample rate mismatch: {sr_r} vs {sr_s}"
    n = min(len(real), len(synth))
    real, synth = real[:n], synth[:n]

    mel_r = log_mel(real, sr_r)
    mel_s = log_mel(synth, sr_r)

    # normalize overall level before distancing: loudness is trivially fixable
    mel_r -= mel_r.mean()
    mel_s -= mel_s.mean()
    lsd = float(np.mean(np.abs(mel_r - mel_s)))
    env_r = mel_r.mean(axis=0)
    env_s = mel_s.mean(axis=0)
    env_corr = float(np.corrcoef(env_r, env_s)[0, 1])
    print(f"log-mel spectral distance: {lsd:.3f} (lower is better)")
    print(f"envelope correlation:      {env_corr:.3f} (higher is better)")

    extent = [0, n / sr_r, 0, 8000]
    fig, axes = plt.subplots(3, 1, figsize=(12, 10), sharex=True)
    for ax, data, title in (
        (axes[0], mel_r, f"real {stem_suffix} ({name})"),
        (axes[1], mel_s, "violino render (ground-truth notes, no expression)"),
        (axes[2], np.abs(mel_r - mel_s), f"abs difference — LSD {lsd:.3f}, env corr {env_corr:.3f}"),
    ):
        im = ax.imshow(data, aspect="auto", origin="lower", extent=extent, cmap="magma")
        ax.set_title(title)
        ax.set_ylabel("Hz (mel-spaced)")
        fig.colorbar(im, ax=ax, pad=0.01)
    axes[2].set_xlabel("s")
    fig.tight_layout()
    png = f"{prefix}.compare.png"
    fig.savefig(png, dpi=110)
    print(f"wrote {midi_path}, {synth_path}, {png}")


if __name__ == "__main__":
    main()
