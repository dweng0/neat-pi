# Hypothesis ledger — wheel encoder investigation

_2026-08-10. Every hypothesis raised during the investigation, the experiment designed to
test it, the observed result, and the verdict. Experiments are reproducible:
`quad-diag.py`, `glitch-sweep.py` (this dir), raw streams in `logs/`._

Legend: ✅ confirmed · ❌ ruled out · ⚠️ superseded (the question stopped making sense) ·
🔧 needs hands (untestable from the desk)

---

## H1 — "The sensor/wiring/GPIO is dead or fried" ❌

**Test:** handheld magnet passed over each sensor, motor off, watching raw counts +
pin levels.
**Result:** clean level flips (1→0) and discrete count increments on both channels
(A first; B after magnet repositioning).
**Verdict:** ruled out. The full path — sensor → wire → divider → GPIO → ISR → count —
works when a strong-enough field moves past it.

## H2 — "GPIO not sensitive enough; try a different pin" ❌

**Test:** none needed — falsified by observation. A pin reading a rock-steady HIGH is
reading *fine*; insensitivity would look like flicker, not silence. Later, H1's magnet
test proved the pin registers real transitions.
**Verdict:** ruled out by reasoning, then by H1's data.

## H3 — "The counter is stuck / doesn't reset (sat at 582)" ❌

**Test:** send `Z`, re-read.
**Result:** 582 → 0 instantly. Counts are firmware-side `volatile` state that survives
host reconnects (host script is stateless); only `Z` or an ESP32 reboot clears them.
**Verdict:** ruled out — behavior, not fault.

## H4 — "Field too weak / air gap too big at the disc" ✅ (confirmed at the END, layer 3)

**Test (early):** stack a booster magnet on the shaft disc → counts appeared during
drives. *Believed confirmation at the time — actually coincided with PWM noise (H6).*
**Test (decisive):** duty 255 = 100 % on = no PWM switching → electrically quiet. Watch
for real rotation edges at full speed and through coast.
**Result:** ~0 edges at constant full speed, 0 through the entire coast. The spinning
disc never trips the sensors; only point-blank handheld magnets ever have.
**Verdict:** **confirmed** — the underlying fault. The disc's field at the sensors is
below trip threshold. 🔧 fix: close the gap / reseat or strengthen the disc magnet /
verify the disc actually carries alternating poles.

## H5 — "Saturated sensor" (user) ❌

**Test:** logical + behavioral. Saturation pins the output — it silences a sensor, it
cannot *create* counts. Checked quiet states for stuck levels.
**Result:** everything is silent at rest and under brake; the flood only appears when
PWM switches; sensors still respond to a handheld magnet afterwards (not damaged).
**Verdict:** ruled out.

## H6 — "Drive-time counts are PWM electrical noise, not rotation" ✅ (layer 1)

**Test:** `quad-diag.py` battery, built on the bench fact that the disc cannot start
below duty ~150 (stiction).
**Result:**
- duty 40/80/120 from rest (disc still): **46–52k edges/s** — impossible as rotation
- F200 (spinning): ~23k/s — *fewer* than not-spinning F40 → counts track PWM, not motion
- coast: edges stop **dead** the instant `S` cuts PWM, while the wheel still spins
- rest & brake: 0 — both non-switching states silent
- phantom rate per channel ≈ **20,000/s ≈ the firmware's `PWM_FREQ`** (fingerprint)
**Verdict:** confirmed. Every "working" drive count in the saga was this.

## H7 — "Missing pull-up between 5 V and the outputs" (user, from wiring diagrams) ❌

**Test:** cross-check adam-meyer schematics against the firmware's wiring notes.
**Result:** those diagrams show the pull-up required by open-collector hall outputs; the
915-1055 encoder board has it **internally** (~2.4k to 5 V). Not missing — but it *is*
half of the 0.4 V-noise-margin divider problem (see H6/H9 mechanism).
**Verdict:** ruled out as stated; folded into the noise-margin mechanism.

## H8 — "The noise is narrow glitches; a software glitch filter will remove it" ❌

