# Neato D10 → ROS 2 Robot (Home Assistant Controlled)

**A "brain transplant" project: gutting a bricked Neato D10 and rebuilding it as an open, locally-controlled robot.**

> Living document — last updated 2026-08-04. Findings below are confirmed from a hands-on teardown unless marked **TBD** (needs measurement) or `?` (unconfirmed).

---

## TL;DR

Neato went bust and Vorwerk killed the cloud in Q4 2025. The D10 still vacuums on a button press but is otherwise a dumb brick — no app, no scheduling, no integration.

**⚠️ Premise correction (2026-08-04):** this doc originally claimed the D10 is "bare-metal, no OS, nothing to root." **A later teardown step proved that wrong.** Lifting the RF shield revealed a full **NXP i.MX 8M Nano SoloLite** (`MIMX8MN1DVTJZAA`) **Linux computer** — the BGA SoC (1× Cortex-A53 + 1× Cortex-M7, no GPU/NPU) plus `NANYA` DDR3, a `Kingston` 4 GB eMMC, and the matched `NXP PCA9450` PMIC — sitting alongside the LPC51U68. The robot is a **two-brain design**: the **i.MX 8M runs Linux (the real brain)**; the **LPC51U68 is the real-time body controller** (motors/encoders/sensors). So there *is* an OS here.

**The other wall still stands — and it's the decisive one.** Research (2026-08-04) confirms the D8/D9/D10 boot chain is locked, with no public root path:
- **OpenNeato supports D3–D7 only and explicitly excludes the D8/D9/D10** — citing a *"different board"* and a *"password-locked serial port."* That's the very project that would have rooted these if it were feasible.
- **Neato firmware is encrypted/signed**, decrypted at boot from secure key storage → on this i.MX that's **HABv4 secure boot** (SoC fuses hold the key hash; unsigned images are rejected).

**Decision — reuse ruled out, transplant confirmed.** Even though there's a real Linux brain in there, you'd face a password-locked console *and* a signed-boot wall with no known bypass — and the SoC is only a single-A53 SoloLite anyway (the **Raspberry Pi 4** already on hand, quad A72, is far stronger for the intended ROS 2 + Nav2 + slam_toolbox stack). So: **discard Neato's electronics, keep the mechanically good and electrically standard parts (chassis, motors, LiDAR, battery, sensors), and drop in a Pi 4 + ESP32 running ROS 2**, exposed to Home Assistant as an MQTT vacuum.

The transplant was always the right call — originally justified by "no OS to root" (wrong: there *is* a Linux SoC) but ultimately correct because of the **locked boot chain** (right). End goal unchanged: an open, locally-controlled ROS 2 robot driven from Home Assistant.

