# Neato D10 — Brain Transplant

Turning a bricked robot vacuum into a robot I actually own.

## What this is

I have a Neato D10. The company went bust, Vorwerk switched off the cloud in
Q4 2025, and the robot lost its app, its scheduling, and every bit of smarts it
ever had. It still sucks up dirt if you push the button — otherwise it's a very
expensive brick with wheels.

You can't rescue it the usual way. There's no Linux box inside to root (the
mainboard is a bare-metal NXP Cortex-M0+), and the firmware is signed with a
password-locked serial console. Two solid dead ends.

So this project doesn't try to fix the brain. It **replaces** it.

The plan: gut Neato's electronics entirely, keep everything mechanical and
electrical standard — chassis, motors, LiDAR, battery, sensors — and drop in my
own brain:

- **Raspberry Pi 4 + ESP32** running **ROS 2**
- Exposed to **Home Assistant** as a bog-standard **MQTT vacuum**, so it
  schedules and integrates like any other smart-home device — except this one
  answers only to me, locally, forever.

No cloud to depend on. No company that can go bust and switch my hardware off.
Just a documented robotics build.

## 📖 It's journalled as a blog

This whole build is written up, episode by episode, as a first-person devlog
serial — dead ends included, because those are the good part. Each real bench
session becomes the next episode.

There are two parallel serials:

- **The main build** (`blog/01-…` onward) — the brain transplant itself: the
  teardown, the salvage verdicts, motor characterisation, the LiDAR decode, the
  encoder hunt, wiring up the ESP32.
- **Learning Rust on the Robot** (`blog/rust-…`) — porting the known-good C++
  ESP32 firmware to bare-metal `no_std` Rust on the real hardware, as an excuse
  to go deep on ownership and the borrow checker.

The episodes live in [`blog/`](blog/) as plain Markdown. The
[`site/`](site/) directory is an [Astro](https://astro.build) project that reads
those files (plus the reference docs) in place and publishes them as a
subscribable serial with an RSS feed. The site is a *reader* of the build — it
never writes to the working docs.

## Repo layout

```
Neato/
  blog/                       # the devlog episodes (both serials)
  site/                       # Astro site that publishes the blog + reference docs
  esp32-firmware/             # the known-good C++ ESP32 firmware (PlatformIO)
  esp32-firmware-rs/          # the bare-metal no_std Rust port (in progress)
  pwm-serial-protocol/        # shared serial protocol crate (Pi ↔ ESP32)
  hardware-scripts/           # bench test scripts (motor current, encoders, …)
  photos/                     # bench photos, read in place by the site

  neato-d10-brain-transplant.md      # the living build doc — "what's true now"
  neato-d10-handoff.md               # rolling handoff — the baton between sessions
  neato-d10-measuring-motor-current.md
  PLAN.md                            # how the site + serial workflow was designed
```

## Reference docs vs. episodes

Two content types, different jobs:

| Type          | Job                                   | Voice              |
|---------------|---------------------------------------|--------------------|
| **Reference** | Stay authoritative & current — the wiki | Factual, as-is today |
| **Episodes**  | Tell the story, chronologically — the serial | First-person devlog |

Reference answers *"what's true now."* Episodes answer *"what happened, and what
I learned."* An episode is a snapshot in time and can go stale; the reference
docs never should.

## The workflow

The build is driven by a two-trigger loop (project-level Claude Code skills that
travel with the repo):

- **"lets cook"** → reads the rolling handoff, briefs me on where things stand
  and the next actions, then gets out of the way so I can work.
- **"finish up"** → serialises the session into the next blog episode **and**
  rewrites the handoff for next time.

The handoff is the private baton I pass to myself (current state); the episode is
the public story (a snapshot). Same session, two outputs.

## Running the site locally

```bash
cd site
npm install
npm run dev      # local preview
npm run build    # static output in dist/
```
