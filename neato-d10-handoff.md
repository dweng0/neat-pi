# Neato D10 Brain-Transplant — Handoff / Continuation Doc

**Purpose:** snapshot of where this project stands so it can be resumed in a fresh conversation without re-deriving anything. Pairs with the main build doc: `neato-d10-brain-transplant.md`.

**Last updated:** 2026-08-06

---

## One-paragraph context

The user has a Neato D10 robot vacuum. Neato went bust; Vorwerk killed the cloud (Q4 2025). **Working plan: "brain transplant"** — rebuild as a ROS 2 robot on a Raspberry Pi 4 + ESP32, controlled from Home Assistant as an MQTT vacuum, keeping the mechanically/electrically standard parts. The user wants **full vacuum function** (keep brush + blower motors), and intends to **blog the findings on GitHub** eventually.

**Premise correction (2026-08-04), resolved:** the transplant was originally justified by "bare-metal LPC51U68, no OS." That was **wrong** — lifting the RF shield revealed a full **NXP i.MX 8M Nano SoloLite Linux computer** beside the LPC51U68 (a **two-brain design**). But the *conclusion* still holds: the **D8/D9/D10 boot chain is locked** (OpenNeato excludes them; encrypted/signed firmware; i.MX HAB secure boot), so **reusing the existing brain is not viable. Transplant confirmed.**

The user is hands-on: soldering gear, a multimeter, a Pi 4, an Elegoo Arduino starter kit, comfortable opening hardware. **Teardown is complete — every motor measured, every driver chosen, vetted, ordered and now ARRIVED.**

**Latest (2026-08-06, session 2):** **the whole power stage arrived** — 2× DRV8871, Cytron MD13S, side-brush MOSFET, plus the speaker amp. That **unblocks build step 2** (real wheel motor + encoder). This session's bench work was **sensors**: the digital safety switches are fully characterised, and the cliff/wall sensor is characterised via prior art (the multimeter couldn't crack it — see below). **Focus now: build step 2 (encoders → odometry) — the drivers are here, so it's the top real-hardware milestone.**

---

## Key confirmed facts (from hands-on teardown)

