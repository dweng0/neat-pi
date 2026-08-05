# Neato D10 Brain-Transplant — Handoff / Continuation Doc

**Purpose:** snapshot of where this project stands so it can be resumed in a fresh conversation without re-deriving anything. Pairs with the main build doc: `neato-d10-brain-transplant.md`.

**Last updated:** 2026-08-04

---

## One-paragraph context

The user has a Neato D10 robot vacuum. Neato went bust; Vorwerk killed the cloud (Q4 2025). **Working plan: "brain transplant"** — rebuild as a ROS 2 robot on a Raspberry Pi 4 + ESP32, controlled from Home Assistant as an MQTT vacuum, keeping the mechanically/electrically standard parts. The user wants **full vacuum function** (keep brush + blower motors), and intends to **blog the findings on GitHub** eventually.

**Premise correction (2026-08-04), now resolved:** the transplant was originally justified by "bare-metal LPC51U68, no OS, can't be rooted." The "no OS" half was **wrong** — lifting the RF shield revealed a full **NXP i.MX 8M Nano SoloLite Linux computer** (SoC + DDR3 + 4 GB eMMC + PCA9450 PMIC) beside the LPC51U68, so the robot is a **two-brain design** (i.MX = Linux brain, LPC51U68 = real-time body controller). But the *conclusion* still holds: research confirms the **D8/D9/D10 boot chain is locked** (OpenNeato excludes them — "password-locked serial port"; encrypted/signed firmware; i.MX HAB secure boot), so **reusing the existing brain is not viable. Transplant confirmed — full steam ahead on the Pi 4 + ESP32 path.**

The user is hands-on: has soldering gear, a multimeter, a Pi 4, an Elegoo Arduino starter kit, and is comfortable opening hardware. Teardown is essentially complete.

---

## Key confirmed facts (from hands-on teardown)

| Thing | Finding |
|---|---|
| **Main application processor** | **NXP i.MX 8M Nano SoloLite** (`MIMX8MN1DVTJZAA`, industrial grade) under the RF shield — **1× Cortex-A53 (Linux) + 1× Cortex-M7 (real-time), no GPU/VPU/NPU.** Big BGA + `NANYA NT5AD256M16D9-HR` DDR3L + `Kingston EMMC04G-W627` (4 GB eMMC) + `NXP PCA9450B` PMIC. **This is a Linux computer** — boots an OS from eMMC. Found 2026-08-04. *(First digit `1`=SoloLite hard to read; worth a re-check — `1` vs `6`/Quad is 4× the cores.)* Photos: `board-imx8m-soc.jpg`, `board-nanya-dram-and-lpc51u68.jpg`, `board-kingston-emmc.jpg`, `board-soc-cluster-overview.jpg`. |
| Real-time MCU | **NXP LPC51U68** (Cortex-M0+, 256 KB flash) — **the body controller, NOT the whole brain.** Handles motors/encoders/sensors in real time; takes commands from the i.MX 8M. |
| Mainboard P/N | `520-0394 Rev.B`, has USB-C "SW Update" service port (candidate UART/console + i.MX serial-download port — investigate). |
| ~~WiFi/BT "module"~~ | The perforated RF shield did **not** cover a throwaway WiFi module — it covered the i.MX 8M Linux subsystem above. WiFi/BT combo chip is a small QFN in that cluster. **Do not discard this board section.** |
| Button board | Small `neato` PCB (TP1–TP22) under power+reset buttons → **discard** |
| **LiDAR** | **Neato LDS 2.2** (`290-1044 REV 4`, © 2019). Standard Neato LDS. **8N1 UART, 3.3 V, 115200 baud.** Connectors: `J2 MAIN` (5 V power + serial data out), `J3 MOTOR` (spin motor, host-driven PWM closed-loop off reported RPM). LM393 comparator on chip side = classic design. Existing driver code exists (xv_11 / ssloy / berndporr repos). |
| Battery | Li-ion **14.4 V nominal** `4S2P`, 6200 mAh / 89 Wh. Real range **12 V (empty) → 16.8 V (full)**. 6-pin JST = power + thermistor/sense (pinout not yet mapped). |
| **Drive wheel motors** | `260-0016`, 14.4 V, batch `21 41`. Brushed DC. **Stall MEASURED 2026-08-04: L ≈ 2.1 A (R 6.7 Ω), R ≈ 2.4 A (R 5.9 Ω)** via winding resistance. Matched pair → size to ~2.5 A. |
| **Wheel encoders** | `LEGO WHEEL ENCODER ASY: 915-1055 REV`, board marked `STD-3`. Disc is **solid, not slotted** → likely **magnetic (Hall)**, not optical — good, dirt-tolerant. **Sensing only, carries no motor power.** Motor terminals are the two chunky solder posts flanking the disc. Wire count / channel count still TBD. |
| Roller brush motor | `905-0460-RoHS 14.4VDC`, batch `215I31`. Brushed DC. No current rating → **still must be measured** (the one remaining unknown current). |
| **Blower/vacuum** | EVERFLOW `F121225BU (AFX19bR)` — **`DC14.4V 2.0AMP`**, dated 2021-08-13. **Current is printed on the label — no measurement needed.** `…BU` suffix = Everflow's 4-wire PWM family → **brushless with integrated driver; needs NO H-bridge.** 4 wires seen (black/red/yellow/blue). |
| Multimeter | **MS8233A**, 2000 counts, 600 V CAT III. **10 A jack IS fused** (panel reads `MAX 10A FUSED`, max 30 s every 15 min) — corrected 2026-08-04 from an earlier "unfused" note. Ω ranges 200 → 2 M; 200 Ω range gives 0.1 Ω resolution (marginal for motor windings — round up). |

