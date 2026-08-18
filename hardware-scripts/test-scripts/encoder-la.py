#!/usr/bin/env python3
"""
encoder-la.py  —  logic-analyzer capture + quadrature verdict for the Neato wheel encoder.

WHY THIS EXISTS
  The wheel encoders are DYNAMIC (motion-only) differential Hall sensors (A3423-class):
  they respond ONLY to a MOVING magnetic field. Static magnets, slow hand-turns and a
  DC multimeter all fail BY DESIGN, so every earlier test was invalid. The only honest
  test is: raw A/B off the wire while the motor SPINS, watched by something fast.
  That is exactly what the 24 MHz 8-ch logic analyzer is for. This script drives it.

WIRING (test #1 — bypass the ESP32 and the divider entirely)
  CH0 (D0) -> blue   = encoder A   (at the sensor)
  CH1 (D1) -> yellow = encoder B   (at the sensor)
  GND      -> brown  = encoder GND / star ground   (brown is GND!)
  ORANGE   -> +5 V rail  (orange = Vcc — power the encoder here)
  ⚠️ POLARITY (bench-confirmed 2026-08-18; the old teardown note had this BACKWARDS):
     orange = Vcc = +5 V,  brown = GND.  Reverse it and both channels read flat HIGH,
     looks exactly like a dead sensor. Meter orange = +5 V before trusting a flat capture.
  Encoder powered at 5 V (breadboard rig as left). Motor spun by the DRV8871 rig, or
  hand-twisted. The analyzer only WATCHES A/B; it is not in the drive path. These FX2
  clones are 5 V-tolerant, so no divider is needed for this capture.

USAGE
  # real capture (analyzer plugged in, encoder powered, motor SPINNING):
  ./encoder-la.py --secs 3

  # dry-run the whole pipeline with no hardware (synthetic square waves):
  ./encoder-la.py --demo --secs 1

  # re-analyse a capture taken earlier (or in PulseView):
  ./encoder-la.py --replay ~/encoder-spin.sr

OUTPUTS (written next to --out, default ~/encoder-spin.sr)
  <out>.sr    native sigrok session   -> open in PulseView for the interactive view
  <out>.vcd   Value Change Dump       -> open in GTKWave, or any online VCD viewer
  a short ASCII-art preview + a WaveDrom snippet are printed to the terminal

VERDICT
  Counts edges on each channel, runs an x4 quadrature decode (net position + how many
  illegal 00<->11 double-steps), estimates frequency and which channel leads. Then:
    ALIVE   both channels edging, clean quadrature      -> sensors + disc are FINE
    PARTIAL edges on one channel only                   -> wiring/one sensor, not the disc
    DEAD    ~no edges on a real spin                     -> genuinely dead -> order A3423
"""

import argparse
import os
import shutil
import subprocess
import sys

DRIVER = "fx2lafw"
CHANNELS = "D0,D1"  # D0=A(blue), D1=B(yellow)


def die(msg, code=1):
    print(f"\n\033[31m✗ {msg}\033[0m", file=sys.stderr)
    sys.exit(code)


def need_sigrok():
    if not shutil.which("sigrok-cli"):
        die("sigrok-cli not found. Install with:  brew install sigrok-cli libsigrok")


def scan(driver):
    """Return the scan line for the driver, or None if nothing enumerates."""
    out = subprocess.run(
        ["sigrok-cli", "--driver", driver, "--scan"],
        capture_output=True, text=True,
    ).stdout.strip()
    # first line is the header "The following devices were found:"
    lines = [l for l in out.splitlines() if l.strip() and "following devices" not in l]
    return lines[0] if lines else None


def capture(driver, samplerate, secs, out_sr):
    """Capture to a .sr session file. Returns path."""
    cmd = [
        "sigrok-cli", "--driver", driver,
        "--config", f"samplerate={samplerate}",
        "--channels", CHANNELS,
        "--time", str(int(secs * 1000)),
        "-o", out_sr,
    ]
    print(f"  $ {' '.join(cmd)}")
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        die(f"capture failed:\n{r.stderr.strip()}")
    return out_sr


def to_vcd(sr_path, vcd_path):
    with open(vcd_path, "w") as f:
        subprocess.run(["sigrok-cli", "-i", sr_path, "-O", "vcd"], stdout=f, check=False)


