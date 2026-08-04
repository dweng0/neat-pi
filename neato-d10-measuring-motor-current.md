# Bench Procedure — Measuring Motor Current (Neato D10 salvage)

**Goal:** get each salvaged motor's **stall current** so the final motor drivers can be sized correctly (drivers are sized to *stall*, not running). Part of the Neato D10 brain-transplant project — see `neato-d10-handoff.md`.

**Last updated:** 2026-08-04

> **This doc was revised after the teardown photos.** Two things changed: the blower turned out to have its current printed on it *and* to need no driver at all, and the primary method switched from live current measurement to **winding resistance**, which is safer and more accurate on the meter in hand.

---

## What still needs measuring

| Motor | Status |
|---|---|
| Blower/vacuum (EVERFLOW `F121225BU`) | ✅ **Done — 2.0 A printed on the label.** Brushless PWM fan, needs no driver. **Do not measure it** (see below). |
| Roller brush (`905-0460-RoHS`) | ⬜ Measure |
| Drive wheel L (`260-0016`) | ⬜ Measure |
| Drive wheel R (`260-0016`) | ⬜ Measure |

**Why the blower is exempt:** it's brushless with an integrated driver IC. Probing it in Ω mode measures driver electronics, not a winding — the number means nothing. And its rated current is already on the label.

---

## Why resistance instead of live current

A motor's stall/inrush spike lasts tens of milliseconds. The meter here is an **MS8233A: 2000 counts, ~2–3 samples/sec.** It will not catch that transient. Chasing it produces a number that's wrong in an unknown direction.

Winding resistance gives the same answer deterministically:

> **stall current ≈ 14.4 V ÷ R**

At stall the rotor isn't turning, so there's no back-EMF, and the winding is just a resistor across the supply. This is the textbook stall calculation, and it needs **no power applied at all**.

### Meter notes (MS8233A specifically)

- **The 10 A jack IS fused** on this unit — the panel reads `MAX 10A FUSED, MAX 30 sec every 15 min` (confirmed from the meter photo, 2026-08-04). That's a safety margin, not a licence: a fuse can be blown, missing, or the wrong type, and it still only protects against a *sustained* overload, not the initial short-circuit spark across a 6200 mAh Li-ion pack. Still worth avoiding current mode when resistance gives the same answer — but the earlier "genuine unfused short" framing was wrong for this meter.
- **Lowest Ω range is 200** → 0.1 Ω resolution. Motor windings land around 1–5 Ω, so expect ~5–10% error. Fine for sizing a driver; **round up generously**, don't treat it as precise.
- Resistance mode uses the meter's own internal battery. No external supply, no L293D, nothing connected to the motor.

---

## Procedure — winding resistance

**Setup**
1. Red lead in the **`mAVΩ`** jack (NOT the 10 A jack). Black in **COM**.
2. Dial → **Ω, 200 range**.
3. **Short the probes together.** Note the reading — that's lead resistance, typically 0.1–0.4 Ω. Subtract it from every measurement.

**Per motor**
4. Motor **completely disconnected** — unplugged from the harness, not wired to anything.
5. Probe **across the two motor terminals**:
   - **Wheel motors:** the two chunky solder posts flanking the encoder disc, where the can's brush terminals pass through the encoder board. **NOT the harness wires** — those run through encoder circuitry and will give a meaningless reading.
   - **Brush motor:** its two terminals directly.
6. Note the reading. **Rotate the shaft a few degrees by hand and measure again.** Repeat ~6 times through a full rotation — commutator position changes the reading significantly.
7. **Take the LOWEST value**, subtract lead resistance → that's R.
8. `14.4 ÷ R` = stall current.

**Worked example:** lowest reading 3.4 Ω, leads 0.4 Ω → R = 3.0 Ω → stall ≈ 4.8 A.

---

## Optional — live running current

Only worth doing if you want the running figure for power budgeting. It is **not** required for driver sizing.

- Red lead → **10 A jack**, dial → **10 A DC**, set **before** connecting anything.
- **In series** — break the circuit so current flows *through* the meter:
  `[+14.4V] → [red probe | METER | black probe] → [motor +] … [motor −] → [GND]`
- Wheels only may run via the Elegoo **L293D** (≤600 mA/ch). The brush motor: direct from supply through the meter, never through the L293D under load.
- **Read fast, disconnect.** Remember the 30-second limit on that jack.
- **Move the red lead back to `mAVΩ` afterwards.** Leaving it in the 10 A jack and then probing a voltage is the classic way to short a supply through the meter.

---

## Turn numbers into parts

Rule of thumb: **driver continuous rating ≥ measured stall current, with margin.**

| If stall came out… | Driver |
|---|---|
| Wheels ≤ ~1 A each | **TB6612FNG** (dual — one board covers both) ✅ current plan |
| Wheels ~1–3 A each | TB6612 marginal; consider `DRV8871` ×2 |
| Wheels ≥ ~3 A each | **`DRV8871` ×2** or equivalent — TB6612 is out |
| Brush ≤ ~1.5–2 A | MOSFET module, or a BTS7960 for headroom |
| Brush higher | **BTS7960** (rated far above this); just mind wiring and heatsinking |
| Blower | **No driver.** PWM direct from ESP32 GPIO at ~25 kHz. |

Once measured, update the BOM in `neato-d10-brain-transplant.md` and order.

---

## While you're in there — two free data points

Both use continuity/Ω mode, both take a minute:

- **Count the wheel harness wires.** 6 = 2 motor + 4 encoder (quadrature, direction-aware — good). 5 = 2 motor + 3 encoder (single channel, direction-blind — matters for odometry quality and for slam_toolbox). Then use continuity to find which two wires reach the motor solder posts, and **label them now** while it's obvious.
- **Count the blower's wires.** 4 confirms the brushless-PWM conclusion (black GND / red +14.4 V / yellow tach / blue PWM). 2 would mean it's a plain brushed motor after all, and a MOSFET or BTS7960 goes back on the list.

---

## Safety notes

- Resistance measurements are **zero-risk** — nothing is powered. Do these first, and you may not need to power anything at all.
- **Never** measure current in parallel / across the supply — series only.
- **Never** leave the red lead in the 10 A jack when you go back to measuring voltage.
- The raw 14.4 V pack can dump enormous current into a short. Keep leads tidy, connections deliberate.
- Secure any motor before powering it — an unmounted motor can jump when it kicks.

---

## Record your results here

```
Lead resistance (probes shorted): 0.1 Ω   ← subtract from all readings below

                  R lowest (Ω)   minus leads   → stall (14.4/R)
Drive wheel L  :  6.8            6.7           2.1 A    (2026-08-04; stable cluster 6.8–9 Ω via alligator clips, lowest stable taken)
Drive wheel R  :  6.0            5.9           2.4 A    (2026-08-04; steady 6.0–6.7 Ω via clips, lowest stable taken)
Roller brush   :  ______         ______        ______ A
Blower/vacuum  :  n/a — 2.0 A from label, brushless, no driver needed

Wheel harness wire count : ______  (6 = quadrature, 5 = single channel)
Blower wire count        : ______  (4 = PWM fan, 2 = brushed)

Notes:
```
