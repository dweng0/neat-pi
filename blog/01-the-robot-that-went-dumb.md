---
title: "The robot that went dumb"
episode: 1
pubDate: 2026-08-04
sessionDate: 2026-08-03
status: published
teaser: "Neato went bust, Vorwerk killed the cloud, and it turns out you can't even root the thing. So I decided to gut it."
heroPhoto: topdownview.jpg
seeAlso: [reference/build-doc]
---

I have a Neato D10. Had a Neato D10. It still vacuums if I push the button, but that's about all it does now.

Here's the short version of how it got here: Neato the company went bust. Vorwerk, who owned them, switched off the cloud in Q4 2025. No app, no scheduling, no "clean the kitchen," no integration with anything. The robot boots, the robot sucks up dirt when prodded, and otherwise it's a very expensive brick with wheels.

My first instinct was the obvious one for anything with WiFi in it: **root it.** People do this. Valetudo is a whole project built on kicking the cloud client off a robot vacuum and running it locally. So I went looking for the D10's soft underbelly.

There isn't one. I hit two walls, and they're both solid.

**Wall one: there's no OS to replace.** I'd assumed there was a little Linux box in there I could get a shell on. There isn't. The mainboard runs on an NXP microcontroller — a Cortex-M0+ with 256 KB of flash. That's not a computer running an operating system I can swap the cloud client out of. That's bare-metal firmware. Valetudo-style rooting was never physically possible here, because the thing it roots — a Linux userland — doesn't exist on this robot.

**Wall two: even the firmware is locked.** The D8/D9/D10 generation ships signed firmware and a password-locked serial console. The community reverse-engineering projects (OpenNeato and friends) only ever cracked the *older* D3–D7, which shipped with an open serial shell you could just talk to. That door got welded shut a generation before mine.

So: can't root it (no OS), can't reflash it (locked boot chain). Two independent dead ends. I sat with that for a bit, slightly annoyed, and then it flipped into something better.

Because here's the thing — the *robot* is fine. The chassis is fine. The motors are fine. The LiDAR is fine. The battery is fine. The only broken part is the brain, and the brain is the one part I was never going to be allowed to fix.

So don't fix it. **Replace it.**

The plan, which I'm now committed to: gut Neato's electronics entirely. Keep everything mechanically and electrically standard — chassis, motors, LiDAR, battery, sensors — and drop in my own brain. A Raspberry Pi 4 and an ESP32 running ROS 2. Then expose the whole thing to Home Assistant as a bog-standard MQTT vacuum, so it schedules and integrates like any other smart-home device — except this one answers only to me, locally, forever.

No security to fight. No cloud to depend on. No company that can go bust and switch my hardware off. Just a documented robotics build.

I'm calling it the brain transplant.

There's one more thing I found when I started opening it up, and it's the reason I think this is actually going to work rather than being a heroic waste of a weekend. But that's the next episode — I need to get the lid off first.
