---
title: "The brain under the shield"
episode: 6
pubDate: 2026-08-04
sessionDate: 2026-08-04
status: published
teaser: "I peeled off a metal shield I'd already written off as 'just the WiFi', and found a whole Linux computer staring back. For about an hour, the entire project was in question."
heroPhoto: shielding-removed.jpg
seeAlso: [reference/brain-transplant, reference/handoff]
---

Something had been nagging me since the teardown. This whole robot — navigation, mapping, the app, the cloud it used to phone home to — supposedly ran on an `NXP LPC51U68`. That's a Cortex-M0+ microcontroller with 256 KB of flash. It's a perfectly good little chip for *twitching motors and reading sensors*, but running a mapping robot's brain on it? That never quite sat right. So I went back to the board and looked at the one thing I'd waved away the first time: a perforated metal RF shield I'd labelled "WiFi module, discard."

I pried it off. And the story fell apart in the best possible way.

Underneath were three chips that a bare-metal microcontroller has absolutely no use for. A **`Kingston`** chip — `EMMC04G`, four gigabytes of eMMC flash *mass storage*. A **`NANYA`** chip — `NT5AD256M16D9`, which is DDR3 *system RAM*. And between them, a big square BGA. Storage, plus working memory, plus a large application processor is not a microcontroller. **That is a computer that boots an operating system.** Almost certainly Linux.

The RF shield hadn't been hiding a throwaway WiFi module. It had been hiding the actual brain.

I got close-ups of the BGA and read it off: `MIMX8MN1DVTJZAA`. That's an **NXP i.MX 8M Nano** — and the little `PCA9450` power-management chip sitting next to it is the *dedicated companion PMIC* NXP built specifically to feed an i.MX 8M, which nailed the ID beyond doubt. So the real architecture was never "one tiny MCU." It's **two brains**: the i.MX 8M runs Linux and does the thinking, and the `LPC51U68` I'd found first is its real-time sidekick — the thing that actually drives motors and counts encoder ticks on the i.MX's behalf. Standard robotics design. I'd just been looking at the sidekick and calling it the whole show.

And here's where my pulse picked up. If there's a *Linux computer* in here, already wired to every motor, every sensor, and the LiDAR... why am I bolting in a Raspberry Pi at all? Why not just take over the brain that's already there? For a good hour, the whole "brain transplant" premise looked obsolete. This was going to be a very different, much more elegant project.

Then I did the reading, and the elegant version died — deservedly.

The community has a project exactly for this: **OpenNeato**, which strips the dead cloud out of Neato robots and gives them a local brain. I went looking for how people root the D10. They don't. OpenNeato supports the **D3–D7** and stops dead there, and it says why in plain language: the D8/D9/D10 have a *different board* and a **password-locked serial port**. On top of that, Neato's firmware is signed and encrypted, which on an i.MX means **HAB secure boot** — the chip's fuses hold a key, and the boot ROM simply refuses to run anything not signed with Neato's private key. There's a Linux computer in there, yes. It's just a Linux computer with the doors welded shut, and nobody's found a window.

One more nail: I decoded the rest of that part number. The `1` makes it a Nano **SoloLite** — a *single* Cortex-A53 core, no GPU. It's a modest little thing, low-end-Pi tier. Even if I *could* get in, the `Raspberry Pi 4` already sitting on my bench — four bigger cores — would run rings around it for the ROS 2 and Nav2 stack I'm aiming at.

So I'm ending exactly where I started the week: **transplant it, drop in the Pi.** But I'm ending there *honestly* now. My original writeup justified the transplant by saying "there's no OS to root" — and that was flat wrong. There's a whole Linux SoC hiding under a shield I'd dismissed. The real reason the transplant is the right move was the *other* wall all along: the boot chain is locked, signed, and password-gated, and the D10 is a generation past where anyone has broken in. Same decision. Much better map.

I've corrected the build doc and the handoff to say all that — including a note that OpenNeato's D3–D7 debug header is `RX / 3.3V / TX / GND`, in case I ever want to clip on and watch the locked boot log scroll past out of pure curiosity.

Next session, back to the boring, wonderful, *tractable* stuff: measure the roller brush, count the wheel encoder wires, and put the driver order in. The brain stays Neato's. The body becomes mine.