**Test 1 (spacing filter):** firmware `G <µs>` = minimum spacing between accepted edges.
Sweep 0/5/20/50/100/200 µs, phases below stiction (pure noise) and above (rotation+noise).
**Result 1:** accepted rate ≈ 1/window at every setting (200 µs → 5,052/s ≈ 5,000/s
predicted). The filter only **downsamples** — noise edges are always waiting when the
window expires. Coherence never improved.
**Test 2 (stability filter):** `G <µs>` reinterpreted = new state must persist X µs.
Sweep 0/1/2/5/10 µs.
**Result 2:** 1–2 µs suppresses almost nothing (dips are *wide*); at 10 µs, F200's noise
dies (~31/s) but F80's passes untouched (23k/s). Dip width ≈ **PWM off-time**
(11 µs at duty 200, 34 µs at duty 80).
**Verdict:** ruled out. The "glitches" are as wide as the PWM off-time — the line's
reference level is following chopped motor current. Windows wide enough to reject them
would burn the CPU in the ISR (and exceed the PCNT hardware filter's ~12.8 µs max).

## H9 — "The coupling is ground/supply bounce through the shared ground wire" ✅ (layer 2)

**Test:** prediction — if the noise rides on motor current through the daisy-chained
ground (firmware wiring note: DRV8871 GND + encoder GND + divider GND share one run),
then at duty 255 (no chopping) it must vanish, and any residue must correlate with
current transients, not speed.
**Result:** at F255 phantom edges collapse **1000×** (48k/s → 43/s); the residue clusters
in the **first second only** (inrush/acceleration current), zero at constant speed, zero
during coast.
**Verdict:** confirmed as the coupling path. 🔧 fix: star grounding (encoder GND straight
to ESP32 GND; motor return straight to the supply), decoupling caps at the encoder board,
route encoder lines away from motor leads. Software cannot fix wiring.

## H10 — "Channel B under-counts / channels are in-phase / ratios are meaningful" ⚠️

**Test:** ratio + coherence across all driven runs.
**Result:** the "ratios" (1.64, 0.6, 1.9…) were ratios of *noise* and flipped between
runs; coherence ≈ 0 everywhere. There was never a real two-channel signal to compare.
**Verdict:** superseded — question dissolves until H4 and H9 are fixed.

## H11 — "The shaft magnet isn't spinning" ⚠️ (partially resolved by user)

**Test:** user observation at the bench.
**Result:** disc visibly spins at duty ≥ ~150 (stiction); does not start below.
**Verdict:** resolved as a bench fact (and it became the control that cracked H6).
Whether the disc's *poles* are intact/strong enough is H4's territory.

## H12 — "The 3.3k 'divider' was actually wired in SERIES" ✅ (found by the user, post-analysis)

**How it surfaced:** while planning the star-ground rewire, the user described the
resistors as "connecting the encoder (blue/yellow) to the ESP32" — i.e. in the signal
path — and explained the intent: "to prevent GPIO32/33 from firing, they're only rated
3.7". That's series-resistor-as-protection thinking; the firmware comments had always
*described* a to-GND divider, but the bench had the series circuit.
**Why it matters:** a GPIO input draws ~no current, so a series resistor drops ~no
voltage — the pins were seeing the full ~5 V (out of spec, survived by luck). Worse for
the noise story: a 3.3k series feed into a no-load pin leaves the pin as a **floating
high-impedance node — an antenna** next to 20 kHz chopped motor wiring. This plausibly
accounts for a large share of the phantom-edge storm (and revises the earlier "0.4 V
divider noise margin" arithmetic, which assumed the divider existed).
**Fix:** blue/yellow straight to GPIO32/33; each 3.3k from the pin node to ESP32 GND
(creates the real divider: idle-high ≈ 2.9 V, low-impedance, in-spec).
**Verdict:** confirmed as a wiring fault; noise contribution to be quantified by
re-running `quad-diag.py` after the rewire (below-stiction phases must read ~0).

---

## Where that leaves the system

| Layer | Status |
|-------|--------|
| Electronics path (sensor→GPIO→count) | ✅ works |
| Drive-time counting | ❌ swamped by ground-bounce noise (H6+H9) — wiring fix needed |
| Rotation sensing at the disc | ❌ zero real signal (H4) — mechanical/magnetic fix needed |
| Software filtering as a fix | ❌ dead end, proven twice (H8) |

**Fix order:** (1) star ground + decoupling (kills H9), (2) close the air gap / verify
disc poles (fixes H4), (3) re-run `quad-diag.py` — below-stiction phases must read 0 and
coast must show decaying real edges before any count is trusted.
