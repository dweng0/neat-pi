# Wheel encoder diagnosis — investigation log

_Started 2026-08-10. Tools: `hall-probe` (this dir), `diag/quad-diag.py`, `esp32-firmware/src/main.cpp` (STEP-2 drive+encoder firmware). Port `/dev/cu.usbserial-10` @ 115200._

## TL;DR — FINAL (2026-08-10, after the glitch-filter + F255 experiments)

Three layers, all now proven by experiment:

1. **All drive-time counts were electrical noise.** ~46–52k phantom edges/s whenever PWM
   switches; the phantom rate per channel ≈ the **20 kHz PWM frequency** (fingerprint).
2. **The noise is ground/supply bounce, not narrow spikes — software cannot filter it.**
   A stability-filter sweep showed the false "dips" are **as wide as the PWM off-time**
   (~11 µs at duty 200, ~34 µs at duty 80): the signal's reference level is moving with
   chopped motor current through the **shared daisy-chained ground**. Software filters
   only downsample it; wide-enough windows would eat all CPU. Fix is wiring: **star
   ground** (encoder GND direct to ESP32 GND; motor return direct to supply), supply
   decoupling at the encoder board.
3. **Under electrically-quiet conditions the real rotation signal is ZERO.** At duty 255
   (100 % on → no switching) phantom noise collapses 1000× (48k/s → ~43/s) — and the
   remaining trickle clusters at **spin-up only** (inrush current transient), reads **0 at
   constant full speed and 0 through the whole coast**. The sensors never register the
   spinning disc. Only handheld magnets (stronger field, point-blank) have ever tripped
   them → the underlying fault is **field strength / air gap at the disc**, exactly where
   the investigation started — but now proven on clean data.

Noise-margin arithmetic behind layer 1–2: encoder board pulls up to 5 V through internal
~2.4k; our ~3.3k-to-GND divider parks idle-high at ≈ **2.9 V** vs the ESP32's ≈ **2.48 V**
high threshold — **0.4 V of margin**, erased by ground bounce every PWM cycle. The ISR
counts every CHANGE with no rejection (original firmware).

## The definitive battery (`diag/quad-diag.py`, 2026-08-10)

Built on one bench fact: **the magnet disc only starts from rest at duty ≥ ~150
("stiction point")** — so edges at duty < 150 from rest cannot be rotation.

| Phase | Condition | dA | dB | edges/s | net pos | Reading |
|-------|-----------|----|----|---------|---------|---------|
| REST | motor off, 8 s | 0 | 0 | 0 | 0 | perfectly quiet ✅ |
| NOISE F40 | PWM on, **disc still** | 51 837 | 63 247 | **46 034** | +3 554 | phantom edges, no rotation |
| NOISE F80 | PWM on, **disc still** | 53 226 | 72 024 | **50 100** | +2 464 | phantom edges |
| NOISE F120 | PWM on, **disc still** | 54 718 | 75 182 | **51 960** | −404 | phantom edges |
| SPIN ×3 | F 200, spinning | ≈31.7k | ≈37.2k | ≈23 000 | ≈−550 | **less** than not-spinning F40 |
| COAST | F 220 → S, wheel spinning down | — | — | 7 425 → **0** within 0.5 s of S | −138 | edges die with PWM, not with motion |
| BRAKE-REST | brake, at rest | 0 | 0 | 0 | 0 | quiet — noise needs *switching* |
| REVERSE | R 200 | 30 491 | 38 665 | 23 052 | −398 | pos does **not** flip sign vs F |

Key observations:

1. Edge rate correlates with **PWM switching being active**, not with rotation or speed.
2. SPIN repeatability is eerily tight (31 607 / 31 496 / 31 949) — a consistent electrical
   source, not flaky contacts.
3. Coherence (|Δpos| / total edges) ≈ **0.003–0.03 in every driven phase** — healthy
   quadrature ≈ 1.0. The edges are near-random, they cancel.
4. Both quiet H-bridge states (coast `S`, brake `B`) are silent.

Raw streams: `diag/logs/quad-diag-20260810-144805.log`.

## Hypotheses — status

Full ledger with per-hypothesis experiments and verdicts: **`diag/HYPOTHESES.md`**.
Summary: ✅ confirmed — PWM phantom edges (H6), ground-bounce coupling path (H9), field
too weak at the disc (H4). ❌ ruled out — dead sensor/wiring (H1), GPIO insensitivity
(H2), stuck counter (H3), saturation (H5), missing pull-up (H7), software glitch
filtering as a fix (H8, proven twice). ⚠️ superseded — B-under-counts/in-phase ratios
(H10: they were ratios of noise), shaft magnet not spinning (H11: resolved as the
stiction bench fact).

## Glitch-filter experiments (`diag/glitch-sweep.py`, firmware `G <µs>` command)

The firmware gained a runtime-settable filter (`G 50` = 50 µs, `G 0` = off/original
behavior; suppressed-edge counter reported in `E` output). Two designs tested:

