//! Minimal Standard MIDI File (SMF format 0/1) parser.
//!
//! Only what a player needs: notes, controllers, pitch bend, tempo.
//! Everything else is parsed past and discarded. No dependencies.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventKind {
    NoteOn { channel: u8, key: u8, velocity: u8 },
    NoteOff { channel: u8, key: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    /// -8192..=8191, center 0.
    PitchBend { channel: u8, value: i16 },
    /// Microseconds per quarter note.
    Tempo { microseconds_per_quarter: u32 },
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct TrackEvent {
    pub delta: u32,
    pub kind: EventKind,
}

pub struct Smf {
    pub ticks_per_quarter: u16,
    pub tracks: Vec<Vec<TrackEvent>>,
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn u8(&mut self) -> Result<u8, String> {
        let b = *self.data.get(self.pos).ok_or("unexpected end of file")?;
        self.pos += 1;
        Ok(b)
    }

    fn peek(&self) -> Result<u8, String> {
        self.data.get(self.pos).copied().ok_or_else(|| "unexpected end of file".into())
    }

    fn bytes(&mut self, n: usize) -> Result<&'a [u8], String> {
        let s = self.data.get(self.pos..self.pos + n).ok_or("unexpected end of file")?;
        self.pos += n;
        Ok(s)
    }

    fn u16(&mut self) -> Result<u16, String> {
        let b = self.bytes(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, String> {
        let b = self.bytes(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// MIDI variable-length quantity.
    fn vlq(&mut self) -> Result<u32, String> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            let b = self.u8()?;
            value = (value << 7) | (b & 0x7F) as u32;
            if b & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err("variable-length quantity too long".into())
    }
}

pub fn parse(data: &[u8]) -> Result<Smf, String> {
    let mut c = Cursor { data, pos: 0 };
    if c.bytes(4)? != b"MThd" {
        return Err("not a MIDI file (missing MThd)".into());
    }
    if c.u32()? != 6 {
        return Err("unexpected MThd length".into());
    }
    let format = c.u16()?;
    if format > 1 {
        return Err(format!("SMF format {format} not supported (only 0 and 1)"));
    }
    let ntracks = c.u16()?;
    let division = c.u16()?;
    if division & 0x8000 != 0 {
        return Err("SMPTE time division not supported".into());
    }

    let mut tracks = Vec::with_capacity(ntracks as usize);
    for _ in 0..ntracks {
        if c.bytes(4)? != b"MTrk" {
            return Err("missing MTrk chunk".into());
        }
        let len = c.u32()? as usize;
        let end = c.pos + len;
        let mut events = Vec::new();
        let mut running_status: Option<u8> = None;
        while c.pos < end {
            let delta = c.vlq()?;
            let status = if c.peek()? & 0x80 != 0 {
                let s = c.u8()?;
                if s < 0xF0 {
                    running_status = Some(s);
                }
                s
            } else {
                running_status.ok_or("data byte without running status")?
            };
            let kind = match status & 0xF0 {
                0x80 => {
                    let key = c.u8()?;
                    c.u8()?; // release velocity
                    EventKind::NoteOff { channel: status & 0x0F, key }
                }
                0x90 => {
                    let key = c.u8()?;
                    let velocity = c.u8()?;
                    if velocity == 0 {
                        EventKind::NoteOff { channel: status & 0x0F, key }
                    } else {
                        EventKind::NoteOn { channel: status & 0x0F, key, velocity }
                    }
                }
                0xA0 => {
                    c.bytes(2)?; // polyphonic aftertouch
                    EventKind::Other
                }
                0xB0 => {
                    let controller = c.u8()?;
                    let value = c.u8()?;
                    EventKind::ControlChange { channel: status & 0x0F, controller, value }
                }
                0xC0 => {
                    c.u8()?; // program change
                    EventKind::Other
                }
                0xD0 => {
                    c.u8()?; // channel aftertouch
                    EventKind::Other
                }
                0xE0 => {
                    let lsb = c.u8()? as i16;
                    let msb = c.u8()? as i16;
                    EventKind::PitchBend { channel: status & 0x0F, value: ((msb << 7) | lsb) - 8192 }
                }
                _ => match status {
                    0xFF => {
                        let meta_type = c.u8()?;
                        let len = c.vlq()? as usize;
                        let body = c.bytes(len)?;
                        if meta_type == 0x51 && len == 3 {
                            EventKind::Tempo {
                                microseconds_per_quarter: u32::from_be_bytes([
                                    0, body[0], body[1], body[2],
                                ]),
                            }
                        } else {
                            EventKind::Other
                        }
                    }
                    0xF0 | 0xF7 => {
                        let len = c.vlq()? as usize;
                        c.bytes(len)?;
                        EventKind::Other
                    }
                    _ => return Err(format!("unsupported status byte {status:#04x}")),
                },
            };
            events.push(TrackEvent { delta, kind });
        }
        c.pos = end;
        tracks.push(events);
    }
    Ok(Smf { ticks_per_quarter: division, tracks })
}
