---
title: "It only answers in motion"
episode: 19
pubDate: 2026-08-11
sessionDate: 2026-08-11
status: published
teaser: "I set out to test the encoder sensors properly and got three flat, identical readings — including one from a motor I wasn't even studying. Three sensors don't die the same way. The problem was never the hardware; it was that I'd been asking a motion sensor to answer while standing still."
heroPhoto: encoder-board-hall-chips-u1-u2.jpg
seeAlso: [reference/handoff]
---

Episode 18 ended clean: circuit rebuilt, both hall channels proven, one suspect left — the
disc. Tonight was meant to be a fifteen-minute victory lap: plug the second motor's encoder
into the proven rig, hand-turn its disc, watch it count, pocket a working spare.

It did not go like that.

First, a fair question I should have asked weeks ago: do I even need these encoders? I have
a LIDAR. But LIDAR-only odometry drifts badly during pure rotation — and a vacuum turns
*constantly* — and it leans harder on the Pi. Wheel odometry is the backbone; the LIDAR
corrects its drift. So yes, I need them. Onward.

Meter on the sensor, power confirmed — `5.00 V` at the pads, both signal lines idling at
`4.40 V`. Textbook. Then I pressed a magnet to the chips and… nothing. Swapped to the
second motor — my *clean control*, the disc I'd been careful never to wave a magnet near.
Hand-turned it slowly. Nothing. `4.40 V`, flat.

That was the moment the "I wiped disc #1 myself" theory died. One wiped disc, fine. Two
independent discs, both silent? That's not a coincidence, that's a pattern — and a pattern
usually means *I'm* the common factor.

So I went back to first principles and re-verified the wiring from scratch, sure I'd crossed
something. I hadn't. brown is Vcc, orange is ground, blue and yellow are the two outputs —
and the proof was in the readings: two signal lines sitting *together* at `4.40 V`, just
below the rail. Crossed wires can't produce that. The wiring was innocent.

Then I got out the big gun — a proper neodymium magnet — and waved it at the sensors. Still
nothing. And that's when I noticed something: the roller-brush motor has a hall sensor too,
a little tacho on three wires. A third sensor, on a third motor, one I hadn't touched all
day. I wired it up. Same `4.40 V`. Same silence.

Three sensors. Three motors. Identical dead-flat readings. At that point you have to stop
blaming the hardware. Three things do not break the same way by chance — so the fault was
mine, in how I was *looking*.

I finally did the thing I should have done first: I searched. And the answer reframed the
whole week. The Neato wheel encoder, the community threads told me, is built on Allegro
`A3423`-family hall sensors — *dynamic, differential* devices. Mine are two little three-pin
chips rather than one dual-channel part, so I can't swear to the exact number etched on
them. But the behaviour is unmistakable, and dynamic-differential explains every symptom:
**this kind of sensor does not respond to a static magnet. By design.** It only fires on a
*changing* field, as poles sweep past it. Hold a magnet on it dead still and you get exactly
what I got all day — nothing.

Everything snapped into place. The reason a magnet gave me edges last episode was that I was
*moving* it in and out — a changing field. The reason a held magnet does nothing is that
there's nothing to change. And the reason my meter reads a flat `4.40 V` when the disc spins
is that the pulses are real but far too fast for a three-readings-a-second multimeter to
show — it just averages them into a blur. I'd built a whole day of tests that a sensor of
this type physically cannot answer.

Which stings, because the robot kept trying to tell me. Somewhere back there I'd asked
myself "does it need to be spinning?" and talked myself out of it — hall sensors read static
fields, everyone knows that. Everyone's wrong about *this* kind.

So: you cannot meter these with a DC voltmeter. Not the sensors' fault, not the discs' fault
— the tool and the method were wrong. The verdict from two episodes ago ("weak disc") and
tonight's brief panic ("three dead sensors") are probably both wrong. The sensors are likely
fine. I've just never once given one a test it could actually pass.

The right test is the boring one I keep deferring to: the logic analyzer, clipped to A and
B, motor spinning, watching the raw square waves appear — or not. It's already in the post.
And if the chips really are gone, the fix is an `A3423`, not the linear hall sensors I
nearly bought — a proper dynamic sensor that reads the disc I already own.

I closed a lot of doors today. Just not the one I set out to close.
