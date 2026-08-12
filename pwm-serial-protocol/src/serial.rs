//! Serial line reader — hides the byte-at-a-time UART reading behind a
//! "feed me bytes, I'll hand you back whole lines" interface.
//!
//! In milestone 1 `main` echoed raw bytes. Here we accumulate incoming bytes
//! into a small fixed buffer until we see a newline, then surface the completed
//! line as a `&str` so the command parser can work with text instead of bytes.
//!
//! Because we're `no_std` there's no heap and no `std::String`, so we use
//! `heapless::String`, which stores its characters inline with a fixed maximum
//! capacity (`MAX_LINE`). Pushing past capacity fails instead of growing.

use heapless::String;

/// Longest command line we'll accept. Commands like "F 180" are tiny, so 64 is
/// generous. Bytes arriving after the buffer is full (before a newline) get
/// dropped — see `feed`.
pub const MAX_LINE: usize = 64;

/// Accumulates bytes into a line buffer and yields a line when a newline lands.
pub struct LineReader {
    buf: String<MAX_LINE>,
    complete: bool,
}

impl Default for LineReader {
    fn default() -> Self {
        Self::new()
    }
}

impl LineReader {
    /// Start with an empty buffer.
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            complete: false,
        }
    }

    /// Feed one received byte in.
    pub fn feed(&mut self, byte: u8) -> Option<&str> {
        if self.complete {
            self.buf.clear();
            self.complete = false;
        }
        match byte {
            b'\n' => {
                self.complete = true;
                Some(self.buf.as_str())
            }
            b'\r' => None,
            _ => {
                self.buf.push(byte as char).ok();
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_until_newline() {
        let mut r = LineReader::new();
        // Every byte before the '\n' is "not a line yet".
        assert_eq!(r.feed(b'F'), None);
        assert_eq!(r.feed(b' '), None);
        assert_eq!(r.feed(b'1'), None);
    }

    #[test]
    fn yields_whole_line_on_newline() {
        let mut r = LineReader::new();
        for &b in b"F 180" {
            assert_eq!(r.feed(b), None);
        }
        // The newline is what completes the line.
        assert_eq!(r.feed(b'\n'), Some("F 180"));
    }

    #[test]
    fn carriage_return_is_ignored() {
        // Terminals send "\r\n"; we only care about the '\n'.
        let mut r = LineReader::new();
        assert_eq!(r.feed(b'S'), None);
        assert_eq!(r.feed(b'\r'), None);
        assert_eq!(r.feed(b'\n'), Some("S"));
    }

    #[test]
    fn buffer_resets_between_lines() {
        // The bug we fixed: the previous line must NOT bleed into the next one.
        let mut r = LineReader::new();
        for &b in b"F 1" {
            r.feed(b);
        }
        assert_eq!(r.feed(b'\n'), Some("F 1"));

        // Second line should come back clean, not "F 1S".
        assert_eq!(r.feed(b'S'), None);
        assert_eq!(r.feed(b'\n'), Some("S"));
    }
}
