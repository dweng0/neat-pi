---
title: "The motors won't tell you their limits"
episode: 4
pubDate: 2026-08-04
sessionDate: 2026-08-04
status: published
teaser: "Four motors, a cheap multimeter that can't be trusted, and one smug fan that prints its own numbers and needs no help at all."
heroPhoto: roller-brush-motor.jpg
seeAlso: [reference/measuring-motor-current, reference/handoff]
---

To spin a motor from a microcontroller you need a motor driver — an H-bridge or a MOSFET — because an ESP32 GPIO pin puts out milliamps at 3.3 V. That's enough to *signal* a decision, nowhere near enough to *spin a motor*. The driver is the muscle: it switches the fat battery current on and off under the ESP32's command. (There's no separate "driver for the ESP32" — the ESP32 just runs. The drivers are for the motors.)

So I need drivers. Which drivers depends on current. And here's the rule that governs the whole purchase: **you size a motor driver to stall current, not running current.** A motor pulls far more when it's jammed — brush wrapped in hair, wheel against a wall — than when it's happily spinning. Size for the happy case and the first time the robot gets stuck, the driver dies.

I have four motors. Good news up front: only three need drivers.

**The blower solved itself.** It's that EVERFLOW `F121225BU` with `DC14.4V 2.0AMP` printed right on the label — so no measurement needed, the current's given. But the better news is in the part number. The `…BU` suffix is Everflow's four-wire PWM family: it's **brushless with an integrated driver already inside it.** I feed it 14.4 V and a PWM signal straight off an ESP32 GPIO (around 25 kHz), and I even get a tach line back to read its RPM — which means I can *detect a clog* by watching the fan bog down. No H-bridge. In fact you must **not** put an H-bridge on it — trying to reverse a brushless fan just confuses its internal controller — and you must not measure its winding resistance either, because you'd be probing driver electronics, not a coil. It's the one motor that needs nothing from me. Smug little thing.

That leaves **two drive wheels and the roller brush** to size. And this is where my multimeter let me down.

The plan was going to be: power each motor, jam it, read the current spike. Except my meter is an MS8233A — 2000 counts, and critically its 10 A jack is **unfused** and rated for at most 30 seconds every 15 minutes. A stall inrush is a millisecond spike. A 2000-count handheld can't catch it, and I'm not keen on shoving a stall current through an unfused jack to try. That's how you let the smoke out — of the meter, maybe of me.

So I changed method. Instead of measuring stall current directly, I'll **measure winding resistance with no power applied** and calculate the stall from Ohm's law: `stall ≈ 14.4 V ÷ R`. Safer — nothing's live — and honestly more trustworthy than a live stall reading this meter could never catch cleanly. The technique needs care (short the probes first to subtract lead resistance; rotate the shaft and take several readings because the commutator position swings the number; keep the lowest), but it's a bench job I can do right now with what's in my hand.

The provisional driver shortlist, pending those numbers:

- **Two wheel motors → one TB6612FNG**, a dual H-bridge that's happy with 3.3 V logic and good for a bit over an amp per channel. *If* the measured stall comes in at 3 A or more, I step up to a pair of DRV8871s instead.
- **Roller brush → one BTS7960 or a MOSFET module.** It only spins one direction, so it doesn't need a full H-bridge.
- **Blower → nothing**, as established.

And I'm deliberately *not* buying any of them until I've measured. Buying now is guessing, and guessing at stall current is how you buy drivers that melt.

There's a nice bonus hiding in my parts bin, too. My Elegoo starter kit has an L293D — far too weak to be a final driver, but perfect as a bench rig to spin a wheel motor gently and sanity-check things. And it has a couple of PN2222 transistors and flyback diodes, which happen to be *exactly* the classic circuit for driving the LiDAR's spin motor. So when the LiDAR bring-up comes, I've already got the parts to make it turn.

Next actions from here are refreshingly concrete: get the meter in resistance mode and measure three windings; count the wires in the wheel harness (six means a direction-aware quadrature encoder, which I want; five means direction-blind, which complicates odometry); and confirm the blower really does have four wires and not two. Then the parts I've ordered arrive, and this stops being a teardown and starts being a build.
