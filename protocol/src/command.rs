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

/// Parse one line of wire text into a [`Command`].
///
/// The first character selects the command, case-insensitively. `F` and `R`
/// take a PWM duty in `0..=255`; a bare number is treated as [`Command::Forward`]
/// at that duty. Surrounding whitespace is ignored.
///
/// # Examples
///
/// ```
/// use pwm_serial_protocol::command::{parse_command, Command};
///
/// assert_eq!(parse_command("F 180"), Ok(Command::Forward(180)));
/// assert_eq!(parse_command("200"),   Ok(Command::Forward(200)));
/// assert_eq!(parse_command("s"),     Ok(Command::Stop));
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
        'S' => Ok(Command::Stop),
        'B' => Ok(Command::Brake),
        'F' => {
            let trimmed_line = line.trim()[1..].trim();
            let duty = parse_duty(trimmed_line)?;
            Ok(Command::Forward(duty))
        }
        'R' => {
            let trimmed_line = line.trim()[1..].trim();
            let duty = parse_duty(trimmed_line)?;
            Ok(Command::Reverse(duty))
        }
        '0'..='9' => {
            let duty = parse_duty(line.trim())?;
            Ok(Command::Forward(duty))
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
        assert_eq!(parse_command("S"), Ok(Command::Stop));
    }

    #[test]
    fn parses_forward_with_duty() {
        assert_eq!(parse_command("F 180"), Ok(Command::Forward(180)));
    }

    #[test]
    fn parses_bare_number_as_forward() {
        assert_eq!(parse_command("200"), Ok(Command::Forward(200)));
    }

    #[test]
    fn junk_is_unknown() {
        assert_eq!(parse_command("hello"), Err(ParseError::UnknownCommand));
    }
}
