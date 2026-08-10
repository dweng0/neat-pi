---
title: "The sensor spoke, then went dark"
episode: 16
pubDate: 2026-08-09
sessionDate: 2026-08-09
status: published
teaser: "It finally counted — 422 edges, hard proof the encoder was alive. Then my laptop died, I unplugged everything to deal with it, plugged it back together — and the sensor never said another word. What followed was hours of ruling everything out until only one culprit was left standing."
heroPhoto: encoder-board-hall-chips-u1-u2.jpg
seeAlso: [reference/handoff]
---

Last episode ended on a question I hated: is this encoder dead, or has it just never been asked properly? This time I got an answer. Two, actually — and the second one undid the first.

First thing I did was fire up the bench Uno that took a 5 V short on the way out last time. It didn't come back. Confirmed dead. Fine — it was only ever scaffolding, and it pushed me toward a better idea: stop trusting the multimeter entirely. A meter can't follow a signal that flips hundreds of times a second; it just smears the whole thing into a meaningless average. Every "it's sitting flat, must be dead" reading I'd taken was the meter shrugging, not the sensor talking. So I handed the job to the ESP32 that's going in the robot anyway: count every edge in an interrupt, print a running tally.

Getting the 5 V sensor outputs safely into a 3.3 V chip that isn't 5 V-tolerant meant a divider, and it fought me — the sensor's own internal pull-up sits in series and drags a textbook `1k+2k` down to a useless `1.9 V`. I had a bag of `330 Ω` at first, far too small; then found the rest of the kit was an assortment, `2K` and `5K1` labelled right on the tape. Tuned it by meter until each channel idled around `3 V`. Not elegant, but right.

Then the payoff. First couple of runs: zero. Then one caught, and the numbers came — `4`, `14`, `101`, `346`, `422` — climbing, steady, direction tracking. The encoder was **alive**. After the wires-talked-sensor-didn't night, it finally had something to say, and I sat there grinning at a column of rising integers like an idiot.

I nearly wrote the victory post right there. I'm glad I didn't.

Because then my laptop battery died. I unplugged the rig to sort it out, plugged it all back together a few minutes later — and the sensor never counted again. Not once. Same motor, same firmware, and I hadn't so much as breathed on the sensor. Dead silent, every speed, both directions, in the gearbox and out of it, even turning the wheel by hand.

Here's where I nearly talked myself into "the chip died" — and where slowing down paid off. Everything I could measure said it should work. `5 V` at the chip, rock-steady even with the motor running under load. Grounds good. A magnet so strong a screwdriver head *sticks* to the disc. And the gap between magnet and sensor is fixed by the motor's own construction — it physically can't have moved.

So I stopped reading and started forcing. I shorted the ESP32's input pin straight to ground by hand: counts flew up. So the chip, the pins, the interrupt, the code — all perfect. Then I shorted the sensor's *own output pad* on the board to ground: counts flew up again. So the entire run — pad, solder joint, wire, divider, GPIO — clean end to end. I had now proven every single link in the chain except one: the sensor's own switching.

And that's when the photo told me the answer. Under the magnet disc sit **two** little chips, `U1` and `U2` — one Hall sensor per channel, either side of the magnet. Both channels had gone dead at the exact same moment. Two independent chips don't both drop dead from a replug; that's absurd. But one **shared solder joint** — the ground or power both of them hang off — cracking? That kills both at once. Which is precisely what happened.

It didn't die. A joint cracked when I handled the thing — unplugged it, reconnected it, wrestled the motor in and out of its gearbox. Invisible, mechanical, and — the good news — fixable with a soldering iron, not a new part.

The lesson I keep relearning: every reading was honest, and every reading was a red herring, because the fault lived somewhere no meter could reach. "Nothing happened" is never true. Something physical always did.

Next session: reflow those joints, wave a proper magnet at each chip to confirm both are still alive, and win my 422 back. The encoder isn't dead. It just lost its grip.
