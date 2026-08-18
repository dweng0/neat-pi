//! Command parsing — turn a line of text into a typed `Command`.
//!
//! This is the Rust idiom that replaces the C++ `if (c == 'S') ... else if ...`
//! chain from the old firmware. We describe the possible commands as an `enum`,
//! then `match` on them wherever we act. The compiler then forces us to handle
//! every variant, so we can't forget one.
//!
//! Serial protocol (same as the C++ firmware, 115200 baud, newline-terminated):
//!
//! ```text
//! F <0-255>   forward at PWM duty   e.g. "F 180"
//! R <0-255>   reverse at PWM duty   e.g. "R 200"
//! S           stop / coast
//! B           brake (short the motor)
//! Z           zero
//! <number>    bare number -> treated as forward at that duty
//! ```

/// A parsed, validated instruction for the motor.
///
/// Every variant carries a [`CommandPayload`] so the label + duty travel
/// together. `S`/`B`/`Z` have no meaningful duty, so they carry `0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Forward(CommandPayload),
    Reverse(CommandPayload),
    Stop(CommandPayload),
    Brake(CommandPayload),
    Zero(CommandPayload),
}

/// The label + duty that rides inside a [`Command`].
///
/// `label` is a `&'static str` (a borrow of text baked into the binary), not a
/// `String`: this crate is `no_std` on the chip, so there's no heap to allocate
/// a `String` on. A `&'static str` costs nothing and is `Copy`, which is why the
/// whole type can derive `Copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandPayload {
    pub label: &'static str,
    pub duty: u8,
}

impl CommandPayload {
    /// Build a payload. A `u8` is a single Copy byte, so we take it by value.
    pub fn new(label: &'static str, duty: u8) -> Self {
        Self { label, duty }
    }
}

/// Map an already-split direction token + optional duty into a typed [`Command`].
/// `F`/`R` require a duty (missing → [`ParseError::BadDuty`]); `S`/`B`/`Z` ignore it.
pub fn get_command_action(direction: &str, duty: Option<u8>) -> Result<Command, ParseError> {
    match direction {
        "F" => Ok(Command::Forward(CommandPayload::new("F", duty.ok_or(ParseError::BadDuty)?))),
        "R" => Ok(Command::Reverse(CommandPayload::new("R", duty.ok_or(ParseError::BadDuty)?))),
        "S" => Ok(Command::Stop(CommandPayload::new("S", 0))),
        "B" => Ok(Command::Brake(CommandPayload::new("B", 0))),
        "Z" => Ok(Command::Zero(CommandPayload::new("Z", 0))),
        _ => Err(ParseError::UnknownCommand),
    }
}

/// Render a `Command` back to its wire text (the inverse of `parse_command`).
///
/// Implementing `Display` means BOTH sides get encoding: the host tool can
/// `format!("{cmd}\n")`, and the firmware can `write!` it into a buffer.
/// NOTE: no trailing newline here — Display is the pure value ("F 180");
/// whoever sends it adds the framing '\n'.
impl core::fmt::Display for Command {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Command::*;
        match self {
            Forward(p) => write!(f, "F {}", p.duty),
            Reverse(p) => write!(f, "R {}", p.duty),
            Stop(_) => write!(f, "S"),
            Brake(_) => write!(f, "B"),
            Zero(_) => write!(f, "Z"),
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

/// Parse one line of wire text into a [`Command`].
///
/// The first character selects the command, case-insensitively. `F` and `R`
/// take a PWM duty in `0..=255`; a bare number is treated as [`Command::Forward`]
/// at that duty. Surrounding whitespace is ignored.
///
/// # Examples
///
/// ```
/// use pwm_serial_protocol::command::{parse_command, Command, CommandPayload};
///
/// assert_eq!(parse_command("F 180"), Ok(Command::Forward(CommandPayload::new("F", 180))));
/// assert_eq!(parse_command("200"),   Ok(Command::Forward(CommandPayload::new("F", 200))));
/// assert_eq!(parse_command("s"),     Ok(Command::Stop(CommandPayload::new("S", 0))));
/// ```
///
/// # Errors
///
/// - [`ParseError::Empty`] — the line was blank or only whitespace.
/// - [`ParseError::BadDuty`] — an `F`/`R` duty was missing or outside `0..=255`.
/// - [`ParseError::UnknownCommand`] — the first character matched nothing above.
pub fn parse_command(line: &str) -> Result<Command, ParseError> {
    // Grab the first char as an Option, bail if empty, uppercase for case-insensitivity.
    let first_character = line.trim().chars().next();
    let c = first_character
        .ok_or(ParseError::Empty)?
        .to_ascii_uppercase();
    match c {
        'S' => Ok(Command::Stop(CommandPayload::new("S", 0))),
        'B' => Ok(Command::Brake(CommandPayload::new("B", 0))),
        'F' => {
            let trimmed_line = line.trim()[1..].trim();
            let duty = parse_duty(trimmed_line)?;
            Ok(Command::Forward(CommandPayload::new("F", duty)))
        }
        'R' => {
            let trimmed_line = line.trim()[1..].trim();
            let duty = parse_duty(trimmed_line)?;
            Ok(Command::Reverse(CommandPayload::new("R", duty)))
        }
        '0'..='9' => {
            let duty = parse_duty(line.trim())?;
            Ok(Command::Forward(CommandPayload::new("F", duty)))
        }
        _ => Err(ParseError::UnknownCommand),
    }
}

fn parse_duty(duty_str: &str) -> Result<u8, ParseError> {
    duty_str.parse::<u8>().map_err(|_| ParseError::BadDuty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stop() {
        assert_eq!(parse_command("S"), Ok(Command::Stop(CommandPayload::new("S", 0))));
    }

    #[test]
    fn parses_forward_with_duty() {
        assert_eq!(
            parse_command("F 180"),
            Ok(Command::Forward(CommandPayload::new("F", 180)))
        );
    }

    #[test]
    fn parses_bare_number_as_forward() {
        assert_eq!(
            parse_command("200"),
            Ok(Command::Forward(CommandPayload::new("F", 200)))
        );
    }

    #[test]
    fn junk_is_unknown() {
        assert_eq!(parse_command("hello"), Err(ParseError::UnknownCommand));
    }
}