**Sources:** [OpenNeato (renjfk) — D3–D7 scope, D8–D10 excluded](https://github.com/renjfk/OpenNeato) · [OpenNeato user guide — debug port pinout `RX/3.3V/TX/GND`](https://github.com/renjfk/OpenNeato/blob/main/docs/user-guide.md) · [NXP i.MX 8M HABv4 secure boot guide](https://github.com/nxp-imx/uboot-imx/blob/lf_v2025.04/doc/imx/habv4/guides/mx8m_secure_boot.txt)

The single best discovery from the teardown: **the LiDAR is a Neato LDS**, the most reverse-engineered laser scanner in hobby robotics. That de-risks the hardest part of the whole project.

---

## Component Inventory

Everything found inside, what happens to it, and why. Signal direction is relative to the component (**IN** = power/commands arriving; **OUT** = data leaving). Connector labels are the silkscreen names where known.

### Salvaged (keep)

| Component | Verdict | Power | Conn IN | Conn OUT | Role in new build / notes |
|---|---|---|---|---|---|
| Chassis, wheels, caster | Keep | — | — | — | Physical platform. Host the Pi + ESP32 here once the old board is out. |
| Drive wheel motors ×2 (brushed DC + encoder) | Keep | `260-0016`, 14.4 V nominal; stall ~2.1 A (L) / ~2.4 A (R), measured 2026-08-04 | Motor power from driver (JST to old board) | Encoder pulses (channel count **TBD**) | Locomotion + odometry. Wire to an H-bridge; read encoders on the ESP32. Motor terminals = the two chunky solder posts flanking the encoder disc. |
| **Wheel encoders ×2** (on the wheel motors) | Keep | Logic supply **TBD** (3.3 or 5 V) | Encoder Vcc/GND | A (+ B?) pulses | `LEGO WHEEL ENCODER ASY: 915-1055 REV`, board marked `STD-3`. Disc is **solid, not slotted** → likely **magnetic (ring magnet + Hall)**, not optical. Dirt-tolerant, which suits a vacuum. Sensing only — carries no motor power. |
| Roller brush motor (brushed DC) | Keep (optional) | `905-0460-RoHS 14.4VDC` (batch `215I31`); current **TBD** | Motor power (JST) | — | Needed only if the robot should actually vacuum. Drive via H-bridge or MOSFET. **Only remaining motor whose current is unknown.** |
| Blower / vacuum motor | Keep (optional) | EVERFLOW `F121225BU (AFX19bR)` — `DC14.4V 2.0AMP`, dated 2021-08-13 | 14.4 V + PWM in | Tach out (RPM) | **Brushless blower with integrated driver — needs NO H-bridge.** Feed it 14.4 V, PWM it straight from an ESP32 GPIO. See Blower section. |
| **LiDAR — Neato LDS 2.2** (`290-1044 REV 4`) | **Keep (star part)** | Logic 5 V (~45 mA idle / ~135 mA spinning); TX/RX at **3.3 V**; spin motor separate | `J2 MAIN` (5 V + 3.3 V UART), `J3 MOTOR` (host-driven PWM) | `J2 MAIN`: 3.3 V UART **8N1 @ 115200** — distance packets | The reason this project is feasible. Read on Pi/ESP32; drive spin motor in a closed loop off the reported RPM. See LiDAR section below. |
| Battery — Li-ion 14.4 V `4S2P` | Keep | 12 V (empty) → 16.8 V (full); 6200 mAh / 89 Wh | Dock charge (was via old board) | 6-pin JST: power + thermistor/sense | Powers everything through a buck converter. **Map the 6 pins before use** — only 2 are main +/−; others are temp/sense. |
| Bumper / cliff / wall / drop sensors | Keep (as present) | 3.3–5 V logic, **TBD** | Sensor power (JST) | Digital/analog lines to old board | Reuse for obstacle + cliff safety on the ESP32. Identify each individually when wiring. |

### Discarded

| Component | Verdict | Power | Conn IN | Conn OUT | Why discarded |
|---|---|---|---|---|---|
| Mainboard (`520-0394 Rev.B`) — **NXP i.MX 8M Linux SoC + LPC51U68 MCU** | **Discard** (reuse ruled out — locked boot chain) | 14.4 V in; regulates rest | All harness JSTs + USB-C | — | **Correction: not bare-metal** — carries a full i.MX 8M Nano SoloLite Linux computer (DDR3 + 4 GB eMMC + PCA9450 PMIC) beside the LPC51U68 body controller. Reuse investigated and rejected: D8–D10 boot chain is locked (password-locked console + signed/HAB secure boot; OpenNeato excludes these models). Replaced by Pi 4 + ESP32. |
| WiFi/BT module (under perforated RF shield) | Discard | 3.3 V (from board) | On-board + U.FL antenna lead | — | Only ever talked to Neato's dead cloud. The Pi has its own WiFi. |
| Button / UI board (`neato`, TP1–TP22) | Discard | 5 V | 6-pin JST to mainboard | Button presses / LED status | Sits under the power + reset buttons. Pi/ESP32 handle any controls you want. TP1–TP22 are just factory test pads. |
| USB-C "SW Update" service port | Discard (with mainboard) | — | — | — | Vendor firmware-update channel; useless without Neato's signed images. |

---

## Target Architecture

Three layers, clean separation of concerns.

### 1. Home Assistant — scheduling + notifications
- Robot appears as HA's built-in **MQTT Vacuum** entity: start / stop / return-to-base commands, plus status (docked / cleaning / error / battery).
- All scheduling is normal HA automation ("weekdays 09:00 → start"). No scheduling logic on the robot.
- HA subscribes to the robot's state topic to react ("if error → notify").
- **MQTT, not Zigbee.** Zigbee is for low-power battery sensors and is the wrong tool for a WiFi robot. Publishing/subscribing topics *is* MQTT — the Pi speaks it natively over WiFi.

### 2. Raspberry Pi 4 — the brain
- **ROS 2** with **Nav2** (path planning + coverage) and **slam_toolbox** (mapping + localization from LiDAR + odometry).
- One small **bridge node** (~100 lines Python): subscribes to the MQTT command topic → issues ROS cleaning goals; publishes status (battery / stuck / done / lost) back to MQTT.

### 3. ESP32 — real-time motor + sensor co-processor
- Linux isn't real-time, so timing-critical work lives here: PWM to motor drivers, counting encoder pulses, polling bumper/cliff/wall sensors, reading battery voltage.
- Connects to the Pi over USB serial. Runs **micro-ROS** so it joins the ROS 2 graph directly — the Pi sees sensors/encoders as native ROS topics.

### Status reporting (derived once sensor data flows)

| Status | How it's derived |
|---|---|
| Stuck | Motors commanded, but encoders show no movement |
| Low battery | Battery voltage below threshold → return-to-base / notify |
| Finished | Nav2 coverage complete → publish docked/done |
| Lost | slam_toolbox localization confidence drops (least clean signal — localization loss is a known headache) |

---

## The LiDAR (Neato LDS 2.2)

The make-or-break component, and it landed on the easy side.

- **What it is:** a Neato Laser Distance Sensor ("Piccolo"/LDS family) — the same lineage as the famous XV-11 scanner, reverse-engineered by the community since ~2010.
- **Interface:** one data line, 8N1 UART, **3.3 V logic, 115200 baud** — consistent across every LDS version. Feeds straight into a Pi/ESP32 UART.
- **Two connectors:** `J2 MAIN` (5 V power + serial data) and `J3 MOTOR` (spin motor). The **host drives the motor itself** via PWM, closing the loop on the RPM the LDS reports in its own packets to hold a steady spin.
- **Existing code to adapt** (not write from scratch): C++ Raspberry Pi classes that read coordinates *and* run the motor loop, Python packet decoders, and ROS drivers (`xv_11_laser_driver`, `neato` packages). Packet format is documented (header, index, speed, distance readings, checksum).
- **Chip side:** an `LM393` comparator (U1) — the classic LDS laser-detection circuit, confirming the standard design.
- **Only open item:** this is the newer **LDS 2.2 (2019)**; most reference code targets the ~2010 XV-11. Same architecture, so expect minor adaptation at most. **First job when the logic analyzer arrives: clip onto `J2`'s data line, capture, and confirm baud + packet layout against the documented format.**

---

## The Blower (EVERFLOW F121225BU) — no driver required

Label reads `MODEL: F121225BU (AFX19bR)` / `DC14.4V 2.0AMP` / `2021 08 13`. Two things fall out of that:

- **Its current is printed on it: 2.0 A.** No measurement needed. This was expected to be the hardest number to capture and it turned out to be free.
- **It is almost certainly a 4-wire brushless PWM blower, not a brushed DC motor.** In Everflow's naming the `…BU` suffix marks their 4-wire PWM family (their catalogue F126025BU and F129025BU are both PWM parts), while the otherwise-similar `…BL` variant is 3-wire tach-only. Four wires are visible on the harness: black, red, yellow, blue.

**Consequence: the planned BTS7960 for the blower is deleted.** A brushless fan carries its own commutation electronics, so:

- Wiring is `black = GND`, `red = +14.4 V`, `yellow = tach out`, `blue = PWM in` (standard 4-wire fan convention — verify before trusting).
- Speed control is a PWM signal direct from an ESP32 GPIO. ~25 kHz is the usual fan PWM frequency.
- The tach line is a **free RPM feedback signal** — useful for detecting a clogged or jammed blower.
- **Do not put an H-bridge on it.** Reversing a brushless fan just confuses its internal controller.
- **Do not measure its winding resistance.** You'd be probing driver electronics, not a winding; the reading is meaningless.

**Open item:** confirm the wire count is 4 (not 2). Four confirms all of the above; two would mean it's a plain brushed motor after all and a MOSFET/BTS7960 comes back on the list.

---

## Motor Current Status

| Motor | Rating source | Current | Driver decision |
|---|---|---|---|
| Blower/vacuum | **Printed on label** | **2.0 A** | ✅ None needed — PWM direct from ESP32 |
| Roller brush | Not printed anywhere | **TBD** | Blocked on measurement |
| Drive wheel ×2 | Winding resistance (2026-08-04) | **~2.1 A (L) / ~2.4 A (R)** — matched pair, size to ~2.5 A | **`DRV8871` ×2** — TB6612 out (stall > its ~1.2 A continuous) |

Method changed: rather than trying to catch a millisecond inrush spike on a 2000-count meter, **derive stall current from winding resistance** — `stall ≈ 14.4 V ÷ R`. No power applied, no risk. See `neato-d10-measuring-motor-current.md`.

---

## Bill of Materials

### Already on hand
Raspberry Pi 4 · soldering iron + solder · multimeter · breadboard · Arduino (e.g. Elegoo starter kit — bench part-testing only, *not* the final co-processor).

### Ordered
| Item | Purpose | Spec / search term | Qty |
|---|---|---|---|
| ESP32 WROOM-32 (USB-C) | Real-time motor/sensor co-processor (micro-ROS) | Dual-core, 4 MB flash, CH340C | 2 |
| 8-ch logic analyzer | Decode LDS 2.2 packets + encoder signals | 24 MHz clone + PulseView (sigrok) | 1 |
| Heat-shrink assortment | Wiring insulation | 2:1, assorted diameters | 1 |
| Dupont jumper wires | Bench prototyping | M-M / M-F / F-F | 1 set |

### Hold until measured (post-teardown specifics)
| Item | Purpose | Spec / search term | Blocked on |
|---|---|---|---|
| Buck converter | 14.4 V battery → 5 V for the Pi | **Input must start below 12 V** (e.g. 6–24 V in), **5 V @ ≥5 A** out. `LM2596 buck converter 5V 5A` or `DC-DC step down 6-24V to 5V 5A`. Avoid car-type fixed-12 V modules (Pi browns out near empty). | Spec locked ✅ — just buy when ready |
| Drive-motor driver | Drive the 2 wheel motors | **`DRV8871` ×2** — wheels measured ~2.1 A / ~2.4 A stall (2026-08-04), matched pair. In the 1–3 A band where TB6612/DRV8833 are marginal; DRV8871 (3.6 A) has headroom. | ✅ Resolved |
| Brush driver | Drive the roller brush | Single-direction: MOSFET module, or `BTS7960` for headroom | Measure brush winding resistance |
| ~~Blower driver~~ | ~~Drive the vacuum motor~~ | **REMOVED** — blower is a brushless PWM fan with its own driver. Optionally a small high-side MOSFET switch for hard on/off. | ✅ Resolved |
| JST connector kit | Tap the harness without cutting | Pitch **TBD** — measure the plugs first (JST family has several pitches) | Measure connector pitch |
| Logic level shifter | 5 V ↔ 3.3 V, only if a kept sensor is 5 V logic | Bidirectional module | Confirm any 5 V sensor logic |
| T10 Torx driver (long reach) | Remaining recessed case screws | Long, slim shaft — `T10 Torx screwdriver long reach precision` | Optional but likely useful |
| Standoffs / mounts | Mount Pi + ESP32 in the chassis | Sizes **TBD** | Measure free space |

### Software (all free)
ROS 2 (Humble+) · Nav2 · slam_toolbox · micro-ROS · Mosquitto (or HA's broker) · Home Assistant + MQTT integration.

---

## Build Order

Sequenced so there are working milestones early and the hard nav work last.

1. **ESP32 + one motor driver → drive a single wheel from a serial command.** Proves the toolchain; instant feedback.
2. **Add encoders** → confirm distance-travelled readings (odometry foundation).
3. **LiDAR bring-up** → power the LDS, drive its spin motor, capture `J2` with the logic analyzer, confirm/adapt the packet decoder. (De-risked, but still the highest-value integration.)
4. **Bare-bones MQTT bridge** → HA sends `start`, robot just drives forward. Proves the whole HA → Pi → ESP32 → motor pipeline end-to-end *before* SLAM exists.
5. **slam_toolbox + Nav2** → mapping, localization, coverage. The 80%. Expect real time in Nav2 config tuning — documented, big community, but fiddly.

---

## Open Risks

| Item | Status |
|---|---|
| LiDAR reverse-engineering | **Resolved** — confirmed Neato LDS 2.2, standard interface, existing drivers. Only confirm LDS 2.2 packet format vs XV-11 with the analyzer. |
| Nav2 tuning | The real time-sink. Powerful but fiddly; everyone hits this wall. |
| "Lost" detection | Localization-loss is the least clean status signal to detect reliably. |
| Blower current + driver | **Resolved** — 2.0 A on the label; brushless PWM fan needs no driver at all. Confirm 4-wire count. |
| Motor current draw | **Wheels resolved** — ~2.1 A / ~2.4 A stall (2026-08-04) → DRV8871 ×2. **Brush still TBD** — measure winding resistance next. |
| Encoder channel count | **TBD** — 6 harness wires = quadrature (direction-aware); 5 = single channel, direction-blind. Matters for odometry quality and for slam_toolbox. |
| Encoder supply voltage | **TBD** — 3.3 V or 5 V; determines whether a level shifter is needed. |
| Sensor logic levels | **TBD** — map each kept sensor when wiring. |
| Battery 6-pin pinout | **TBD** — identify power vs thermistor/sense before connecting. |
| Internal mounting space | **TBD** — measure once the old board is out. |

---

## References
- Neato XV-11 LDS protocol + Linux implementation — https://github.com/ssloy/neato-xv11-lidar
- XV-11 LDS on a Raspberry Pi (C++, incl. motor control) — https://github.com/berndporr/neato-xv11-lidar
- XV-11 LIDAR tutorial (ev3dev) — https://www.ev3dev.org/docs/tutorials/using-xv11-lidar/
- OpenNeato (D3–D7 context) — https://github.com/renjfk/OpenNeato
- ROS 2 Nav2 — https://docs.nav2.org
- slam_toolbox — https://github.com/SteveMacenski/slam_toolbox
- micro-ROS — https://micro.ros.org
- HA MQTT Vacuum — https://www.home-assistant.io/integrations/vacuum.mqtt/
