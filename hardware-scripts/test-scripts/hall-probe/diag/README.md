# diag/ — encoder investigation test scripts

Standalone diagnostic scripts for the wheel-encoder investigation (see
`../ENCODER-DIAGNOSIS.md` for the running findings). These are deliberately
separate from `hall-probe` itself — that stays a learn-Rust artifact; these
exist to capture specific evidence, and each notes what it's trying to prove.

Run with the repo venv (the system python has no pyserial):

    ../../../../.esp-venv/bin/python quad-diag.py

Port is single-owner — close any running `hall-probe` / serial monitor first.

## Scripts

### quad-diag.py

Hypothesis battery built around one bench fact: **the magnet disc only starts
from rest at duty ≥ ~150 (stiction)**. Therefore edges at duty < 150 from rest
cannot be rotation — they're electrical. Phases:

| # | Phase       | What it captures                                                  |
|---|-------------|-------------------------------------------------------------------|
| 1 | REST        | 8 s motor-off baseline — a healthy encoder counts **zero**        |
| 2 | NOISE-SWEEP | duty 40/80/120 from rest — disc still ⇒ any edges are PWM noise; rate-vs-duty shows scaling |
| 3 | SPIN ×3     | F 200 3 s, three runs — repeatability of dA/dB/ratio/coherence    |
| 4 | COAST       | F 220 then S, keep polling — real edges must **decay** with spin-down |
| 5 | BRAKE-REST  | brake (B) at rest — noise under the other H-bridge state          |
| 6 | REVERSE     | R 200 — `pos` must flip sign vs phase 3 if quadrature decode works |

Metrics: `ratio` = dA/dB (healthy ≈ 1.0), `coherence` = |Δpos| / (dA+dB)
(≈ 1.0 clean quadrature, ≈ 0 edges cancel ⇒ noise or in-phase channels),
`rev` = direction reversals in pos, `rate` = edges/s.

### glitch-sweep.py

Tests whether a firmware glitch filter can remove the phantom edges. Needs the
`G <µs>` firmware command (added 2026-08-10; `G 0` = original behavior). For each
filter value it runs a below-stiction NOISE phase (F 80, disc still — every edge
is noise) and a SPIN phase (F 200), reporting accepted vs suppressed edge rates
and coherence. Filter values come from argv: `glitch-sweep.py 0 1 2 5 10`.

Answer it produced: **no** — the dips are as wide as the PWM off-time (ground
bounce), so spacing filters merely downsample and stability filters only work at
high duty. See `HYPOTHESES.md` (H8, H9).

### HYPOTHESES.md

The investigation's hypothesis ledger: every hypothesis raised, the experiment
designed to test it, the result, and the verdict (✅/❌/⚠️/🔧).

Raw serial streams are kept in `logs/` for later re-analysis.
