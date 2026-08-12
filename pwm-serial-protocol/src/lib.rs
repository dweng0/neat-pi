// The pure, hardware-free half of the firmware: turning bytes into lines
// (serial) and lines into typed commands (command). No esp-hal here, so this
// crate also builds for the host — which is what lets us unit-test it.
//
// `no_std` normally (it runs on the chip), but under `cargo test` we allow std
// so the test harness can run on your Mac.
#![cfg_attr(not(test), no_std)]

pub mod command;
pub mod serial;
