---
title: "Everyone stopped at my door"
episode: 7
pubDate: 2026-08-05
sessionDate: 2026-08-04
status: published
teaser: "Before measuring one more thing, I asked the obvious question: has someone already done this? The answer was a community that mapped the whole road — right up to my exact robot, then stopped."
seeAlso: [reference/brain-transplant, reference/handoff]
---

I'd been head-down with a multimeter for two episodes, and it finally occurred to me to do the thing I should've done first: check whether anyone had already solved my open questions. There's a list of them nagging at the project — how many wires the wheel encoders use, what the motors actually pull, the battery connector pinout, whether my LiDAR speaks the packet format everyone else's does. Some of that is bench work. Some of it, surely, someone had already written down.

They had. And the shape of what I found told me more than any single answer.

**First, a genuine gift: the LiDAR is completely solved.** The Neato laser — the "Piccolo LDS" — is one of the most reverse-engineered sensors in hobby robotics, and the documentation is exhaustive. `8N1`, `3.3 V`, `115200 baud`. A full revolution is **90 packets, 22 bytes each, four distance readings per packet — 360 readings, one per degree.** Each reading carries a distance, a signal strength, and a couple of warning flags for bad points. The turret motor is driven by the host with PWM and closed-loop speed control off the RPM the sensor reports back — which is exactly the `J2 MAIN` / `J3 MOTOR` split I found on my own board. So the single scariest part of this whole build, the part I'd mentally filed under "here be dragons," turns out to be a solved problem with reference code in three different languages. That's a load off.

**Then, a cautionary tale that's worth more than a success story.** There's a project — `94-psy/OpenNeato` — that is almost *exactly* what I'm attempting: a Neato `D7`, logic board torn out, a single-board computer dropped in running ROS 2 and Nav2. And it's marked **SUSPENDED INDEFINITELY.** The author left an honest post-mortem, and both failure modes are things I need to hear:

1. **Heat.** Their SBC hit `85 °C+` and thermally throttled, sealed inside the chassis with nowhere to put a heatsink or fan. This one stings, because the `Raspberry Pi 4` I'm planning to use runs *hotter* than the board they gave up on. Cooling just went from "afterthought" to "design constraint." I've written it into the risk list in red.
2. **Serial.** They tried to run real-time control through Neato's factory diagnostic serial port and got buffer overflows, microcontroller crashes, dropped connections. The port was built for factory tests, not for a navigation loop hammering it at high frequency.

That second one is oddly reassuring, because it's the exact mistake my architecture avoids. I'm not talking to Neato's board at all — I'm removing it and putting the real-time loop on my own `ESP32` running micro-ROS, with the Pi handling the thinking. Their dead-end is evidence my split is the right one. You learn as much from the abandoned attempt as the finished one.

**And then the pattern, which is the real story of this episode.** Every project I found stops at the same place — mine. `OpenNeato` (the cloud-replacement flavour) supports the `D3`–`D7` and explicitly excludes the D8/D9/D10: *different board, password-locked serial port.* Another project — Brainslug, now "fang of vacuula" — revives `gen2` and `gen3` Neatos by clipping an ESP onto their serial command interface, and says of the newer machines, flatly: *"gen4 robots use a completely different board, chip and firmware, and we cannot interface with these directly."* The `94-psy` attempt got as far as a `D7` and broke on heat and serial.

Three independent efforts, three different approaches, and all three draw their boundary at the exact generation sitting on my bench. The D10 is `gen4`. It's the door nobody's walked through — the i.MX 8M brain behind signed boot, a serial console nobody's cracked, a board that even the "just talk to it over serial" trick can't touch. Which quietly kills the last version of the shortcut. On an older Neato I could've kept the board and whispered commands to it. On this one there is no whispering. It's a transplant or it's nothing.

That reframes the boring part of the work entirely. The encoder wire count, the motor stall currents, the battery pinout — I went looking for those and came back empty, because **nobody has published them for this machine.** That's not a gap in my research. That's the frontier. When I clip the meter onto the roller brush next session, I'm not re-measuring something in a datasheet somewhere — I'm writing down the first public numbers for a robot the community wrote off as sealed.

So: fresh start next time, with a much clearer map. Measure the brush. Count the encoder wires. Order the drivers. And find somewhere for a Pi 4 to breathe.
