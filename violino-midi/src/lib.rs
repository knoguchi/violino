//! violino-midi: the player layer.
//!
//! Parses Standard MIDI Files and turns note + CC streams into the
//! per-sample bowing gestures that `violino-core` consumes. The CC mapping
//! defaults follow SWAM Violin's published MIDI controls so that one MIDI
//! performance can drive both engines for A/B comparison.

pub mod player;
pub mod smf;
pub mod timeline;
pub mod wav;

pub use player::{render, Mapping, RenderOptions};
pub use smf::{parse, EventKind, Smf};
pub use timeline::{to_timeline, TimedEvent};
