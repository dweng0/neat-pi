#![no_std]

// Library crate for the Neato ESP32 firmware.
//
// The binary (src/bin/main.rs) is a *separate* crate that depends on this one,
// so anything it needs must be `pub` here.
//
// The pure "abstract the bytes away" logic (serial line reader + command
// parser) now lives in the sibling `protocol` crate, so it can be unit-tested
// on the host. Import it directly from there, e.g.:
//
//     use protocol::command::{Command, parse_command};
//     use protocol::serial::LineReader;
//
// This crate keeps only the hardware-bound part:
//   - motor: turn a Command into DRV8871 H-bridge / PWM output

pub mod motor;