def render_waveform(sr_path, samplerate, first_edge, freq, width=120):
    """Draw a decimated A/B square wave scaled to the ACTUAL edge rate.

    sigrok's own ASCII art is 1 char/sample — at 1 MS/s that window is ~120 µs
    and shows flat lines even on a healthy encoder (edges are ms apart). So we
    pick a window of ~10 cycles starting at the first edge, decimate it to
    `width` columns, and draw it ourselves so inline visualisation is honest.
    """
    start = first_edge if first_edge is not None else 0
    if freq and freq > 0:
        window = int(10 * samplerate / freq)          # ~10 electrical cycles
    else:
        window = 20000                                # fallback: 20 ms @ 1 MS/s
    window = max(window, width)
    step = max(1, window // width)

    cols_a, cols_b = [], []
    for i, (a, b) in enumerate(stream_samples(sr_path)):
        if i < start:
            continue
        if i - start >= window:
            break
        if (i - start) % step == 0:
            cols_a.append(a)
            cols_b.append(b)
    if not cols_a:
        print("    (no samples in window)")
        return

    hi, lo = "▔", "▁"
    span_ms = len(cols_a) * step / samplerate * 1000
    print(f"    window: {span_ms:.1f} ms starting at first edge, {len(cols_a)} cols "
          f"({step} samples/col)")
    print("    A(blue) " + "".join(hi if v else lo for v in cols_a))
    print("    B(yell) " + "".join(hi if v else lo for v in cols_b))
    return cols_a, cols_b


def _wavedrom_wave(cols):
    """Compact WaveDrom wave string from decimated levels: level char then '.' to hold."""
    out = []
    prev = None
    for v in cols:
        c = "1" if v else "0"
        out.append(c if c != prev else ".")
        prev = c
    return "".join(out)


def wavedrom_from_cols(cols_a, cols_b):
    if not cols_a:
        return
    wd = ('{ "signal": ['
          f'{{ "name": "A blue", "wave": "{_wavedrom_wave(cols_a)}" }}, '
          f'{{ "name": "B yell", "wave": "{_wavedrom_wave(cols_b)}" }} '
          '], "config": { "hscale": 1 } }')
    print("\n  WaveDrom (paste into https://wavedrom.com/editor.html for a clean figure):")
    print("    " + wd)


def stream_samples(sr_path):
    """Yield (a, b) ints for each sample, streaming CSV so we never hold it all in RAM."""
    p = subprocess.Popen(
        ["sigrok-cli", "-i", sr_path, "-O", "csv"],
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
    )
    for line in p.stdout:
        line = line.strip()
        if not line or line.startswith(";"):
            continue
        parts = line.split(",")
        if len(parts) < 2 or not parts[0].lstrip("-").isdigit():
            continue  # header row like "logic,logic"
        yield int(parts[0]), int(parts[1])
    p.stdout.close()
    p.wait()


# x4 quadrature step table: index = (prev_a<<3|prev_b<<2|a<<1|b), value = +1/-1 fwd/rev, 0 = none, 2 = illegal
_QUAD = {
    0b0000: 0, 0b0001: +1, 0b0011: +1, 0b0010: -1,
    0b0100: -1, 0b0101: 0, 0b0111: +1, 0b0110: 2,
    0b1100: +1, 0b1101: 2, 0b1111: 0, 0b1110: -1,
    0b1000: +1, 0b1001: 2, 0b1011: -1, 0b1010: 0,
    # remaining combos that imply a double-step are illegal
}


def analyse(sr_path, samplerate):
    edges_a = edges_b = 0
    pos = 0
    illegal = 0
    n = 0
    pa = pb = None
    first_a_edge = last_a_edge = None
    first_edge = None  # earliest edge on EITHER channel (where the render window starts)
    for i, (a, b) in enumerate(stream_samples(sr_path)):
        n += 1
        if pa is None:
            pa, pb = a, b
            continue
        if a != pa:
            edges_a += 1
            if first_a_edge is None:
                first_a_edge = i
            last_a_edge = i
        if b != pb:
            edges_b += 1
        if (a != pa or b != pb) and first_edge is None:
            first_edge = i
        if a != pa or b != pb:
            key = (pa << 3) | (pb << 2) | (a << 1) | b
            step = _QUAD.get(key, 2)
            if step == 2:
                illegal += 1
            else:
                pos += step
        pa, pb = a, b

    freq = None
    if edges_a >= 2 and first_a_edge is not None and last_a_edge > first_a_edge:
        # edges_a transitions over that span; 2 edges per electrical cycle
        span_s = (last_a_edge - first_a_edge) / samplerate
        cycles = edges_a / 2.0
        if span_s > 0:
            freq = cycles / span_s
    return {
        "samples": n, "edges_a": edges_a, "edges_b": edges_b,
        "pos": pos, "illegal": illegal, "freq": freq,
        "first_edge": first_edge,
    }


def verdict(r):
    a, b = r["edges_a"], r["edges_b"]
    both = a > 20 and b > 20
    one = (a > 20) ^ (b > 20)
    ratio_ok = both and 0.5 <= (a / b if b else 0) <= 2.0
    clean = r["illegal"] <= max(4, 0.05 * (a + b))
    if both and ratio_ok and clean:
        return "ALIVE", (
            "both channels edging with clean quadrature -> sensors AND disc are FINE. "
            "The method was the problem all along (dynamic Hall needs motion). "
            "Encoder investigation resolved; move to firmware kickstart + odometry."
        )
    if both and not clean:
        return "ALIVE?", (
            "both channels edging but many illegal double-steps -> real signal, but "
            "check sample rate / grabber contact / phase; A and B may be miswired or noisy."
        )
    if one:
        ch = "A(blue,D0)" if a > b else "B(yellow,D1)"
        return "PARTIAL", (
            f"edges on {ch} only -> one channel / one grabber, NOT the disc. "
            "Re-seat the quiet channel's clip and re-run; a single live channel already "
            "proves the disc field is moving past the sensor."
        )
    return "DEAD", (
        "~no edges on a REAL spin (this is the first valid test of that claim). "
        "Sensors genuinely not switching -> order an Allegro A3423 (dynamic dual-channel, "
        "reuses the existing disc). Do NOT buy linear hall (49E/SS49E)."
    )


def main():
    ap = argparse.ArgumentParser(description="Logic-analyzer quadrature verdict for the Neato wheel encoder.")
    ap.add_argument("--secs", type=float, default=3.0, help="capture duration (default 3s)")
    ap.add_argument("--samplerate", default="1m", help="sigrok samplerate (default 1m; edges are ~100-500 Hz so 1 MS/s is huge headroom)")
    ap.add_argument("--out", default=os.path.expanduser("~/encoder-spin.sr"), help="output .sr path")
    ap.add_argument("--demo", action="store_true", help="use the sigrok demo driver (dry-run, no hardware)")
    ap.add_argument("--replay", metavar="FILE.sr", help="skip capture; analyse an existing .sr")
    args = ap.parse_args()

    need_sigrok()
    # samplerate as a number for timing math
    sr_num = args.samplerate.lower().replace("m", "e6").replace("k", "e3")
    try:
        samplerate = float(sr_num)
    except ValueError:
        samplerate = 1e6

    if args.replay:
        sr_path = os.path.expanduser(args.replay)
        if not os.path.exists(sr_path):
            die(f"no such capture: {sr_path}")
        print(f"→ replaying {sr_path}")
    else:
        driver = "demo" if args.demo else DRIVER
        if not args.demo:
            print("→ scanning for the analyzer …")
            found = scan(driver)
            if not found:
                die("no fx2lafw device found. Checklist:\n"
                    "   • analyzer plugged into USB?\n"
                    "   • try:  sigrok-cli --driver fx2lafw --scan\n"
                    "   • firmware auto-loads on connect; if scan is empty, replug\n"
                    "   • dry-run the pipeline meanwhile:  ./encoder-la.py --demo")
            print(f"  found: {found}")
        else:
            print("→ DEMO mode: synthetic square waves, no hardware")
        print(f"→ capturing {args.secs}s @ {args.samplerate} on {CHANNELS} (D0=A/blue, D1=B/yellow) …")
        sr_path = capture(driver, args.samplerate, args.secs, args.out)

    vcd_path = os.path.splitext(sr_path)[0] + ".vcd"
    to_vcd(sr_path, vcd_path)

    print("\n── analysis ──")
    r = analyse(sr_path, samplerate)
    fq = f"{r['freq']:.0f} Hz" if r["freq"] else "n/a"
    print(f"    samples={r['samples']:,}  edges A={r['edges_a']}  edges B={r['edges_b']}  "
          f"net pos={r['pos']:+d}  illegal steps={r['illegal']}  A-freq≈{fq}")

    print("\n── waveform (decimated to the real edge rate) ──")
    cols = render_waveform(sr_path, samplerate, r["first_edge"], r["freq"])

    tag, msg = verdict(r)
    color = {"ALIVE": 32, "ALIVE?": 33, "PARTIAL": 33, "DEAD": 31}.get(tag, 37)
    print(f"\n    \033[{color}m▶ VERDICT: {tag}\033[0m — {msg}")

    if cols:
        wavedrom_from_cols(*cols)

    print(f"\n  files: {sr_path}  (PulseView)   {vcd_path}  (GTKWave)")


if __name__ == "__main__":
    main()