**Important gotcha:** the "19.5 V 1.5 A" on the device plate is the **dock charge input**, NOT the battery. Tap the 14.4 V battery for the buck converter.

**Winding-resistance measurement — what actually works (learned 2026-08-04):** hand-held probes give garbage — contact resistance bounces the reading 20→100 Ω and fakes a high number. Use **alligator clips** on the two chunky posts, then **rotate the shaft, STOP, let it settle, and only then read.** Take the lowest *stable, repeatable* value. Ignore anything that flashes by mid-twist (those are the clips shifting, not the commutator). Leads measured `0.1 Ω`, subtract from every reading.

---

## Architecture (decided)

- **Home Assistant** — scheduling + notifications. Robot exposed as HA **MQTT Vacuum** entity. (MQTT, not Zigbee — that was discussed and settled.)
- **Raspberry Pi 4** — the brain. ROS 2 + Nav2 (coverage/path planning) + slam_toolbox (mapping/localization) + a ~100-line MQTT↔ROS bridge node.
- **ESP32 (micro-ROS)** — real-time co-processor: motor PWM, encoder counting, sensor polling, battery voltage. Joins the ROS 2 graph directly over USB serial.

---

## Motor driver plan (wheels resolved; brush pending one measurement)

Full vacuum = **4 motors**, but only **3 need drivers** — the blower turned out to have its own.

| Motors | Driver | Why |
|---|---|---|
| 2× drive wheels | **2× `DRV8871`** (one single H-bridge per wheel) ✅ **RESOLVED** | Measured stall ~2.1 A / ~2.4 A. In the 1–3 A band where TB6612FNG (~1.2 A/ch continuous) is marginal; DRV8871 (3.6 A) has headroom. One motor per board. |
| Roller brush | **1× BTS7960** or a MOSFET module — *pending measurement* | One-direction, moderate current. Measure winding resistance to choose. |
| ~~Blower/vacuum~~ | **NONE — resolved** | Brushless PWM fan with integrated driver. 14.4 V + PWM straight from an ESP32 GPIO (~25 kHz). Free tach line back for RPM/clog detection. |

