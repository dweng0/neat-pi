---
title: "The line I couldn't give back"
episode: 1
pubDate: 2026-08-09
sessionDate: 2026-08-09
status: published
teaser: "I had one tiny function to finish — turn bytes into a line. The compiler refused it twice, and then, when I tried to write a test to prove it worked, refused that too. Both refusals were right."
heroPhoto: esp32-devkit-pinout.jpg
serial: learning-rust-on-the-robot
---

The C++ firmware already does this. It reads a byte, stashes it, and when it sees a newline it hands you the whole line to parse. Fifteen lines, no drama. I sat down to write the same thing in Rust — a `feed` function on a little `LineReader` — and got stuck on something so small I was almost embarrassed to admit it: I couldn't work out how to give the finished line back.

Here's the shape. On each byte, if it's a `\n` the line is done, so I return it and empty the buffer for the next one. My first instinct:

```rust
b'\n' => {
    self.buf.clear();
    Some(self.buf.as_str())   // ...returns an empty string. Genius.
}
```

Obviously wrong once you say it out loud — I clear it, *then* hand back the nothing that's left. Fine, flip the order:

```rust
let line = self.buf.as_str();  // borrow the buffer
self.buf.clear();              // now mutate it — NO.
Some(line)
```

And this is where Rust stops being C++ with different punctuation. `line` is a *reference into* `self.buf`. `clear()` wants to mutate `self.buf`. You cannot mutate a thing while a live reference into it is still hanging around — the borrow checker won't have it. In C++ nobody would stop me; I'd get a dangling view into a buffer I just wiped and find out at 2am. Rust just... refuses. The error is the feature.

The fix took a minute to see and then felt obvious. Don't clear on the way *out* — clear on the way *in*. I added a `complete` flag. When I hit the newline I set the flag and return the buffer as-is, borrow intact, nothing mutated. Then at the very *top* of the next `feed`, before touching anything, I check the flag and wipe the buffer then. The clear and the return never happen in the same breath, so there's no borrow to conflict with:

```rust
if self.complete {
    self.buf.clear();
    self.complete = false;
}
```

Compiled. And here's the thing I keep relearning: the borrow checker wasn't being pedantic, it was pointing at a real lifetime problem I'd have shipped without noticing.

So `feed` was done. And I wanted to *prove* it — a couple of tiny tests, feed it `"F 180\n"`, check I get `"F 180"` back; feed a `\r` in the middle, check it's ignored; feed two lines, check the first doesn't bleed into the second. That's when I hit the second wall.

`cargo test` didn't run. This whole crate is built for the ESP32 — a `no_std`, Xtensa target my Mac physically can't execute. And worse, the crate pulls in `esp-hal`, the hardware layer, which won't even *compile* for a laptop. You can't test firmware by pretending your MacBook is a microcontroller.

The escape was to notice that `feed` doesn't know anything about the chip. Neither does the command parser. They're pure logic — bytes in, text out — leaning on nothing but `heapless`. So I pulled those two files out into their own little crate, `protocol`, that has no hardware dependencies at all. Plain Rust. Builds anywhere. The firmware now depends on it; the motor code, which *does* touch pins, stayed behind.

One trap on the way: I first tucked `protocol` *inside* the firmware folder, and the tests still tried to build for the chip. Turns out the firmware carries a hidden `.cargo/config` that forces the Xtensa target on everything beneath it. Anything nested inside inherits it. Moving `protocol` to sit *beside* the firmware instead of under it fixed it instantly — out of the folder, out from under the config.

Then:

```
running 4 tests
test serial::tests::buffer_resets_between_lines ... ok
test serial::tests::carriage_return_is_ignored ... ok
test serial::tests::returns_none_until_newline ... ok
test serial::tests::yields_whole_line_on_newline ... ok
```

Four green, running on my Mac in a hundredth of a second, no board plugged in. The one I care about most is `buffer_resets_between_lines` — it's the test that would've caught the borrow bug's evil twin, one line leaking into the next.

None of this has touched real hardware yet — it compiles for the chip and the logic is tested on the host, which is not the same as "works on the robot." That green light is still owed. Next is the command parser: turning `"F 180"` into a real typed `Command`, an `enum` and a `match`, which everyone tells me is the part where Rust starts to feel good. After that, the boss fight I keep circling — `LEDC`, PWM, and actually driving the DRV8871.
