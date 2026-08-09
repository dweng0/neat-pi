---
name: finish-up-rs
description: >-
  Wrap up a "Learning Rust on the Robot" session — serialise what was learned
  into the next blog episode of the learning-rust-on-the-robot serial AND rewrite
  rust-learning-handoff.md for next time. Writes a first-person devlog episode
  into ./blog (tagged serial: learning-rust-on-the-robot), then overwrites
  rust-learning-handoff.md in place with fresh state and NEXT ACTIONS. Use when
  the user says "finish up rust", "finish-up-rs", "wrap up the rust session",
  "write the next rust episode", or is ending a Rust-learning session.
---

# finish-up-rs — serialise the Rust session and pass the baton

The **end** half of the Rust-learning loop (start half is `cook-rs`). One session, two outputs:

- **The episode** — the public story. A snapshot in time. First-person devlog. Goes in `./blog`,
  tagged `serial: learning-rust-on-the-robot`.
- **The handoff** — the private baton. Current truth. Rolling (overwritten in place):
  `rust-learning-handoff.md`.

Do both, in this order, and **show the user each draft for edits before finalising** — episodes
and handoffs are writing they own.

---

## Site structure (how these episodes are organised)

The site (`site/`, live at **blog.housekeeper.systems**) is a **multi-serial blog**. The Rust
learning is its own serial, **`learning-rust-on-the-robot`**, already registered in
`site/src/lib/serials.ts`.

- An **episode** = one post. Files live in `./blog/NN-title.md`, numbered continuously across the
  *whole* blog (not per-serial) — take the highest `episode:` across all `./blog/*.md` and add 1.
- **These episodes MUST carry `serial: learning-rust-on-the-robot` in frontmatter.** Without it,
  they'd wrongly default to the Neato build serial.
- Only touch `site/src/lib/serials.ts` if changing the serial's registration (already done); you
  normally only write `./blog` and the handoff.

---

## Step 1 — gather what happened this session

Reconstruct honestly from the conversation:
- Which module body/bodies did we work on (`serial.rs::feed`, `command.rs::parse_command`,
  `motor.rs`, wiring `main`)? Did the user write it, or did we?
- What **Rust concept** actually landed this session? (`no_std`/heapless, enums + `match`,
  `Result`/error handling, ownership/borrows, LEDC/PWM, traits.) The learning IS the story here.
- What fought back — borrow-checker errors, toolchain/rust-analyzer snags, esp-hal API confusion?
  **Keep these.** The struggle is the most useful and most relatable part of a learning devlog.
- Did anything get flashed and confirmed on real hardware, or just compile?
- What's the next thing to learn/do?

If the session was thin, say so and offer to skip the episode and just refresh the handoff. Don't
manufacture an episode from nothing.

---

## Step 2 — write the next episode → `./blog`

1. **Find the next number.** List `./blog/*.md`, read frontmatter `episode:` values across all of
   them, take highest + 1. Filename `NN-kebab-title.md` (zero-padded).

2. **Frontmatter** (match existing episodes, plus the required `serial`):
   ```yaml
   ---
   title: "Short, punchy, no episode number in it"
   episode: <N>
   pubDate: <today, YYYY-MM-DD>
   sessionDate: <when the work happened, YYYY-MM-DD>
   status: published
   teaser: "A hook, not a summary — hint the concept or the struggle."
   heroPhoto: some-photo.jpg   # optional — a bare filename from ./photos, omit if none fits
   serial: learning-rust-on-the-robot   # REQUIRED for this thread
   ---
   ```
   `heroPhoto` is raw/uncompressed — the site's image pipeline handles webp + sizes at build. Drop
   the full-res file in `./photos` and name it; don't pre-shrink.

3. **Voice — match `./blog/*.md` (read one first so tone carries):**
   - **First person, past tense, plain-spoken.** "I stared at the borrow error for ten minutes…"
   - **Honest about the struggle.** A beginner learning in the open — the confusions, the wrong
     turns, the moment it clicked. Don't present it as if you knew it all along.
   - **Teach one idea per episode.** Show the actual code — the `todo!()` you filled, the error the
     compiler gave, the fix. Keep code snippets short and real.
   - **Specs/APIs as inline `code`.** `no_std`, `heapless::String`, `Command::Forward(u8)`, `LEDC`.
   - **End on a hook** — the next body to tackle or concept to meet. It's a serial; leave a thread.
   - Roughly 400–800 words. A learning devlog, not a tutorial or a lab report.

4. **The one hard rule: never fake the learning.** Don't claim code works if it only compiled;
   don't claim it was flashed if the board wasn't plugged in; don't smooth a struggle that
   happened into a clean success. "Compiles" ≠ "works on hardware" — carry that distinction, it's
   the same honesty brand as the main serial.

---

## Step 3 — rewrite the rolling handoff → `rust-learning-handoff.md`

**Overwrite in place** (rolling baton, not an append log — git is the history).

Keep the existing structure (context, principle, where-things-stand incl. the module status
table, **NEXT ACTIONS**, blog serial outline, links). Update to reflect this session:
- Bump **Last updated** to today.
- Update the **module status table**: move any body from `todo!()` → done, note if `main` got
  wired, note anything flashed/confirmed on hardware.
- Rewrite **NEXT ACTIONS** so the top is genuinely the next thing.
- Tick off / adjust the **blog serial outline** if an episode was written.

The test: a fresh `cook-rs` tomorrow could read this and know exactly which body to write next.

---

## Guardrails (the trust model)

- **Never fake progress.** `todo!()` still there = not done. Compiled ≠ works. Unknown stays
  visibly unknown.
- **Episode = snapshot in time. Handoff = current truth.** An old episode can read as naive later
  (that's fine, it's dated and it's a *learning* log); the handoff must always be "now".
- **Only `finish-up-rs` writes**, and only to `./blog` and `rust-learning-handoff.md` (and the
  already-done serial registration). Never touch the `site/` build output.
- **The user owns the writing.** Show both drafts before finalising.
- **Publishing = commit + push.** Writing files updates local content; the live site rebuilds only
  on push to `main`. Don't commit/push unless asked — offer it as the final step.
