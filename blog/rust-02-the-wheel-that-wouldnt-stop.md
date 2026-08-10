---
title: "The wheel that wouldn't stop"
episode: 2
pubDate: 2026-08-10
sessionDate: 2026-08-10
status: published
teaser: "I built a little tool to ask the robot a question. It answered — and then, when I told it to stop, it didn't. Turns out Ctrl-C kills your program, not the thing on the other end of the wire."
heroPhoto: encoder-board-hall-chips-u1-u2.jpg
serial: learning-rust-on-the-robot
---

Different track this time. Instead of porting firmware *onto* the robot, I wanted a tool on my Mac that could *talk* to it — send a drive command down the serial cable and print back whatever the robot said. The real question underneath was simple: is the wheel's hall sensor actually alive? I've been staring at that encoder board wondering if it's reporting anything at all. So: `cargo run -- F 180`, watch what comes back. A probe. And since I'm doing all of this the hard way on purpose, I typed every line myself.

The send path taught me three things in a row, all of them small, all of them the kind of thing you'd `.clone()` your way past and never learn.

First: I wanted to translate my command letter. My tool takes `F` or `B` on the command line, but the firmware's word for reverse is `R` (to it, `B` means *brake*). Easy — a `match`:

```rust
let fw = match direction {
    "F" => "F",
    "B" => "R",
    _ => "F",
};
```

The compiler stopped me: `expected &String, found &str`. And this is the bit that finally clicked. `direction` came from the command-line args, so it's a `&String` — an owned, heap-backed string I'm holding a reference to. But `"F"` and `"R"` are `&str`, string *slices*, baked into the binary. When I call a normal function, Rust quietly converts one to the other for me; a `match` won't. So I added `.as_str()` — `match direction.as_str()` — and the patterns lined up. I've written `&str` versus `String` a hundred times without ever feeling the seam. This time I felt it.

Second: I `match`ed on the result of opening the port to get the port out, and then, a few lines later, tried to print the opened value again — and got told it was *moved*. The `match` had consumed it. In C++ that value would still be sitting there, half-alive; in Rust it's just *gone*, and the compiler knows exactly where it went. Fine. Deleted the line. Moving on.

Third: sending. `port.write_all(...)` — "cannot borrow `port` as mutable." I hadn't declared it `let mut`. My reflex was *but I'm not changing anything* — and then the penny dropped. I'm not reassigning `port`, no, but **writing to the port changes the port**: it shoves bytes into the OS buffer, advances internal state. A method that mutates the thing it's called on takes `&mut self`, and you can't call that on a binding you swore was immutable. `let mut port`, done.

Then the reading half: a fixed 256-byte buffer, a loop, `port.read(&mut buf)` filling it and handing back how many bytes it got, `String::from_utf8_lossy` to turn those raw bytes into text. And it *worked* — the robot answered:

```
[motor] fwd duty=180
```

The full round trip. My laptop said a thing, the robot did it and reported back, my loop caught the reply. But no encoder numbers. Turns out this firmware only reports the encoder when you *ask* — an `E` command — it doesn't just stream it. So I sent an `E` on every pass of the loop, and *there* were the counts:

```
[enc] A=62 B=0 pos=0 (levels: A=1 B=1)
[enc] A=62 B=0 pos=0 (levels: A=1 B=1)
```

Here's where I have to be honest, because this is a learning log and not a highlight reel: the plumbing is perfect and the answer is *murky*. Channel A sat frozen at 62 the whole time and never ticked up; channel B was flat zero. So my tool works beautifully and the data it fetched says the sensor might not be. That's a hardware puzzle for another night — but at least now I have an instrument to poke it with.

And then the bit that actually rattled me. I changed my fixed loop to `loop {}` so it'd run forever, hit `Ctrl-C` when I'd seen enough — and the wheel kept spinning. `Ctrl-C` killed *my program*. The robot never heard a thing. It was still happily executing the last order I gave it, a dead process's final word, no one left to say stop. I had to open another shell and shove an `S` straight at the serial device to make it quit.

That's the thread I'm pulling next. `Ctrl-C` shouldn't just execute me — it should let me get a last word out, an `S`, *then* exit. In Rust that's a signal handler flipping a shared flag the loop watches, which drags in a couple of ideas I've dodged forever: sharing one value between the handler and the loop, and doing it safely. The wheel that wouldn't stop is about to teach me ownership across threads.
