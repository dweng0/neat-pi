# Rust-on-the-robot — rolling handoff

> The baton for the **"Learning Rust on the Robot"** thread. Current truth, not history.
> `cook-rs` reads this on resume; `finish-up-rs` overwrites it in place at session end.
> Git is the history — don't append dated copies.

**Last updated:** 2026-08-09

---

## Context

The Neato's ESP32 co-processor firmware exists in C++ (`esp32-firmware/`, known-good, drives
the wheel motor via a DRV8871). The user has written Rust before but always at arm's length
(`String` everywhere, `.clone()`/`.unwrap()` to appease the compiler); this thread is about going
*deeper* — earning the ownership model, working `no_std`, understanding *why* the rules bite. The
vehicle is porting the firmware to bare-metal `no_std` Rust in a **sibling** project,
`esp32-firmware-rs/`, leaving the C++ untouched as a fallback. Documented publicly as its own blog
serial (see below), separate from the main brain-transplant build story.

> **Framing note:** treat the user as an experienced dev going deep, NOT a raw beginner. Don't
> over-explain basics they know; do slow down on the deep stuff (borrows/lifetimes, `no_std`,
> peripherals). The serial's public premise matches this ("going deeper on purpose").

**Crate layout (as of 2026-08-09):** the pure, hardware-free logic has been split out of the
firmware into its own crate so it can be unit-tested on the host:

```
Neato/
├─ esp32-firmware-rs/   ← firmware: talks to the chip, depends on esp-hal + protocol
│  └─ src/  bin/main.rs, lib.rs (pub mod motor only), motor.rs
└─ protocol/            ← pure logic, no hardware; deps = heapless only; builds on the Mac
   ├─ Cargo.toml
   └─ src/  lib.rs (#![cfg_attr(not(test), no_std)]), serial.rs, command.rs
```

`protocol` MUST sit **beside** `esp32-firmware-rs`, not inside it — the firmware's `.cargo/config`
forces the Xtensa target + `build-std` on everything below that folder, so a nested crate would
inherit it and `cargo test` would try to build for the chip. As a sibling it uses plain host
defaults; `cd protocol && cargo test` just works (no `--target` needed). Firmware depends on it
via `protocol = { path = "../protocol" }`; `motor.rs` imports `use protocol::command::Command;`.

**Guiding principle:** build the concrete thing working on the real robot first, *then* extract
the reusable library. No premature abstraction.

---

## Where things stand

**Hardware:** classic ESP32 (ESP-WROOM-32, Xtensa LX6 dual-core) on a DevKitC board with a
CH340C USB-serial bridge. GPIO26→IN1, GPIO27→IN2 (DRV8871), GPIO2 onboard LED, UART0 on GPIO1/3.

**Toolchain (confirmed working):**
- `espup` installed the `esp` Rust toolchain fork (Xtensa needs it — not on upstream Rust).
- `espflash` for flash + monitor. `esp-generate` scaffolded the project.
- `. $HOME/export-esp.sh` must be sourced per shell (or add to `~/.zshrc`).
- rust-analyzer: server component installed on `stable`; project routes analysis through `esp`
  via `rust-analyzer.toml` + `.nvim.lua` (uses `RUSTUP_TOOLCHAIN`, not `RUST_TOOLCHAIN`).
- Build: `cargo build`. Flash + monitor: `cargo run` (runner = `espflash flash --monitor`).
- **Simulator (Wokwi):** a virtual ESP32 is set up (`wokwi.toml` + `diagram.json`, how-to in
  `esp32-firmware-rs/WOKWI.md`). It runs the firmware with no hardware and supports step-debugging
  over a gdb server — the no-JTAG-probe path. See WOKWI.md for run + debug commands.

> **Testing policy — read this before touching hardware.** Prefer the Wokwi simulator to smoke-test
> firmware (does it boot, does the loop run, does serial behave) *before* flashing a physical ESP32.
> Any agent or session should run the sim first; only go to real hardware once the sim looks right,
> or when validating something the emulator can't model (the real DRV8871, motor stall current,
> precise PWM timing). "Runs in Wokwi" ≠ "works on the real motor" — keep that distinction.

**Milestone 1 — heartbeat + serial echo:** ✅ code written, compiles clean.
`src/bin/main.rs`. LED toggles at 1 Hz, serial echoes bytes. **Not yet flashed** — board wasn't
plugged in. First real-hardware confirmation still pending.