**Spacing filter** (min gap between accepted edges), swept 0–200 µs with the disc still:
accepted rate ≈ 1/window at every setting (200 µs → 5,052/s vs 5,000 predicted). It only
**downsamples** continuous chatter. Falsified the narrow-glitch model.

**Stability filter** (new state must persist X µs), swept 0–10 µs: 1–2 µs catches almost
nothing; 10 µs kills duty-200 noise (~31/s) but passes duty-80 noise untouched (23k/s).
⇒ dip width ≈ **PWM off-time** (11 µs @ duty 200, 34 µs @ duty 80): the line follows
chopped motor current. Software cannot fix this (needed windows would eat the CPU; PCNT's
hardware filter maxes at ~12.8 µs).

## The duty-255 experiment (decisive, both remaining layers)

Duty 255 = 100 % on = **no PWM switching** = electrically quiet, and the disc definitely
spins (well above stiction).

- Phantom edges collapse **1000×**: ~48,000/s → **43/s** with `G 0`.
- Bucketed over time: the residue sits **only in the first ~1 s** (inrush current
  transient) — **0 edges at constant full speed, 0 through a 3 s coast**.
- ⇒ coupling is current-driven ground/supply bounce (H9 ✅), and the **real rotation
  signal is zero** (H4 ✅): the spinning disc never trips the sensors.

## Fix plan (revised after the experiments — software options are dead, H8)

1. **Star ground** (fixes H9): encoder GND straight to ESP32 GND; motor return current
   straight to the supply — never daisy-chained through the signal ground. Plus 100 nF +
   ~10 µF decoupling at the encoder board's 5 V, and route encoder lines away from motor
   leads (twist A/B with their ground).
2. **Field/gap at the disc** (fixes H4): close the sensor-to-disc air gap, reseat or
   strengthen the disc magnet, verify the disc actually carries alternating poles.
3. **Noise margin** (hardening): the 2.9 V idle-high vs 2.48 V threshold deserves a
   proper level shifter / comparator / Schmitt buffer regardless.
4. **Re-verify** with `diag/quad-diag.py` after each change: below-stiction phases must
   read **0**, and coast must show **decaying real edges**, before any count is trusted.
5. Real-edge sanity number: motor at a few kRPM with a simple pole disc ⇒ expect
   **hundreds to low-thousands of edges/s** — anything ≫ that is noise.

## Observation history (chronological, superseded readings included)

| Test | A | B | pos | Reading at the time |
|------|---|---|-----|---------------------|
| Driven `F 150`, no boost | frozen at 62 | 0 | 0 | levels A=1 B=1, never toggled |
| `Z` then `F 150`, no boost | stayed 0 | 0 | 0 | zero edges during a real 4 s drive |
| Speed 0 + handheld magnet at sensor | 62 → 120, level 1→0 ×5 | 0 | ticked to 1 | **genuine edges** — sensor path proven |
| Driving with booster magnet on shaft | 325k → 327k | 115k → 116k | ≈−8500 | *(believed rotation; actually noise)* |
| `Z` + `F 150`, booster on, 4 s | +41 909 | +25 630 | −7 545 | *(believed rotation; actually noise)* |
| `F 0`, booster off | frozen | frozen | frozen | stable at rest ✅ |
| `F 150`, booster off | → 3 529 | → 6 695 | random-walk | *(believed rotation; actually noise)* |
| Duty sweep + coherence battery | — | — | — | coherence ≈ 0 at all duties — first hard noise evidence |
| **quad-diag battery** | — | — | — | **definitive: noise-only, see above** |

Notes for the record:
- Counts are firmware-side `volatile` state — they survive host reconnects; reset only on
  `Z` or ESP32 reboot (explains "sat at 582 across script restarts").
- The encoder magnet sits on the **motor shaft, before the gearbox**.
- Research: hall sensors have a **max-Gauss rating** (a too-strong magnet too close can
  damage them — the booster-stack was a risk, but magnet tests show the sensors survived);
  latching types (US1881-style) need an **opposing pole** to unlatch, so a single handheld
  pole flips them once and they hold — matches the level-flip behaviour we saw.

## Next steps

1. **Fix the noise first** — nothing about the encoder can be judged until the lines are
   clean. Cheapest experiment: firmware glitch filter / PCNT; then RC or Schmitt in hardware.
2. Re-run `diag/quad-diag.py` after each change — REST and NOISE-F40/80/120 phases must
   read **0** before drive-time counts mean anything.
3. Then re-evaluate air gap / field strength / A-B phase with real signals.

## Firmware command reference

`F <0-255>` fwd · `R <0-255>` rev · `S` stop · `B` brake · `E` read encoder once ·
`Z` zero the counts. Firmware streams `[enc] …` (~2/s) while counts change.
Port is **single-owner** — close any running `hall-probe` / serial monitor first.
