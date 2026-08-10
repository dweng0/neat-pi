# Wheel encoder diagnosis — hall-probe bench session

_Recorded 2026-08-10. Tool: `hall-probe` (this dir) + `esp32-firmware/src/main.cpp` (STEP-2 drive+encoder firmware). Port `/dev/cu.usbserial-10` @ 115200._

## TL;DR (current best understanding)

**Rotation and both channels are real and working.** At `F 0` the counts are stable (no
drift); at `F 150` both A and B climb steadily and stop when idle → the shaft magnet spins
and both sensors read it. Raw **edge counts are usable** for "is it spinning / rough speed."

**The remaining fault is quadrature quality, not dead hardware:**
- Edge counts are **lopsided and inconsistent** — one run B ≈ 1.9× A, another run A ≈ 1.6× B.
  A healthy A/B pair is ~1:1 and stable.
- **`pos` random-walks** (e.g. −6 → +154 → −104) while driving *one* direction — a clean
  encoder gives monotonic pos. Direction decode is unreliable.
- Snapshot **levels A and B are almost always equal** (both 0 / both 1) → the two channels
  read nearly **in phase**, not 90° apart, which breaks the quadrature math.

**Likely cause:** the bare `~3.3k` resistor divider gives slow/ragged edges through the ESP32
logic threshold, and the firmware counts on **every CHANGE with no debounce/hysteresis** →
one real transition registers as several counts, unevenly per channel → inflated counts +
scrambled phase. **Durable fix:** clean the edges — a **Schmitt-trigger buffer (hysteresis)**
or a small **RC filter cap** per line instead of the bare divider.

_(Earlier theories, kept for history: "field too weak / air gap" — true that a stronger field
made A count, proving the sensor senses; and "phantom edges at idle" — did NOT reproduce, at
`F 0` it's stable. The rig was physically handled throughout, so behaviour wasn't stationary
between tests.)_

## Encoder recap

- Firmware keeps `edgeCountA`, `edgeCountB` (raw transitions) and `encPos` (signed
  quadrature position) as `volatile` globals, updated in an ISR on **CHANGE** interrupts.
- Counts are **firmware-side state**: they survive a host reconnect and only reset on `Z`
  (`encoderZero()`) or an ESP32 reboot. (This is why the count "sat at 582 across script
  restarts" — nothing was stuck; it's just the MCU holding its running total.)
- The encoder magnet sits on the **motor shaft, before the gearbox** (per
  `encoder-test.py` header) — so a spinning motor should whip it past sensor A hundreds of
  times/second.
- Pins: GPIO32 = A (blue), GPIO33 = B (yellow), each via a ~3.3k-to-GND divider off the
  open-drain, idle-high 5 V output.

## What was observed (in order)

| Test | A | B | pos | Reading |
|------|---|---|-----|---------|
| Driven `F 150`, no boost | **frozen at 62** | 0 | 0 | levels A=1 B=1, never toggled |
| `Z` then `F 150`, no boost | **stayed 0** | 0 | 0 | zero edges during a real 4 s drive |
| Speed 0 + **handheld magnet** at sensor | 62 → 120, level 1→0 ×5 | 0 | ticked to 1 | A responds instantly to a magnet |
| Driving with **booster magnet** stuck on shaft magnet (user paste) | 325360 → 327018 | 115328 → 116246 | ~-8430 → -8600 | both channels climbing |
| `Z` then `F 150`, **booster on** (my run, 4 s) | +41909 | +25630 | → -7545 | both count; A/B edge ratio ≈ 1.64 (A>B) |
| `F 0`, booster off, magnet repositioned | frozen at 1 | frozen at 31 | frozen at -2 | **stable at rest — no phantom counts** |
| `F 150`, booster off (real rotation) | 1 → 3529 | 1 → **6695** | random-walk −104…+154 | both climb steadily; **B ≈ 1.9× A**; pos non-monotonic; levels A≈B (in phase) |

## What this proves

- **Sensors A and B, wiring, GPIO, firmware: all functional.** Both channels produce edges
  and `pos` decodes direction. Neither sensor is dead; nothing was fried by a bridge.
- The fix that made it "work" was **adding magnetic field strength** — either a handheld
  magnet at the sensor, or a booster magnet stacked on the shaft magnet. That points
  squarely at **field strength / air gap** as the fault, not the electronics.

## Remaining problems / open questions

0. **Phantom edges at rest (PRIME SUSPECT).** At `F 0`, motor off, nothing moving, channel
   A keeps accumulating — impossible for real magnetic edges, so the GPIO32 input is
   chattering on noise (divider level parked on the ESP32 logic threshold). Decisive check:
   sit at `F 0` for ~10 s and see if it's **only A** (→ retune GPIO32 divider / add filter
   cap / hysteresis) or **A and B both** (→ broader grounding/noise issue). This likely
   inflated the big "working" drive counts and skewed the 1.64 A/B ratio below.
1. **The shaft magnet is not spinning (observation, booster removed).** If the
   motor is driving but the magnet on the shaft isn't rotating, that alone explains why
   real drives never counted — the target the sensor is supposed to see simply isn't
   moving past it. Suspect: magnet detached / slipped on the shaft, or the shaft/motor
   isn't actually turning under load. **This is now the prime root-cause candidate.**
2. **Field too weak on its own.** Even when the shaft magnet does move, its field at the
   sensor was below the trip threshold without a boost. Durable fix = close the air gap
   (sensor closer to the magnet), reseat/secure the magnet, or fit a stronger magnet.
3. **Channel B under-counts.** With the field boosted, A/B edge ratio ≈ **1.64** (a healthy
   quadrature pair should be ≈ 1.0). B misses ~40% of its edges → `pos` is not yet
   trustworthy for odometry. Suspect: weaker field at the B sensor, or the GPIO33 divider
   threshold set a touch too high.

## Next steps

- Confirm whether the **shaft magnet physically rotates** when the motor is driven
  (eyeball the shaft, not just the wheel). If it doesn't spin, fix the magnet mount first —
  everything downstream depends on it.
- Once it spins on its own: measure the **A/B edge ratio** and pull it toward 1.0 by
  improving the B field/gap or tuning the GPIO33 divider.
- Re-run baseline with `hall-probe` (or the python snippet in the session) with **no
  booster magnet** to prove the rig stands on its own.

## Firmware command reference

`F <0-255>` fwd · `R <0-255>` rev · `S` stop · `B` brake · `E` read encoder once ·
`Z` zero the counts. Firmware also streams `[enc] A=.. B=.. pos=..` (~2/s) while moving.
Port is **single-owner** — close any running `hall-probe` / serial monitor before opening it.
