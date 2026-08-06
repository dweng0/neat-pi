---
title: "Buying it was supposed to be the easy part"
episode: 9
pubDate: 2026-08-05
sessionDate: 2026-08-05
status: published
teaser: "The measuring was done, the list was final. All I had to do was click buy — and discovered that every line on it had a lookalike waiting to trip me. Then I turned the robot over and found the next project staring up at me."
heroPhoto: under-carriage-sensor.jpg
seeAlso: [reference/handoff, reference/brain-transplant]
---

Last episode ended with a clean list and a promise: the teardown's finished, the next box is the ESP32, all that's left is to hit buy. Five motors, four drivers, one buck converter. I even had the part names — `2× DRV8871`, `1× Cytron MD10C`, `1× logic-level MOSFET`, a buck. How hard is a shopping trip?

Turns out a bill of materials and a shopping cart are different documents. The BOM says *what*. The cart makes you say *exactly which one*, and that's where the lookalikes live.

**The first "Cytron" wasn't a Cytron.** I found a listing called a "Light & Motor Driver," 10 A, looked plausible, nearly dropped it in the basket. Then I read the spec properly: it's driven over **UART and MODBUS**, controlled by a potentiometer or a USB host, and it's a single MOSFET — *unidirectional*. That's three problems in one. My whole plan is an ESP32 toggling a PWM and a direction pin in real time; bolting a serial-protocol board onto that is a translator I don't need. And unidirectional kills the one trick I was most pleased with — reversing the roller to cough out a hairball. It was a motor driver wearing another motor driver's name. The real one — Pihut's "13A 6V-30V DC Motor Driver," which is actually a `MD13S`, the `MD10C`'s newer sibling — is bidirectional, takes a plain `3.3 V` logic pin, and does `20 kHz` PWM. That's the one. The lookalike went back on the shelf.

**The side-brush MOSFET had the opposite trap.** Last episode I warned myself off the `IRF520` because a `3.3 V` pin can't switch it. The board I settled on — a Gravity MOSFET module — passes that test: it triggers cleanly from `3.3 V`. But reading the fine print, its switching tops out at `1 kHz`. Fine for a brush I only ever turn *on* or *off*; useless if I'd wanted quiet high-frequency speed control. For the side sweeper, on/off is all I need, so it stays. One note to my future self scrawled in the margin: it's a bare switch, so the little brush motor needs a flyback diode across it or the coil's collapse will bite the FET. I've got a drawer of them from the Elegoo kit.

**And the buck converter tried to lie to me with a number.** I found a tidy little module with a display, "20 W," "5 A." Read closer: `5 A` is the *peak*, the "absolute maximum, not for continuous use." The number it's shy about is the continuous one — around `2` to `3 A`. My Pi 4 alone can pull `3 A` under a mapping load, before I've added two ESP32s and a spinning LiDAR. A supply living permanently at its ceiling is exactly how you get a Pi that reboots at random and corrupts its own SD card — the kind of ghost that eats a weekend. The honest word to shop for turns out to be **continuous**, not peak. The one part on the whole list that's genuinely `5 A` continuous is a Pololu regulator, and it's *£38* — more than the two wheel drivers put together. That one I'll source elsewhere; a proper `5 A` RC UBEC does the same job for a fifth of the price, as long as it says *continuous* and means it.

So the order went in — everything but the buck and a little speaker amp I'll pick up separately. `DRV8871` ×2, the real Cytron, the Gravity MOSFET. The boring parts were boring; the traps were all in the parts that had a plausible twin.

**Then I turned the robot over, and the next project looked back at me.**

I've been so deep in motors I'd filed "sensors" under *later*. But there they were, and I started pulling them and reading part numbers. A little board stamped `BUMP SWITCH 290-0056` — a plain mechanical click-switch, and there are four of them across the front bumper. A lever microswitch, `DT-08`, tucked in a wheel arch: that's the dead-man's switch that knows when a wheel has dropped into thin air, i.e. someone's picked the robot up. Both of those are the easy kind — a contact closes, an ESP32 pin reads high or low, done. No driver, no analog, barely any thought.

The one that matters is underneath: a small board reading `LOUIE DPP SENSOR 290-1023 REV 2`, an infrared eye that stares at the floor. That's a **cliff sensor** — the thing that stops the robot cheerfully driving off the top step. It doesn't give a clean high/low; it gives a reflectance reading that I'll have to feed an analog pin and threshold in software. I haven't pulled the whole undercarriage yet, so I don't know how many there are or whether they run at `3.3` or `5 V` — both go on the meter next time. This is the sensor I least want to get wrong. Everything else being buggy means a robot that bumps things. This one being buggy means a robot at the bottom of the stairs.

Oh — and there's a speaker at the back. It has no business being interesting and I want to wire it up anyway. A tiny class-D amp, an ESP32 pin, and the thing can announce when it's stuck or docked or full. That's pure dessert, and it's going on the list.

The good news buried in all this: the sensors add *zero* new drivers. Switches are free, cliff eyes just need an analog pin. The order I placed today still covers the muscle; the sensors are all signal.

Next box is still the ESP32. But now there's a second thread waiting for a meter — how many cliff eyes, and what voltage do they want — before I trust this thing anywhere near a staircase.