**Blower wiring (standard 4-wire fan convention, verify before trusting):** black = GND, red = +14.4 V, yellow = tach out, blue = PWM in. **Do not** put an H-bridge on it (reversing confuses the internal controller) and **do not** measure its winding resistance (you'd be probing driver electronics, not a winding).

**Key principle: size drivers to STALL current, not running current.** Wheels done; brush still pending. Do NOT finalise the driver order until the brush is measured — buying now is guessing.

**Why the ESP32 needs drivers at all:** ESP32 GPIO outputs ~mA at 3.3 V — enough to *signal*, not to *spin a motor*. The motor driver (H-bridge/MOSFET) is the muscle that switches battery current under ESP32 control. There is no separate "driver" needed for the ESP32 itself to run.

---

## Elegoo kit — what's useful here

The user's Elegoo starter kit contains (relevant items only):

- **L293D** (dual H-bridge, ~600 mA/ch) → **bench-test rig for wheel motors only.** Too weak for final use, esp. blower. Use it just to spin a wheel motor gently on the bench.
- **ULN2003 stepper module** → not usable for these DC motors; ignore.
- **PN2222 NPN transistor (×2) + diode rectifier (×2)** → exactly the classic LiDAR spin-motor drive circuit (transistor + flyback diode, PWM from ESP32). Already have what's needed to bring up the LDS motor.
- **Thermistor** → handy reference when mapping the battery's 6-pin connector.
- UNO R3, breadboard, jumpers, sensors, etc. → general bench use.

---

## Parts status

**Already have:** Pi 4, soldering iron+solder, multimeter, breadboard, Arduino/Elegoo kit.

**Ordered (arriving ~next week):**
- 2× ESP32 WROOM-32 (USB-C, CH340C, dual-core, 4 MB flash) — real-time co-processor
- 8-ch 24 MHz logic analyzer (PulseView/sigrok) — decode LDS 2.2 packets + encoders
- Heat-shrink assortment (2:1)
- Dupont jumper assortment

**Ready to buy (wheels resolved):**
- **2× DRV8871 motor driver module** (single H-bridge each; specs to confirm on listing: 3.6 A peak, 6.5–45 V in). For the drive wheels. ✅ Spec locked. Ideally order together with the brush driver once the brush is measured.

**On hold until measured / confirmed:**
- Buck converter — **spec LOCKED:** input starts below 12 V (e.g. 6–24 V in), **5 V @ ≥5 A** out; avoid car-type fixed-12 V modules. Search `LM2596 buck converter 5V 5A`. Can buy anytime.
- Brush driver (1× BTS7960 or MOSFET module) — **blocked on brush winding-resistance measurement.**
- JST connector kit — blocked on measuring harness plug pitch.
- Logic level shifter — only if a kept sensor is 5 V logic.
- T10 Torx long-reach driver — for remaining recessed case screws (user confirmed screws are T10). Optional but likely useful.
- Standoffs/mounts — blocked on measuring free internal space.

---

## NEXT ACTIONS (resume here)

1. **[Immediate, next session] Measure the roller brush winding resistance** — the last unknown current, and the final gate on the driver order.
   - `905-0460-RoHS`, slightly different motor from the wheels. User plans to photograph it first, then measure.
   - Method (proven on the wheels): Ω 200 range, subtract the `0.1 Ω` lead resistance, **alligator clips across the two motor terminals**, rotate → stop → settle → read, take the lowest *stable* value. `stall ≈ 14.4 V ÷ R`.
   - Result picks the brush driver: **≤ ~1.5–2 A → MOSFET module; higher → BTS7960.**
2. **[Immediate] Count the wheel harness wires** — 6 = quadrature encoder (direction-aware, good); 5 = single channel (direction-blind, matters for odometry and slam_toolbox). Then use continuity mode to identify which two wires go to the motor posts and label them. (Chassis was reassembled at end of last session — will need reopening. **Gotcha: the wheel axle screw is reverse-threaded — lefty-tighty, righty-loosey.** If it feels like it's tightening as you loosen, you're going the wrong way.)
3. **[Immediate] Confirm the blower has 4 wires, not 2** — 4 confirms the no-driver conclusion; 2 would put a MOSFET/BTS7960 back on the shopping list.
4. **[Once brush measured] Place the driver order in one parcel** — 2× DRV8871 (wheels, confirmed) + brush driver (MOSFET or BTS7960, per measurement).
5. **[Next week, when parts arrive]**
   - Build order step 1: ESP32 + L293D (or final driver) → drive one wheel from a serial command.
   - LiDAR bring-up: power LDS, drive spin motor (PN2222 + diode circuit), clip logic analyzer on `J2` data line, **confirm LDS 2.2 packet format vs documented XV-11 format** (the one remaining LiDAR unknown).
6. **[Anytime] Buy the buck converter** — spec is locked.
7. **[Before wiring] Map the battery 6-pin connector** — identify power vs thermistor/sense pins (multimeter; thermistor from Elegoo kit as reference).

---

## Build order (for reference)

1. ESP32 + one motor driver → drive a single wheel from serial. (Toolchain proof.)
2. Add encoders → odometry readings.
3. LiDAR bring-up → power, spin, capture `J2`, confirm/adapt packet decoder.
4. Bare-bones MQTT bridge → HA `start` drives robot forward. (Proves full HA→Pi→ESP32→motor pipeline before SLAM.)
5. slam_toolbox + Nav2 → mapping, localization, coverage. (The 80%; Nav2 tuning is the time-sink.)

---

## Open questions / risks

- **i.MX 8M reuse fork — RESOLVED (2026-08-04): reuse is NOT viable → proceed with the Pi transplant.** **Three independent community projects all draw the line right below the D10** (community term: **"gen4"**): (1) OpenNeato/renjfk supports D3–D7, excludes D8–D10 ("different board, password-locked serial port"); (2) brainslug/"fang of vacuula" does gen2/gen3, says gen4 is "a completely different board, chip and firmware — cannot interface with directly"; (3) 94-psy got a D7 working then suspended. Plus Neato firmware is encrypted/signed and i.MX HABv4 secure boot blocks unsigned images. **No public root path exists for the D10, and even the "keep the board, talk over serial" shortcut is closed — full transplant is the *only* path.** The SoC is a modest single-A53 SoloLite anyway (Pi 4 on hand is far stronger). Transplant now justified by the **locked boot chain** (correct), not "no OS" (wrong — there *is* a Linux brain). Sources in build doc + links below.
- **i.MX 8M variant — RESOLVED** (`MIMX8MN1DVTJZAA` = Nano SoloLite, industrial; 1× A53 + 1× M7, no GPU/NPU). First digit hard to read (`1` SoloLite vs `6` Quad) but moot now — reuse is off the table regardless.
- **UART header (bonus from research):** OpenNeato documents the D3–D7 debug port as 4-pin **`RX / 3.3V / TX / GND`**, **3.3 V logic**. The D10's 4-pin header (bottom of board, see `shielding-removed.jpg`) is likely the same convention — noted only for optionally *watching* the locked boot log (blog material); needs a USB-TTL adapter or the Pi's UART.
- **Roller brush stall current** — **TBD, the last gate on the driver order.** (Wheels **resolved**: ~2.1 A / ~2.4 A → DRV8871 ×2. Blower **resolved**: 2.0 A, no driver.)
- Blower wire count — confirm 4 (PWM fan) vs 2 (plain brushed motor).
- Encoder channel count — 6 harness wires = quadrature; 5 = single channel, direction-blind.
- Encoder supply voltage (3.3 V vs 5 V) — determines whether a level shifter is needed.
- LDS 2.2 packet format — **largely SOLVED by research (2026-08-04).** The Neato LDS ("Piccolo LDS") protocol is fully documented (ssloy repo): **8N1, 3.3 V, 115200 baud; 90 packets/rev × 22 bytes × 4 readings = 360 readings/rev (1980 bytes)**; each reading has distance + signal-strength + 2 warning flags. Spin motor is **host-driven PWM, closed-loop on the RPM reported in the data** (open-loop ~3.3 V @ ~60 mA ≈ 240 rpm; closed-loop recommended). Our LDS 2.2 is this same family — expect a match or trivial tweak; just confirm on capture. Data connector `Red +3.3/5V · Brown RX · Orange TX · Black GND`, motor connector `Red PWR · Black GND`.
- Battery 6-pin pinout — **TBD** before connecting.
- Sensor logic levels (bumper/cliff/wall/drop) — **TBD** when wiring.
- Internal mounting space for Pi+ESP32 — **TBD** once old board fully removed.
- **⚠️ THERMAL (NEW risk from prior art, 2026-08-04)** — the closest analogous project (`94-psy/OpenNeato`, a D7→SBC+ROS2+Nav2 build) was **SUSPENDED INDEFINITELY** partly because its **SBC cooked at 85 °C+** (thermal throttling) inside the sealed chassis with no room for a heatsink/fan. **The Pi 4 runs hotter than the Radxa Zero 3W they used** — so cooling/ventilation is a real design constraint, not an afterthought. Plan airflow, a heatsink, or a cooler board (Pi Zero 2 W?) when mounting. Ties into the mounting-space item above.
- **✅ Architecture validated by that same failure** — 94-psy's *other* fatal issue was driving Neato's **factory serial-diagnostic port** for real-time control → buffer overflows, MCU crashes, dropped connections. **Our design sidesteps this entirely:** we gut Neato's board and run the real-time loop on our **own ESP32 (micro-ROS)**, not Neato's serial console. Their dead-end is evidence our Pi-4-brain + ESP32-real-time split is the right call.
- Nav2 tuning — known hard/fiddly (documented, big community).
- "Lost" status detection (localization loss) — least clean signal to detect.

---

## Reference links

- Neato XV-11 / Piccolo LDS protocol (full packet format + motor control) — https://github.com/ssloy/neato-xv11-lidar
- **94-psy/OpenNeato** — closest prior art (D7 → SBC + ROS 2 + Nav2). **Suspended**; read its README for the thermal + serial-bottleneck post-mortem — https://github.com/94-psy/OpenNeato
- **renjfk/OpenNeato** — D3–D7 cloud replacement (keeps Neato board, talks over serial). Debug-port pinout `RX/3.3V/TX/GND` — https://github.com/renjfk/OpenNeato
- Philip2809/neato-brainslug ("fang of vacuula") — ESP-on-serial local control for **gen2/gen3** Neatos. Reviewed 2026-08-04: states **gen4 (= D8/D9/D10) is "a completely different board, chip and firmware, cannot interface with directly"** — a *third* independent confirmation the D10 is locked/uncharted and full transplant is the only path — https://github.com/Philip2809/neato-brainslug
- XV-11 LDS on Raspberry Pi (C++, motor control) — https://github.com/berndporr/neato-xv11-lidar
- XV-11 LIDAR tutorial (ev3dev) — https://www.ev3dev.org/docs/tutorials/using-xv11-lidar/
- OpenNeato (D3–D7 context) — https://github.com/renjfk/OpenNeato
- ROS 2 Nav2 — https://docs.nav2.org
- slam_toolbox — https://github.com/SteveMacenski/slam_toolbox
- micro-ROS — https://micro.ros.org
- HA MQTT Vacuum — https://www.home-assistant.io/integrations/vacuum.mqtt/

---

## Photos captured so far (for the GitHub writeup)

- Battery label (14.4 V 4S2P) + 6-pin connector
- Mainboard both angles (NXP LPC51U68 visible, RF shield, USB-C, JST connectors)
- **RF shield removed** (`shielding-removed.jpg`) + chip close-ups revealing the Linux brain: `board-imx8m-soc.jpg` (i.MX 8M SoC + PCA9450 PMIC), `board-nanya-dram-and-lpc51u68.jpg` (DDR3 + the LPC51U68 in shot together), `board-kingston-emmc.jpg` (4 GB eMMC), `board-soc-cluster-overview.jpg`
- Button/UI board (TP1–TP22)
- **LiDAR LDS 2.2 base board** — underside (silkscreen `290-1044 REV 4`, `J2 MAIN`, `J3 MOTOR`) + chip side (LM393)
- Elegoo kit contents sheet
- Drive wheel motor side-on (`260-0016 14.4V`) + encoder board close-ups (`LEGO WHEEL ENCODER ASY: 915-1055`, `STD-3`)
- **Left motor winding-resistance measurement in progress** (`measure-left-motor-winding-resistance.jpg`) — probes on the two chunky posts flanking the encoder disc
- **Multimeter panel** (`multimeter.jpg`) — MS8233A, shows the `MAX 10A FUSED` label
- Roller brush motor in situ (`905-0460-RoHS 14.4VDC`)
- Blower label (EVERFLOW `F121225BU`, `DC14.4V 2.0AMP`)
- Mainboard in situ with harness attached

**Still to photograph:** roller brush motor terminals (for the next measurement); LiDAR turret top/laser module markings (optional — interface already known); internal space with mainboard removed; individual motor + sensor connectors close-up (for JST pitch sizing); **blower wire entry / connector close-up** (to settle the 4-wire question).
