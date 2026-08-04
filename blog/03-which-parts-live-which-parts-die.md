---
title: "Which parts live, which parts die"
episode: 3
pubDate: 2026-08-04
sessionDate: 2026-08-03
status: published
teaser: "Sorting the salvage from the scrap, and sketching the three-layer nervous system that replaces the brain I just condemned."
heroPhoto: topdownview.jpg
seeAlso: [reference/build-doc, reference/handoff]
---

With everything out on the bench and labelled, the job now is triage: what earns a place in the new robot, and what goes in the bin. The rule I settled on is simple — **keep anything mechanically good and electrically standard, bin anything that only existed to talk to a dead cloud.**

**Dies:** the mainboard (that NXP microcontroller and its locked firmware), the WiFi/BT module under the RF shield, and the little `neato` button board. All three are Neato-proprietary, all three are useless without the cloud, and none of them are anything I want to reverse-engineer when I can just replace them.

**Lives:** the chassis, both drive wheel motors and their encoders, the roller brush motor, the blower, the LiDAR, the battery, and the bump/cliff/wall sensors. These are all boring, standard, well-understood hardware. Brushed DC motors. A Li-ion pack. A Neato LDS. Nothing here is a mystery I have to solve — it's all stuff with datasheets or decade-old community documentation.

That split is the whole reason this is doable. I'm not rebuilding a robot. I'm giving an existing, perfectly good robot body a new nervous system.

So what's the nervous system?

I settled on three layers, each doing the thing it's actually good at:

**Home Assistant — the scheduler and the face.** This is where "vacuum the house at 9am on weekdays" lives, and where notifications come from. The robot shows up in HA as a standard **MQTT vacuum** entity. I went back and forth on Zigbee here and landed firmly on MQTT — it's the natural fit for a custom device talking to HA, and I'm not adding a radio protocol I don't need.

**Raspberry Pi 4 — the brain.** This is the part that does the genuinely hard robotics. It runs ROS 2, with Nav2 for path planning and coverage (actually cleaning the *whole* floor, efficiently), and slam_toolbox for mapping and localisation (building the map and knowing where it is on it). Plus a small bridge node — maybe a hundred lines — that translates between MQTT (what Home Assistant speaks) and ROS 2 (what the robot speaks).

**ESP32 — the spinal cord.** The Pi is smart but it's not real-time; it shouldn't be the thing counting encoder ticks or holding a motor PWM steady. That's the ESP32's job. Running micro-ROS, it joins the ROS 2 graph directly over USB serial and handles the twitchy real-time work: motor PWM, encoder counting, polling the sensors, reading battery voltage. It's the reflexes; the Pi is the thoughts.

The data path end to end: Home Assistant says "clean" → the Pi's bridge turns that into a ROS 2 goal → Nav2 and slam_toolbox figure out *where to go* → the ESP32 turns that into actual volts through actual motors → encoders and LiDAR feed reality back up the chain. Every layer replaceable, every layer local, nothing phoning home.

That's the architecture, decided. It reads clean on paper. The catch — and there's always a catch — is that "actual volts through actual motors" hides a question I couldn't answer from any label on any part: **how much current do these motors actually pull?** Get that wrong and I either buy motor drivers that melt, or I over-buy and waste money and space.

Answering that turned into its own little detective story, complete with a meter that isn't up to the job and one motor that smugly refused to need measuring at all. Next episode.
