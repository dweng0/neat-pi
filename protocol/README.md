# pwm-serial-protocol

A tiny `no_std` Rust crate that parses and encodes the serial protocol used to
drive a PWM motor controller over a UART link. It turns a stream of incoming
bytes into whole lines, and whole lines into typed, validated `Command`s — and
back again.

It's the pure, hardware-free half of an ESP32 motor-control firmware (no
`esp-hal`, no I/O), which is exactly what lets it build for the host and be
unit-tested on your laptop.

## Why

Embedded firmware that reads commands off a serial port usually grows an
ad-hoc `if (c == 'S') ... else if (c == 'F') ...` chain buried inside the main
loop. That's hard to test (you need the chip), easy to get subtly wrong (drop a
byte, bleed one line into the next), and impossible to reuse on the host side of
the link.

This crate pulls that logic out into its own `no_std` library so that:

- **It's testable off-target.** No `esp-hal`, no hardware dependencies. Under
  `cargo test` it builds against `std` and the whole parser runs on your Mac.
- **The compiler enforces completeness.** Commands are an `enum`; you `match`
  on them, so you can't forget to handle a variant.
- **Both ends share one definition.** `Command` implements `Display`, so the
  host tool encodes with `format!("{cmd}\n")` and the firmware parses the same
  wire text — one source of truth for the protocol.
- **Bad input is typed, not silent.** A malformed line returns a `ParseError`
  you can report back over serial instead of quietly doing nothing.

## The protocol

115200 baud, newline-terminated ASCII lines. `duty` is an 8-bit PWM level
(`0..=255`).

```text
F <0-255>   forward at PWM duty    e.g. "F 180"
R <0-255>   reverse at PWM duty    e.g. "R 200"
S           stop / coast
B           brake (short the motor)
Z           zero
<number>    bare number -> forward at that duty   e.g. "200"
```

The first character selects the command (case-insensitive); surrounding
whitespace is ignored.

## How to

Add it to your firmware (or host tool):

```toml
[dependencies]
pwm-serial-protocol = "1.0"
```

### Reading lines off a UART

`LineReader` accumulates bytes into a fixed inline buffer (`heapless::String`,
no heap) and hands you a `&str` the moment a newline lands:

```rust
use pwm_serial_protocol::serial::LineReader;
use pwm_serial_protocol::command::parse_command;

let mut reader = LineReader::new();

// In your UART interrupt / poll loop, feed one received byte at a time:
for &byte in b"F 180\n" {
    if let Some(line) = reader.feed(byte) {
        match parse_command(line) {
            Ok(cmd)  => { /* drive the motor from `cmd` */ }
            Err(err) => { /* report `err` back over serial */ }
        }
    }
}
```

`feed` returns `None` for every byte until a `\n` completes a line. `\r` is
ignored (terminals send `\r\n`). Lines longer than `MAX_LINE` (64) bytes are
truncated rather than growing.

### Parsing a line into a command

```rust
use pwm_serial_protocol::command::{parse_command, Command};

assert_eq!(parse_command("F 180"), Ok(Command::Forward(180)));
assert_eq!(parse_command("200"),   Ok(Command::Forward(200)));
assert_eq!(parse_command("s"),     Ok(Command::Stop));
```

### Encoding a command back to the wire

`Command` implements `Display` (no trailing newline — you add the `\n` framing):

```rust
use pwm_serial_protocol::command::Command;

let cmd = Command::Forward(180);
assert_eq!(cmd.to_string(), "F 180");
// send it: format!("{cmd}\n")
```

## Testing

```sh
cargo test
```

The library is `#![no_std]` on-target, but the test harness builds with `std`
so the parser and line reader run natively.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
