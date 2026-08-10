//! Command parsing — turn a line of text into a typed `Command`.
//!
//! This is the Rust idiom that replaces the C++ `if (c == 'S') ... else if ...`
//! chain from the old firmware. We describe the possible commands as an `enum`,
//! then `match` on them wherever we act. The compiler then forces us to handle
//! every variant, so we can't forget one.
//!
//! Serial protocol (same as the C++ firmware, 115200 baud, newline-terminated):
//!   F <0-255>   forward at PWM duty   e.g. "F 180"
//!   R <0-255>   reverse at PWM duty   e.g. "R 200"
//!   S           stop / coast
//!   B           brake (short the motor)
//!   Z           zero
//!   <number>    bare number -> treated as forward at that duty

/// A parsed, validated instruction for the motor.
///
/// `duty` is the PWM level 0..=255, matching the C++ firmware's 8-bit duty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Forward(u8),
    Reverse(u8),
    Stop,
    Brake,
    Zero,
}

/// Render a `Command` back to its wire text (the inverse of `parse_command`).
///
/// Implementing `Display` means BOTH sides get encoding: the host tool can
/// `format!("{cmd}\n")`, and the firmware can `write!` it into a buffer.
/// NOTE: no trailing newline here — Display is the pure value ("F 180");
/// whoever sends it adds the framing '\n'.
impl core::fmt::Display for Command {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Pull the variants into scope so we can write `Forward` not
        // `Command::Forward` on every arm.
        use Command::*;
        match self {
            Forward(duty) => write!(f, "F {duty}"),
            Reverse(duty) => write!(f, "R {duty}"),
            Stop => write!(f, "S"),
            Brake => write!(f, "B"),
            Zero => write!(f, "Z"),
        }
    }
}

/// Why a line failed to parse. Returning a typed error (rather than silently
/// doing nothing) lets `main` print a helpful message back over serial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The line was empty / only whitespace.
    Empty,
    /// First token wasn't F, R, S, B, or a bare number.
    UnknownCommand,
    /// An F/R command was missing its number, or the number wasn't 0..=255.
    BadDuty,
}

/// Parse one trimmed line into a `Command`.
///
/// EXPECTED BEHAVIOUR (your TODO):
///   - Trim the line. If it's empty, return `Err(ParseError::Empty)`.
///   - Look at the first character, case-insensitively:
///       'S' -> Ok(Command::Stop)
///       'B' -> Ok(Command::Brake)
///       'F' -> parse the rest of the line as a u8 duty -> Ok(Command::Forward(duty))
///       'R' -> parse the rest of the line as a u8 duty -> Ok(Command::Reverse(duty))
///       a digit -> the whole line is a bare number -> Ok(Command::Forward(duty))
///       anything else -> Err(ParseError::UnknownCommand)
///   - If a duty is required but missing or not a valid 0..=255, return
///     `Err(ParseError::BadDuty)`.
///
/// HINTS:
///   - `line.trim()` gives you a `&str` with surrounding whitespace removed.
///   - `line.chars().next()` gets the first char; `c.to_ascii_uppercase()`
///     normalises case; `c.is_ascii_digit()` tests for a bare number.
///   - `"180".parse::<u8>()` returns `Result<u8, _>` — note `u8` parsing already
///     rejects anything above 255 for you, which is handy.
///   - For "F 180", you want the substring after the first char, trimmed, then
///     parsed. `line[1..].trim()` is one way.
///   - Map the parse `Result` into our `ParseError::BadDuty` with `.map_err(...)`
///     or an `if let` / `match`.
pub fn parse_command(line: &str) -> Result<Command, ParseError> {
    todo!("parse `line` into a Command per the rules above")
}
