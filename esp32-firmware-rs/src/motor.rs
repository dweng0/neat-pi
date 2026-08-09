//! Motor driver — turn a `Command` into DRV8871 H-bridge output.
//!
//! This is the milestone-2 port of the C++ `motorDrive` / `motorStop` /
//! `motorBrake` functions. It's the hardest module because it owns real
//! hardware (the LEDC PWM peripheral), so this is where you'll spend time in
//! the esp-hal docs. The command/serial modules above are pure logic and easy
//! to unit-test in your head; this one talks to pins.
//!
//! Wiring (unchanged from the C++ firmware):
//!   GPIO26 -> DRV8871 IN1
//!   GPIO27 -> DRV8871 IN2
//!   common GND with the motor PSU (REQUIRED)
//!
//! DRV8871 truth table (PWM rides directly on one input, there's no enable pin):
//!   IN1     IN2     result
//!   PWM     LOW     forward at duty
//!   LOW     PWM     reverse at duty
//!   LOW     LOW     coast (stop)
//!   HIGH    HIGH    brake (motor shorted)
//!
//! So each of IN1/IN2 needs to be an independent PWM output. In esp-hal that
//! means the LEDC peripheral with TWO channels — one bound to GPIO26, one to
//! GPIO27 — sharing a ~20 kHz timer (above audible). Duty is expressed as a
//! percentage 0..=100 in esp-hal's LEDC API, so you'll scale the 0..=255 duty
//! from the command into a percent.

use protocol::command::Command;

/// Owns the two PWM channels driving the DRV8871 inputs.
///
/// TODO: give this struct fields to hold the two configured LEDC channels
/// (one for IN1 on GPIO26, one for IN2 on GPIO27). You'll discover the exact
/// esp-hal types as you build `new` — they're in `esp_hal::ledc`. Storing them
/// keeps the peripheral alive for the whole program (if the channel is dropped,
/// the pin stops driving).
pub struct Motor {
    // in1: ledc::channel::Channel<'static, ...>,
    // in2: ledc::channel::Channel<'static, ...>,
}

impl Motor {
    /// Set up the LEDC peripheral and both channels, then return a ready Motor.
    ///
    /// TODO (the meaty one). Rough recipe — look these up in the esp-hal 1.1
    /// docs / examples:
    ///   1. `let mut ledc = Ledc::new(peripherals.LEDC);`
    ///   2. Set the global slow clock source (`LSGlobalClkSource::APBClk`).
    ///   3. Configure a timer at ~20 kHz with 8-bit duty resolution.
    ///   4. Create a channel on that timer for GPIO26, another for GPIO27.
    ///   5. Store both channels in `Self` and return it.
    ///
    /// Think about what `new` needs to borrow vs. own — the LEDC peripheral and
    /// the two GPIO pins come out of the `peripherals` box, just like GPIO2 and
    /// UART0 did in `main`. Decide the signature yourself; ask me if the
    /// lifetimes get hairy (LEDC channels borrowing the timer is the tricky bit).
    // pub fn new(...) -> Self { todo!() }

    /// Coast — both inputs low. (C++ `motorStop`)
    ///
    /// TODO: set both channels to 0% duty.
    pub fn stop(&mut self) {
        todo!("both channels -> 0% duty")
    }

    /// Active brake — both inputs high, shorting the motor. (C++ `motorBrake`)
    ///
    /// TODO: set both channels to 100% duty.
    pub fn brake(&mut self) {
        todo!("both channels -> 100% duty")
    }

    /// Drive at a duty in one direction. (C++ `motorDrive`)
    ///
    /// `forward == true`  -> PWM on IN1 (GPIO26), IN2 low
    /// `forward == false` -> PWM on IN2 (GPIO27), IN1 low
    ///
    /// TODO: convert `duty` (0..=255) to a percentage (0..=100) and set the
    /// active channel to that percent while holding the other at 0%.
    pub fn drive(&mut self, forward: bool, duty: u8) {
        todo!("scale duty to percent; PWM the active input, other input low")
    }

    /// Dispatch a parsed command to the right action.
    ///
    /// This one's already written for you — it shows the payoff of the enum:
    /// a single exhaustive `match`. If you ever add a new `Command` variant,
    /// the compiler will make you handle it right here.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Forward(duty) => self.drive(true, duty),
            Command::Reverse(duty) => self.drive(false, duty),
            Command::Stop => self.stop(),
            Command::Brake => self.brake(),
        }
    }
}
