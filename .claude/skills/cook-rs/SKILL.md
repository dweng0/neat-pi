---
name: cook-rs
description: >-
  Resume the "Learning Rust on the Robot" thread and pair on it by voice. Reads
  rust-learning-handoff.md, briefs the user on Rust-learning progress (which
  module bodies are done vs still TODO) and the NEXT ACTIONS, then drops into a
  hands-free voice pairing session over the live Neovim buffer. Use when the user
  says "cook rs", "cook-rs", "lets cook rust", "resume rust", or opens a session
  to continue learning Rust on the ESP32 firmware.
---

# cook-rs — resume the Rust-learning session, then pair

This is the **start** half of the Rust-learning loop, and it fuses two things the user already
uses: the `cook` resume-brief pattern and the `pair-up` voice-over-Neovim pattern. Its job: get
oriented on the Rust thread fast, then get *into the code together by voice*. The end half is
`finish-up-rs`.

## What to do

1. **Read the rolling handoff** — `rust-learning-handoff.md` at the repo root. This is the baton
   for the Rust thread specifically (separate from `neato-d10-handoff.md`, which is the main
   build). It holds current state: toolchain status, which milestone/module bodies are done vs
   still `todo!()`, and the NEXT ACTIONS. Treat it as the source of truth for "where the learning
   is right now."

2. **Brief the user, tightly.** Lead with NEXT ACTIONS. A good brief:
   - **One line of context** — that this is the Rust port of the ESP32 firmware, learning-focused.
   - **Where things stand** — what compiles, what's flashed, and crucially *which module bodies
     are still TODO* (`serial.rs::feed`, `command.rs::parse_command`, `motor.rs`). The user is
     writing these themselves — be precise about what's done vs waiting.
   - **NEXT ACTIONS** — reproduce the handoff's list in order, flagging what's doable now vs
     blocked (e.g. flashing is blocked until the board's plugged in).
   - **The single most useful thing to do next**, called out plainly.

3. **Then start pairing by voice.** This is the key difference from `cook`. After the brief,
   invoke the **`pair-up`** skill (voice in/out via voicemode `converse`, live Neovim buffer via
   the neovim MCP). From there, follow `pair-up`'s loop: every reply is a `converse` call, drive
   the editor with `vim_*` tools on request, and teach as you go. Open the file for whichever
   NEXT ACTION the user picks (e.g. `src/serial.rs`).

4. **Peek deeper only if needed.** The C++ reference is `esp32-firmware/src/main.cpp`; the module
   comments in `esp32-firmware-rs/src/*.rs` spell out expected behaviour and hints. Read them when
   a specific body needs working through — don't dump them into the brief.

## Rules

- **Don't write handoff/blog files in `cook-rs`.** Reading and briefing only; writing state back
  is `finish-up-rs`'s job. (Editing the user's *source code* during pairing is fine and expected —
  that's the whole point — but only when they ask.)
- **The user is learning — don't just write their TODO bodies for them.** Default to guiding:
  explain, hint, let them type. Write code for them only if they explicitly ask. When they're
  stuck, nudge toward the answer rather than dropping the solution.
- **Never invent state.** If the handoff doesn't say a milestone is confirmed, it isn't. Flashing
  to hardware is the confirmation bar — "compiles" is not "works on the bot."
- **Toolchain reminder:** builds need `. $HOME/export-esp.sh` sourced and `~/.cargo/bin` on PATH;
  build with `cargo build`, flash + monitor with `cargo run` (from `esp32-firmware-rs/`).
- Keep the brief short. The user wants to be in the editor talking through code, not reading a
  summary.

## Site note (context, not part of the brief)

The public devlog is a **multi-serial blog** (`site/`, live at blog.housekeeper.systems). The
Rust learning is its own serial, `learning-rust-on-the-robot`, registered in
`site/src/lib/serials.ts`. `cook-rs` doesn't write episodes — that's `finish-up-rs`.