**Milestone 2 — motor control, split into a pure `protocol` crate + hardware `motor`:** 🚧 in progress.
| File | What it does | Status |
|---|---|---|
| `protocol/src/serial.rs` | `LineReader` — bytes → whole lines (heapless buffer) | ✅ `feed()` DONE + 4 host tests pass |
| `protocol/src/command.rs` | `Command` enum + `parse_command()` (F/R/S/B protocol) | `parse_command()` is a `todo!()` |
| `esp32-firmware-rs/src/motor.rs` | `Motor` — DRV8871 via LEDC PWM; `apply()` written | `new`/`stop`/`brake`/`drive` are TODOs |

`feed()` is implemented and genuinely tested on the host — the trick was a `complete` flag that
clears the buffer at the *start* of the next `feed` rather than on newline, dodging the "return a
borrow of `self.buf` while also clearing it" borrow-checker conflict. Tests live in
`protocol/src/serial.rs` under `#[cfg(test)] mod tests` (run: `cd protocol && cargo test`).

`main.rs` still runs milestone 1 untouched, so it always flashes. `parse_command` and the `motor`
bodies are still `todo!()` and not yet wired into `main`. **Both crates build clean** —
`cd protocol && cargo test` (4 pass, host) and `cd esp32-firmware-rs && cargo build` (Xtensa).

---

## NEXT ACTIONS (top = do first)

1. **Fill in `protocol/src/command.rs::parse_command`** — pure logic, and now the *satisfying*
   part: it lives in the host-testable `protocol` crate, so you can TDD it. Enum + `match` +
   `u8` parse (see the doc comment for the F/R/S/B rules and hints). Add tests alongside the
   `serial` ones under `#[cfg(test)] mod tests`, run with `cd protocol && cargo test`.
2. **Fill in `esp32-firmware-rs/src/motor.rs`** — the boss fight. LEDC peripheral, two ~20 kHz
   channels on GPIO26/27, scale duty 0..=255 → percent. This one can't be host-tested (it owns
   the chip), so it's Wokwi/hardware territory. This is where the esp-hal docs get read for real.
3. **Wire `main.rs` to use the modules** — loop becomes: read byte → `protocol::serial` `feed`
   → `protocol::command::parse_command` → `motor.apply`. Retire the echo.
4. **Smoke-test in Wokwi, THEN flash milestone 1 for real** (still pending — board wasn't plugged
   in). `cd esp32-firmware-rs && cargo run`; confirm banner, `.` per second, D2 blinking, echo.
   (If no serial port: CH340 driver.) Real hardware is still the outstanding green light — nothing
   has run on a physical ESP32 yet.
5. **Later:** extract the trait (`MotorController` w/ forward/reverse/stop/brake). The pure-module
   extraction into `protocol/` is already done (this session) — the trait is the remaining refactor.

---

## Blog serial outline — "Learning Rust on the Robot"

Registered in `site/src/lib/serials.ts` as slug `learning-rust-on-the-robot`. Episodes are
**numbered per-serial from 1** (NOT continuous across the whole blog — the Neato serial is at
15+; this serial starts at 1). Files: `./blog/rust-NN-title.md`, frontmatter
`serial: learning-rust-on-the-robot` + `episode: N` (per-serial N). Next number = highest
`episode:` among files carrying this serial + 1.

Episode arc (write them as the work actually happens — honest, not idealised):
- ✅ **Ep 1 — "The line I couldn't give back"** (`rust-01-...`, written 2026-08-09): the `feed`
  borrow-checker fight (returning a borrow while clearing) + hitting the wall that you can't
  `cargo test` firmware, leading to the `protocol` crate split. Covered no_std/heapless, the
  borrow lesson, AND the concrete-then-extract move in one.
- **Next — Enums and the parser**: modelling F/R/S/B as a `Command` enum; exhaustive `match`;
  the compiler as a helper. Now TDD-able in the `protocol` crate.
- **The boss fight: PWM on bare metal** — LEDC, the DRV8871 truth table, borrowing the timer.
- **First light on real hardware** — the still-owed moment: flashing to a physical ESP32.
- **From "it works" to "it's a library"** — the `MotorController` trait extraction. (The pure-logic
  crate split already happened in Ep 1, ahead of schedule, driven by wanting to test.)

---

## Links
- C++ reference firmware: `esp32-firmware/src/main.cpp`
- Rust project: `esp32-firmware-rs/`
- Main build handoff (separate thread): `neato-d10-handoff.md`