| Thing | Finding |
|---|---|
| **Main application processor** | **NXP i.MX 8M Nano SoloLite** (`MIMX8MN1DVTJZAA`, industrial) under the RF shield — 1× Cortex-A53 (Linux) + 1× Cortex-M7 (real-time), no GPU/VPU/NPU. + `NANYA` DDR3L + `Kingston` 4 GB eMMC + `NXP PCA9450B` PMIC. A Linux computer, but boot chain locked → **discard**. Photos: `board-imx8m-soc.jpg`, `board-nanya-dram-and-lpc51u68.jpg`, `board-kingston-emmc.jpg`, `board-soc-cluster-overview.jpg`. |
| Real-time MCU | **NXP LPC51U68** (Cortex-M0+) — the body controller, not the whole brain. Discarded with the board. |
| Mainboard P/N | `520-0394 Rev.B`, USB-C "SW Update" service port. **Discard** (reuse ruled out). |
| **LiDAR** | **Neato LDS 2.2** (`290-1044 REV 4`, © 2019). **8N1 UART, 3.3 V, 115200 baud.** `J2 MAIN` (5 V power + serial), `J3 MOTOR` (host-driven PWM, closed-loop off reported RPM). Protocol documented (Piccolo/XV-11 family) — expect a match on capture. |
| Battery | Li-ion **14.4 V nominal** `4S2P`, 6200 mAh / 89 Wh. Real range **12 V (empty) → 16.8 V (full)**. **"Smart battery" — BMS / protection / balancing lives INSIDE the pack** (D-series moved it off the mainboard), talking **SMBus** (I²C-like) to a **TI fuel-gauge IC, with serial-number authentication.** **6-pin colours: red, white, black, black, blue, yellow** (all same gauge) — likely red = +, black×2 = − / gnd, yellow = thermistor, **blue + white = SMBus data/clock.** **The charge circuit was on the discarded mainboard.** **Measured 2026-08-06: red↔black = 0 V → BMS output FET latched off** (cells hidden, health unknown from the terminals). **Bay = triangular prism ~4 cm/side × 22 cm (~150 cm³, ~8× 18650). Dock = `19.5 V / 1.5 A`** (label `905-0575`). Full battery/charging plan (reuse cells + dumb BMS) in the Battery/charging note below. |
| **Drive wheel motors** | `260-0016`, 14.4 V, brushed DC. **Stall MEASURED: L ≈ 2.1 A (R 6.7 Ω), R ≈ 2.4 A (R 5.9 Ω).** Matched pair. → **DRV8871 ×2 (ARRIVED).** |
| **Wheel encoders** | `LEGO WHEEL ENCODER ASY: 915-1055 REV`, `STD-3`. Solid disc → magnetic (Hall). **Harness CONFIRMED: 6 wires = 2 thick power (red/black) + 4 encoder = QUADRATURE (Vcc/GND/A/B), direction-aware.** Good for odometry + slam_toolbox. Supply voltage (3.3 vs 5 V) still TBD. |
| **Roller brush motor** | `905-0460-RoHS 14.4VDC`, brushed DC. **Winding R MEASURED: ~1.9 Ω → stall ≈ 7.6 A.** Beefy — 3× the wheels. → **Cytron MD13S (ARRIVED).** **Harness: 2 thick power + 3 thin = Hall tacho (Vcc/GND/signal)** → free brush-jam / RPM detection into an ESP32 GPIO. Bidirectional driver → **auto-unjam in software later**. Tacho supply voltage TBD. |
| **Side brush motor** | Small brushed DC can, front-corner sweeper. **2 wires only, EMI cap, no sensor.** **Winding R MEASURED: 20–30 Ω → stall ≈ 0.5–0.7 A.** → **logic-level MOSFET module (ARRIVED)**. Photo: `brush-motor.jpg`. |
| **Blower/vacuum** | EVERFLOW `F121225BU (AFX19bR)` — **`DC14.4V 2.0AMP`** on the label. `…BU` = 4-wire PWM family → brushless w/ integrated driver, **needs NO H-bridge.** **4 wires CONFIRMED.** |
| **Front bumper switches** | **`BUMP SWITCH 290-0056` REV.8** — mechanical tactile click-switch. **User counts 4.** **CONFIRMED 2026-08-06: normally-open** (beeps when pressed) → **GPIO + `INPUT_PULLUP`, active-low, LOW = hit**, ~5 ms debounce. Photo: `front-bumper-switch.jpg`. |
| **Wheel-drop / lift switch** | **`DT-08` lever microswitch** (`3A 125VAC`), wheel arch — the "dead-man's" switch. **CONFIRMED 2026-08-06: normally-open.** Mechanism: on the ground the wheel is held up → lever open (**HIGH**); lift the robot → wheel falls → **presses** the lever (closed, **LOW**). So **LOW = wheel dropped / robot lifted → stop drive.** Same wiring + polarity as bumpers (GPIO + pull-up, active-low). One per drive wheel. Photo: `wheel-arch-dead-mans-switch.jpg`. |
| **Cliff / drop sensors** | **`LOUIE DRP SENSOR 290-1023 REV 2` (© 2017)** — downward IR reflectance, **discrete IR emitter + phototransistor** (two clear windows). **CHARACTERISED 2026-08-06 (via prior art + teardown):** runs on **~3.3 V** (signal swings 0–3 V → native ESP32 ADC, **no level shifter**); wire convention **black = GND, red = Vcc**, brown/yellow/green = emitter-drive + signal. **Host-driven: emitter is strobed/modulated + phototransistor sampled in sync** (ambient rejection) — replicate in firmware; a static DC diode test is defeated by an on-board decoupling cap (don't bother). **≥2 sensors** (two gang into a 10-pin at the old board; vendors sell as a "2× set"). Connector = **JST ZH 1.5 mm, 5-pin**. Photos: `under-carriage-sensor.jpg`, `cliff-sensor-front.jpg`, `cliff-sensor-connector.jpg`, `cliff-sensor-harness-cut-tails.jpg`. |
| **Side / wall sensor** | **CONFIRMED 2026-08-06: same `LOUIE DRP 290-1023` board as the cliff sensor** (silkscreen matches). Same ~3.3 V analog reflectance, same ADC + firmware strobe/sample treatment. **Overrides the forum "Sharp GP2Y0A51 wall sensor" lead** — that does not apply to this D10. One sensor type covers cliff + wall. |
| **Rear speaker** | Small speaker at the back. Keep/drive for audio cues (stuck/docked/bin-full). Plan: **class-D mono amp (PAM8302-class, ARRIVED)** fed from an **ESP32 DAC pin**. Nice-to-have, no build dependency. |
| Multimeter | **MS8233A**, 2000 counts. 10 A jack IS fused (`MAX 10A FUSED`). Ω 200 range = 0.1 Ω resolution. Diode mode open-circuit ~2 V (so a charging cap reads ~1.7 then OL — a false "diode"). |

**Important gotcha:** the "19.5 V 1.5 A" device plate is the **dock charge input**, NOT the battery. Tap the 14.4 V battery for the buck converter.

**Winding-resistance method (proven):** alligator clips (not hand probes), rotate shaft → STOP → settle → read lowest *stable* value. Subtract `0.1 Ω` leads. `stall ≈ 14.4 V ÷ R`.

**Cliff-sensor probing (learned the hard way):** don't try to DC-diode-test it. The fine-pitch JST fouls Dupont, the old board's ESD clamps contaminate readings through the ground plane, and an on-board cap fakes a diode. It's an analog, host-*strobed* reflectance sensor — to finish the pinout, **power it and observe**, don't meter it statically.

---

## Architecture (decided)

- **Home Assistant** — scheduling + notifications. Robot = HA **MQTT Vacuum** entity.
- **Raspberry Pi 4** — the brain. ROS 2 + Nav2 + slam_toolbox + a ~100-line MQTT↔ROS bridge node.
- **ESP32 (micro-ROS)** — real-time co-processor: motor PWM, encoder counting, sensor polling (bump/drop switches, cliff/wall ADC), battery voltage. Joins the ROS 2 graph over USB serial.

---

## Build infrastructure — set up & proven (2026-08-06)

**Step 1 (toolchain + first motor) is DONE.** What exists on the bench + in the repo:

| Thing | State |
|---|---|
| **ESP32 board** | **`ESP32-D0WD-V3`** (rev 3.1), dual-core 240 MHz, Wi-Fi+BT, **4 MB flash**. CH340C USB-serial, **no driver needed on macOS**. **Auto-reset via RTS works** — no BOOT/EN dance. Pins labelled **`D25`/`D26`/`D27`** = GPIO25/26/27. |
| **Serial port** | Enumerates as **`/dev/cu.usbserial-110`** (⚠️ a pre-existing `usbserial-21420` is a red herring). If replugged: `ls /dev/cu.usbserial*`. |
| **Toolchain** | **PlatformIO** (chosen over Arduino IDE — smoother road to micro-ROS). Project venv at **`.esp-venv/`** (gitignored). Xtensa toolchain cached. |
| **Firmware** | `esp32-firmware/` — `platformio.ini` (board `esp32dev`, port + 115200 baked in) + `src/main.cpp`. Currently the **STEP-1 motor driver**: `F <0-255>` fwd / `R <0-255>` rev / `S` stop / `B` brake. LED (GPIO2) heartbeats. Build+flash: `.esp-venv/bin/pio run -d esp32-firmware -t upload`. |
| **Bench test harness** | `hardware-scripts/test-scripts/motor-test.py` (+ README). `--demo` ramp+reverse, `--cmd "F 200"` one-shot, no args = interactive `motor>` prompt. Run via `../../.esp-venv/bin/python3 motor-test.py`. New bench scripts slot in here. |

**Step-1 bench rig (retire it — it was only the toolchain proof):** ESP32 `D25→L293D EN`, `D26→IN1`, `D27→IN2`, `VIN→Vcc1`, motor across the bridge, common ground. **Do NOT** put a real wheel motor on the L293D (stall >2 A cooks it) — that's what the DRV8871s (now arrived) are for.

---

## Motor driver plan — COMPLETE, VETTED & ARRIVED

Full vacuum = **5 motors**, **4 drivers** (blower needs none).

| Motor(s) | Stall | Driver | Confirmed part | Status |
|---|---|---|---|---|
| 2× drive wheels | ~2.1 / 2.4 A, quadrature encoders | **2× DRV8871** | **Adafruit ADA3190** (6.5–45 V in, 3.6 A peak, IN1/IN2, default 30 kΩ Rlim ≈ 2 A) | ✅ **ARRIVED** |
| Roller brush | ~7.6 A, + Hall tacho | **1× Cytron MD13S** (bidirectional) | **Pi Hut SKU 106189**, 6–30 V, 13 A cont / 30 A peak, 3.3 V & 5 V logic, PWM+DIR, 20 kHz | ✅ **ARRIVED** |
| Side brush | ~0.5–0.7 A, 2-wire | **1× logic-level MOSFET module** | **DFRobot Gravity MOSFET Power Controller** (3.3 V trigger, VIN 5–36 V/20 A). ⚠️ 1 kHz max → on/off only (fine for side brush). ⚠️ add flyback diode (Elegoo kit). | ✅ **ARRIVED** |
| Blower/vacuum | 2.0 A, 4-wire PWM brushless | **NONE** — GPIO PWM (~25 kHz) + free tach | — | ✅ resolved |

**Blower wiring (verify before trusting):** black = GND, red = +14.4 V, yellow = tach out, blue = PWM in. Do **not** H-bridge it; do **not** measure its winding resistance.

**Key principle:** size drivers to **stall** current, not running. (OEM XV-11 drove the *wheels* with a ~2.8 A `A3950`, so DRV8871 at 3.6 A is correctly sized.)

---

## Buck converter / 5 V rail — PART ORDERED (⚠️ in transit)

**Chosen: 7 A switching UBEC** (RC-style, 2–7S in, 5.25 V ±0.5 V out, 300 kHz, ~92% eff., shielded). Feeds Pi 4 + 2× ESP32 + LiDAR (~3.5 A sustained est.).

- **⚠️ Setup step:** output is fixed ~5.25 V — **meter it first, confirm ~5.1–5.3 V, then wire the Pi.**
- **⚠️ On arrival: confirm it's the 7 A variant** (multi-variant listing "FPV RC UBEC 5V 3A 5A 7A 15A").
- **Rejected:** 3 A modules (brown out a Pi 4 under Nav2). XL4015 = bench-only. Pololu D36V50F5 correct but £38.

---

## Elegoo kit — what's useful here

- **L293D** (dual H-bridge, ~600 mA/ch) → step-1 toolchain proof only (now done). Too weak for final use.
- **PN2222 NPN (×2) + flyback diode (×2)** → LiDAR spin-motor drive circuit. Diodes also cover the side-brush MOSFET flyback if the Gravity module lacks one.
- **Thermistor** → reference when mapping the battery 6-pin connector.
- UNO R3, breadboard, jumpers, sensors → general bench use.

---

## Parts status

**Already have:** Pi 4, soldering iron+solder, multimeter, breadboard, Arduino/Elegoo kit.

**Arrived 2026-08-06:**
- ✅ **2× ESP32 WROOM-32 (USB-C, CH340C)** — one flashed with step-1 firmware, on the bench.
- ✅ **2× Adafruit DRV8871 (ADA3190)** — drive wheels.
- ✅ **1× Cytron MD13S (SKU 106189)** — roller brush.
- ✅ **1× DFRobot Gravity MOSFET Power Controller** — side brush.
- ✅ **Speaker amp** (PAM8302-class class-D mono) — rear speaker audio cues.

**→ The whole power stage is now in hand. Build step 2 is unblocked.**

**Ordered (still arriving):**
- 8-ch 24 MHz logic analyzer (PulseView/sigrok) — decode LDS 2.2 + encoders. **Gates LiDAR (step 3).**
- **1× 7 A switching UBEC** (eBay `26-14963-25714`, £6.60, ETA **10–17 Aug**, slowest part). ⚠️ confirm 7 A + meter to ~5.25 V before wiring the Pi. **Not on critical path** (bench is USB-powered).
- Heat-shrink assortment, Dupont jumper assortment.

**No longer needed:**
- ~~JST ZH pigtail for the cliff sensor~~ — the harness was **cut** and the sensor keeps its native mated 5-pin plug as a permanent pigtail. (A JST kit is still handy for encoder/tacho/battery plugs — pitches TBD.)

**On hold until measured / confirmed:**
- Logic level shifter — **cliff/wall sensors resolved (3.3 V, none needed)**; only if the encoders / roller tacho turn out 5 V logic (TBD).
- JST kit for encoder/tacho/battery plugs — blocked on measuring those pitches.
- T10 Torx long-reach driver — recessed case screws (optional).
- Standoffs/mounts — blocked on measuring free internal space + thermal plan.

---

## NEXT ACTIONS (resume here)

**✅ DONE this session (2026-08-06): switches + cliff/wall sensor characterised.** Bumper ×4 = NO, LOW = hit. Wheel-drop = NO, LOW = lifted. Cliff + wall sensor = `LOUIE DRP 290-1023`, ~3.3 V analog reflectance, native ADC, host-strobed, ≥2 sensors, same part for both — see confirmed-facts table.

1. **[TOP PRIORITY — doable NOW, newly unblocked] Build step 2 — encoders → odometry.** Wire a **real** Neato wheel motor + a **DRV8871** (⚠️ the OEM pack is dead/latched — power the DRV8871 VIN from a **bench PSU / 12–15 V ≥2 A supply**, NOT the L293D which can't take the ~2 A stall), drive it from the existing serial protocol, then read the **quadrature encoder (A/B)** into the ESP32 for distance + direction. Fat, easy wires — no fiddly connectors. This is the real-hardware version of what step 1 proved on a stand-in, and the drivers are finally here. **Do this first.**
2. **[Quick, ~10 min, do when convenient] Finish the cliff-sensor pinout functionally.** Power the sensor: **red → 3.3 V, black → GND**; then tie each of **brown/yellow/green** high through a resistor while watching the emitter window **through a phone camera** (IR shows as a purple/white glow) → that's the emitter-drive wire; the remaining wire read on the ADC is the signal. Resolves the last residual. (Not blocked on anything — just wasn't worth grinding with a meter.)
3. **[Also open now] Encoder / roller-tacho Vcc.** While on the bench with the wheel motor, meter the encoder + roller-tacho supply voltage (3.3 vs 5 V) → decides level-shifter need on those signal lines.
4. **[When logic analyzer arrives] LiDAR bring-up:** power the LDS, drive spin motor (PN2222 + diode), clip analyzer on `J2`, confirm LDS 2.2 packet format vs documented Piccolo/XV-11.
5. **[DISCUSS NEXT SESSION — direction decided, execution pending] Battery: reuse OEM cells + separate on-bot BMS (option 3).** Approach chosen this session (see Battery/charging note). **Pending inputs before building:** (a) result of the dock + old-board **charge test** — did red↔black climb off `0 V`? that tells us if the OEM 18650 cells are healthy/reusable; (b) **cell type** — measure one cell's diameter (`~18 mm` = 18650, `~21 mm` = 21700); (c) full deep-dive — safe cell harvesting, exact BMS + dock-fed charger parts, `4S1P` vs `4S2P`. ⚠️ don't leave a dead Li-ion charging unattended.
6. **[When UBEC arrives, ETA 10–17 Aug] Power rail:** confirm 7 A variant + meter to ~5.25 V before wiring the Pi.
7. **[When board removed] Measure internal mounting space + plan cooling** — see thermal risk below. Also count the full set of cliff sensors (≥2 confirmed; front corners + centre TBD).

**🗣️ QUEUED FOR DISCUSSION (not yet explored):**
- **Battery / charging — investigated 2026-08-06; DIRECTION DECIDED, detail deferred to next session.**
  - **Contact charging** (dock rings, **`19.5 V / 1.5 A`** — confirmed off the dock label `905-0575 Rev B`, supply `S030BBM1950150`; photo `dock-input-output-label.jpg`), NOT wireless/inductive. That `19.5 V` is what our own charger will be fed; `1.5 A` is the charge-current ceiling.
  - **OEM pack is a "smart battery":** BMS + protection + balancing + a **TI fuel-gauge IC live INSIDE the pack**, talking **SMBus** (I²C-like, 2 wires) to the host, **with serial-number authentication so only genuine packs work.** Original charge circuit was on the discarded mainboard. Pack reads **0 V (BMS FET latched off)**.
  - **Why NOT the OEM path:** keeping it means either the old board alive as a charger (hogs the board bay the Pi needs) or reverse-engineering an *authenticated* SMBus handshake (likely infeasible; the arriving logic analyzer can *read* SMBus but can't defeat auth). Also note the Pi/board bay — not the battery bay — is where the brain goes, so the space win comes from ditching the old board, not shrinking the battery.
  - **✅ DECIDED (drone/RC model): dumb cells + BMS-on-the-bot.** Chain: `dumb 4S cells → on-bot 4S BMS (protection + balance) → 14.4 V power bus + a dock-fed 4S CC/CV charger (19.5 V in → 16.8 V out, ≤1.5 A) → ESP32 ADC voltage-divider for SoC / return-to-dock`. No auth, no SMBus RE, swappable, Pi-controlled charging. **Try option 3 first: reuse the OEM 18650 cells + a separate dumb BMS**; build a fresh 18650 pack only if the cells are dead.
  - **Battery bay:** triangular prism, **~4 cm/side × 22 cm (~150 cm³)** → holds **~8× 18650 = the OEM `4S2P`**; cylinders nest in a triangle (the "stacked triangle" seen). No catalogue pack is triangular → replacement is cell-reuse or DIY-shaped. `4S1P` = half runtime/easier fit; `4S2P` = OEM runtime.
  - **Parts to search** (specs to confirm in brackets): `4S 16.8V Li-ion BMS balancing 20A` [4S · balancing · ≥20 A · common-port]; `DC-DC buck CC CV 16.8V 4S charger` [input covers 19.5 V · out 16.8 V · CC/CV · ~1.5 A]; `18650 cells` + `4S holder`/`spot welder` (if building); `voltage sensor module 25V` or a resistor divider (keep <3.3 V into the ESP32 ADC); `XT60` + `JST-XH balance` connectors; bench safety — `iMAX B6 balance charger`, `LiPo safe bag`, `4S low-voltage alarm`.
  - **4S/2P meaning:** S = series (adds voltage; 4 × 3.6 V ≈ 14.4 V nom, 16.8 V full); P = parallel (adds capacity/runtime). 4S2P = 8 cells.
- **Speaker integration detail** — amp now in hand; ESP32 DAC wiring, what cues to play. Nice-to-have.
- **Voice control via onboard mic array (Phase 2+)** — `HK-ARRAY MIC-V1.1` USB mic array (UAC, driverless on the Pi; real connector is a 5-pin header carrying USB lines). Plan: `wyoming-satellite` on the Pi → HA Assist (STT→intent→TTS) → vacuum entity command → Nav2. Caveats: too loud to voice-control while cleaning (docked/idle only); named zones need SLAM/Nav2 up first (slot after steps 5–6 of build order); extra always-on process → thermal budget; mic ports need an air path (mount up top). Photo: `microphone-array.jpg`.

---

## Build order (for reference)

1. ✅ **DONE (2026-08-06)** — ESP32 + L293D + stand-in motor → serial drive with PWM speed + direction. Toolchain proven.
2. Add encoders → odometry. **← next real hardware milestone; drivers have ARRIVED, so this is now unblocked and top of the list.**
3. LiDAR bring-up → power, spin, capture `J2`, confirm/adapt packet decoder. (Waits on logic analyzer.)
4. Wire the digital sensors (bump ×4, wheel-drop) + cliff/wall ADC → safety inputs. (Switches + cliff/wall already characterised; cliff emitter-vs-signal pin is a 10-min functional test here.)
5. Bare-bones MQTT bridge → HA `start` drives robot forward. (Proves full HA→Pi→ESP32→motor pipeline before SLAM.)
6. slam_toolbox + Nav2 → mapping, localization, coverage. (Nav2 tuning is the time-sink.)

---

## Open questions / risks

- **i.MX 8M reuse — RESOLVED: not viable.** Three community projects all draw the line right below the D10: OpenNeato/renjfk (D3–D7); brainslug (gen2/3, "gen4… cannot interface directly"); 94-psy got a D7 working then suspended. Firmware encrypted/signed + i.MX HABv4 secure boot. **Full transplant is the only path.**
- **Cliff / wall sensor** — **largely RESOLVED 2026-08-06.** Vcc ~3.3 V (native ADC, no shifter); analog reflectance; host-strobed; ≥2 sensors gang-of-2; wall sensor is the same part. Residual: full sensor count (pending full undercarriage) + which of brown/yellow/green is emitter-drive vs signal (10-min functional test — action 2).
- **Encoder / roller-tacho supply voltage** — **TBD** (3.3 V vs 5 V). Determines level-shifter need on those signal lines. Meter during step 2.
- Battery 6-pin pinout — **partially mapped 2026-08-06:** red, white, black, black, blue, yellow (all same gauge); likely red = +, black×2 = − / gnd, yellow = thermistor, white/blue = thermistor / serial. **BMS is in-pack (smart battery); serial interface on the blue wire.** Pack reads 0 V (latched) — see charging note.
- Bump/wheel-drop switch logic — **RESOLVED** (both NO, active-low, LOW = event).
- Internal mounting space for Pi+ESP32 — **TBD** once old board removed.
- **⚠️ THERMAL** — closest prior art (`94-psy/OpenNeato`, D7→SBC+ROS2+Nav2) was **SUSPENDED** partly because its SBC cooked at **85 °C+** inside the sealed chassis. The Pi 4 runs *hotter* → cooling is a design constraint, not an afterthought.
- **⚠️ POWER** — a 3 A 5 V rail browns out a Pi 4 under Nav2 (reboots / SD corruption). Hence the 7 A UBEC.
- **✅ Architecture validated by that failure** — 94-psy's other fatal flaw was driving Neato's factory serial-diagnostic port for real-time control. Our design guts that board and runs the real-time loop on our own ESP32 (micro-ROS) → sidesteps it.
- LDS 2.2 packet format — largely solved by research; confirm on capture.
- Nav2 tuning — known hard/fiddly.
- "Lost" status detection (localization loss) — least clean signal to detect.

---

## Reference links

- Neato XV-11 / Piccolo LDS protocol — https://github.com/ssloy/neato-xv11-lidar
- **94-psy/OpenNeato** — closest prior art (D7 → SBC + ROS 2 + Nav2). Suspended; thermal + serial post-mortem — https://github.com/94-psy/OpenNeato
- **renjfk/OpenNeato** — D3–D7 cloud replacement; debug-port pinout `RX/3.3V/TX/GND` — https://github.com/renjfk/OpenNeato
- Philip2809/neato-brainslug — gen2/gen3 local control; confirms gen4 (D8/D9/D10) locked — https://github.com/Philip2809/neato-brainslug
- **Neato drop/cliff sensor (290-1023 LOUIE DRP) discussion** — Robot Reviews "D5 Cliff Sensors Not Responding": https://robotreviews.com/chat/viewtopic.php?t=22133 ; vendor confirming 2×-set: https://casello.de/products/neato-botvac-d-connected-2x-drp-sensor-kabel-290-1023-louie-drop-rev2
- **Neato smart-battery / in-pack BMS (D-series)** — Robot Reviews "Fix/replace Battery BMS": https://www.robotreviews.com/chat/viewtopic.php?t=22794 ; "Bricked my Botvac Connected battery pack": https://robotreviews.com/chat/viewtopic.php?t=22714
- XV-11 LDS on Raspberry Pi (C++) — https://github.com/berndporr/neato-xv11-lidar
- ROS 2 Nav2 — https://docs.nav2.org
- slam_toolbox — https://github.com/SteveMacenski/slam_toolbox
- micro-ROS — https://micro.ros.org
- HA MQTT Vacuum — https://www.home-assistant.io/integrations/vacuum.mqtt/
- Cytron MD13S (roller driver) — Pi Hut SKU 106189, Cytron lib: https://github.com/CytronTechnologies/CytronMotorDriver

---

## Photos captured so far (for the GitHub writeup)

- Battery label (14.4 V 4S2P) + 6-pin connector
- Mainboard both angles + RF shield removed (`shielding-removed.jpg`) + chip close-ups: `board-imx8m-soc.jpg`, `board-nanya-dram-and-lpc51u68.jpg`, `board-kingston-emmc.jpg`, `board-soc-cluster-overview.jpg`
- Button/UI board (TP1–TP22)
- LiDAR LDS 2.2 base board — underside (`290-1044 REV 4`, `J2 MAIN`, `J3 MOTOR`) + chip side (LM393)
- Elegoo kit contents sheet
- Drive wheel: `left-wheel-motor.jpg`, `left-wheel-motor-2.jpg`, `left-wheel-motor-wiring.jpg`, `left-wheel-chassis.jpg`, `left-wheel-chassis-with-motor-and-wiring.jpg`, `right-wheel-chassis-connected.jpg`; encoder close-ups (`LEGO … 915-1055`, `STD-3`)
- **Left motor winding-resistance measurement** (`measure-left-motor-winding-resistance.jpg`)
- Multimeter panel (`multimeter.jpg`)
- Roller brush motor (`roller-brush-motor.jpg`, `roller-motor.jpg`) + board/top-down (`board-and-topdown-spinning-brush-motor.jpg`)
- **Side brush motor** (`brush-motor.jpg`) — 2-wire small can, EMI cap
- Blower label (EVERFLOW `F121225BU`, `DC14.4V 2.0AMP`) + `blower-motor.jpg`
- **Sensors (2026-08-05):** front bumper switch (`front-bumper-switch.jpg`, `290-0056`), wheel-arch dead-man's microswitch (`wheel-arch-dead-mans-switch.jpg`, `DT-08`), under-carriage cliff sensor (`under-carriage-sensor.jpg`, `LOUIE DRP 290-1023`)
- **Cliff sensor deep-dive (NEW 2026-08-06):** `cliff-sensor-front.jpg` (two emitter/detector windows), `cliff-sensor-connector.jpg` (5-wire JST + silkscreen), `cliff-sensor-harness-clamp.jpg` (ferrite/strain-relief clamshell + white plug), `cliff-sensor-harness-cut-tails.jpg` (harness cut, native plug kept as pigtail — hero shot for episode 12)
- **Battery / dock (NEW 2026-08-06):** `dock-input-output-label.jpg` (dock `19.5 V / 1.5 A`, PN `905-0575 Rev B`). Still to shoot: battery bay + nested 18650 triangle, a single cell for diameter (18650 vs 21700), the 6-pin connector close-up.
- **Mic array (2026-08-05):** `HK-ARRAY MIC-V1.1` USB microphone array (`microphone-array.jpg`)
- **Step 1 bench build (2026-08-06):** ESP32 pinout D25/D26/D27 (`esp32-devkit-pinout.jpg`), L293D on breadboard (`l293d-breadboard-step1.jpg`), **first motor + fan spinning** (`step1-first-motor-fan-test.jpg` — hero for episode 11)

**Still to photograph:** LiDAR turret top/laser module markings (optional); internal space with mainboard removed; individual connectors close-up (JST pitch sizing for encoder/tacho/battery); wheel-encoder + brush-tacho Vcc pins while metering; full undercarriage showing all cliff-sensor positions; cliff-sensor emitter glowing on a phone camera during the functional pinout test.
