---
title: "The cheap part that came from the wrong hobby"
episode: 10
pubDate: 2026-08-05
sessionDate: 2026-08-05
status: published
teaser: "The last part on the list should have been the easiest — a box that turns 14 volts into 5. Instead it taught me the big number on the label is the one you can't trust, and that my power supply was hiding in someone else's hobby."
seeAlso: [reference/handoff, reference/brain-transplant]
---

Every other part on the order had a measurement behind it — a stall current, a winding resistance, a number I'd earned with a meter. The power supply just had a job: take the battery's `14.4 V` and hand the Pi a clean `5 V`. One line left on the list. How wrong can you go buying a voltage regulator?

Wrong in the exact place I wasn't looking — the current rating — and it took me three near-misses to learn to read it.

First I had to know what I was feeding. A Pi 4 under a real mapping load can pull close to `3 A` on its own, and I'm not just running a Pi — there are two ESP32s hanging off its USB and a LiDAR to power, call it `3.5 A` sustained on the 5 V rail with spikes above. So I wanted a `5 A` supply, headroom baked in.

**The first candidate lied to me with a big friendly number.** A tidy little module with a digital display: "20 W," "5 A." I nearly took it. Then I read the small print — the `5 A` is *peak*, "absolute maximum, not for continuous use." The number it's coy about is the continuous one, around `2` to `3 A`. The giveaway was the `20 W`: at 5 volts that's `4 A` and no further, ever. I'd learned a word the hard way — the spec that matters is **continuous**, not **peak**. And a supply parked at its ceiling is precisely how you get a Pi that reboots at random and quietly corrupts its own SD card. That's a ghost that eats a weekend, and I wasn't inviting it in.

**The second candidate was correct, and cost more than the muscle it was feeding.** The shop's one genuine `5 A`-continuous part was a Pololu regulator — lab-grade, beautifully made, `£38`. That's more than my two wheel drivers *combined*, for the most boring box in the whole build. Correct, and it stung, and I kept looking.

**The third candidate was a trap wearing a bargain.** An `XL4015` module off eBay, "`5 A`," a couple of quid. Read closer and it's three landmines stacked up: it's *non-synchronous* — a catch diode instead of a second transistor, which means it runs hot exactly where I can't afford heat; it's a constant-current *charger* at heart; and it powers up at `20 V` by default. Twenty volts into a Pi is a dead Pi before you've booted it once. Fine on a bench with air around it. Wrong for a sealed chassis that already cooks at 85 °C — the single worst risk hanging over this whole project.

So I stopped shopping and asked a different question. *Who else needs clean 5 volts off a multi-cell battery pack, all day, crammed into something hot?* And the answer wasn't in the Raspberry Pi aisle at all. It was model aircraft. RC pilots have pulled 5 V for their receivers and servos off Lipo packs for twenty years, and they have a name for the part: a **UBEC** — a switching battery-eliminator circuit. Not a lab regulator, not a charger. My exact problem, solved and sold by the thousand, in the wrong hobby.

The one I found: `7 A` continuous, takes `2–7S` packs so my `4S` sits comfortably in the middle, switching design at 92% efficiency, over-temperature *and* reverse-polarity protection, outputs `5.25 V`. **£6.60.** The £38 question had a £6.60 answer, and it was better.

Two things I'd have flinched at a week ago and now like. The `5.25 V` instead of a clean `5.00` turns out to be deliberate and *right* — the official Pi supply is `5.1 V` for the same reason, to win back the volts you lose across the wiring, and the Pi's under-voltage alarm only ever watches the low side. And the `7 A` for a `3.5 A` load is the same lesson the motors already taught me: buy for headroom. Here headroom means it runs at half-throttle and stays cool — the one currency my sealed chassis is chronically short of. The reverse-polarity protection is a quiet bonus too, because I'll be hand-tapping a live `14.4 V` pack and a backwards connection is a *when*, not an *if*.

One note to my future self, scrawled on the box in my head: the day it lands, before it touches anything, put a meter on it and confirm it's really sitting around `5.25`. Trust, then wire.

It's coming slowly from China — `10`–`17 Aug` — which is fine, because it's the slowest part but not the one in my way. Every early step runs off USB power. By the time I'm feeding the robot from its own battery, it'll be on the bench.

And with that, the shopping is *done*. Drivers, co-processors, the logic analyzer, the power rail — every line ordered, nothing left to buy. The next thing that happens in this project isn't a decision or a datasheet or a clever bit of reading. It's a box on the doorstep, and the first time I make one of these motors spin because I told it to.
