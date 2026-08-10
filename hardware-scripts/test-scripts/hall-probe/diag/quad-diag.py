#!/usr/bin/env python3
"""
quad-diag.py — hypothesis-driven diagnostic battery for the Neato wheel encoder.

Run with the repo venv (has pyserial):  ../../../../.esp-venv/bin/python quad-diag.py

Ground rules from the bench (2026-08-10):
  * The magnet disc only starts spinning from rest at duty >= ~150 ("stiction point").
  * Therefore: any edges counted at duty < 150 (from rest) CANNOT be real rotation.
  * Zero edges must appear when nothing moves; edges must appear when it spins.

Phases (each phase zeroes the counters first with 'Z'):
  1 REST        motor off 8 s                    -> expect dA=dB=0 (baseline)
  2 NOISE-SWEEP duty 40/80/120 from rest, 2.5 s  -> disc not moving; edges = PWM noise,
                                                    and rate-vs-duty shows noise scaling
  3 SPIN x3     F 200 for 3 s, three times       -> repeatability of dA/dB/ratio/coherence
  4 COAST       F 220 2 s then S, poll 3 s more  -> real edges should DECAY with spin-down
  5 BRAKE-REST  B (brake) then rest 5 s          -> noise under brake vs coast H-bridge state
  6 REVERSE     R 200 for 3 s                    -> pos must flip sign vs phase 3 if
                                                    quadrature decode works

Metrics per phase:
  dA/dB        raw edge deltas               ratio     dA/dB (healthy quadrature ~1.0)
  net_pos      quadrature position change    coherence |net_pos| / (dA+dB): ~1.0 clean,
  rev          pos direction reversals                  ~0 edges cancel (noise / in-phase)
  rate         edges per second

Raw serial stream is logged to logs/quad-diag-<timestamp>.log for later inspection.
"""
import re
import sys
import time
from pathlib import Path

import serial

import glob
import os
# The CH340 re-enumerates with a new number after replug — autodetect, allow override.
_found = sorted(glob.glob("/dev/cu.usbserial-*"))
PORT = os.environ.get("HALL_PORT", _found[0] if _found else "/dev/cu.usbserial-10")
BAUD = 115200
STAMP = time.strftime("%Y%m%d-%H%M%S")
LOG = Path(__file__).parent / "logs" / f"quad-diag-{STAMP}.log"
ENC_RE = re.compile(r"A=(\d+) B=(\d+) pos=(-?\d+)")

log_fh = open(LOG, "w")


def open_port():
    try:
        s = serial.Serial(PORT, BAUD, timeout=0.15)
    except Exception as e:
        print(f"OPEN_FAILED: {e}")
        sys.exit(1)
    time.sleep(0.3)
    s.reset_input_buffer()
    return s


def phase(s, name, cmd, active_secs, tail_secs=0.0, tail_cmd=None):
    """Zero counters, run cmd for active_secs (polling E), optionally send
    tail_cmd and keep polling tail_secs more. Returns list of (A,B,pos,t)."""
    log_fh.write(f"\n===== PHASE {name}: cmd={cmd!r} active={active_secs}s tail_cmd={tail_cmd!r} tail={tail_secs}s =====\n")
    s.reset_input_buffer()
    s.write(b"Z\n")
    time.sleep(0.25)
    s.reset_input_buffer()
    t0 = time.time()
    if cmd:
        s.write(cmd.encode() + b"\n")
    samples = []
    raw = b""

    def poll_until(deadline):
        nonlocal raw
        last_e = 0.0
        while time.time() < deadline:
            if time.time() - last_e > 0.12:
                s.write(b"E\n")
                last_e = time.time()
            chunk = s.read(256)
            if chunk:
                raw += chunk
                for m in ENC_RE.finditer(chunk.decode(errors="replace")):
                    samples.append((int(m[1]), int(m[2]), int(m[3]), time.time() - t0))

    poll_until(t0 + active_secs)
    if tail_cmd:
        s.write(tail_cmd.encode() + b"\n")
        poll_until(time.time() + tail_secs)
    s.write(b"S\n")
    time.sleep(0.25)
    raw += s.read(400)
    log_fh.write(raw.decode(errors="replace"))
    log_fh.flush()
    return samples


def analyze(name, sm, secs):
    if len(sm) < 2:
        print(f"{name:12s}: <2 samples ({len(sm)})")
        return None
    A = [x[0] for x in sm]
    B = [x[1] for x in sm]
    P = [x[2] for x in sm]
    dA, dB, net = A[-1] - A[0], B[-1] - B[0], P[-1] - P[0]
    tot = dA + dB
    steps = [P[i + 1] - P[i] for i in range(len(P) - 1) if P[i + 1] != P[i]]
    rev = sum(1 for i in range(len(steps) - 1) if steps[i] * steps[i + 1] < 0)
    coh = abs(net) / tot if tot else 0.0
    ratio = dA / dB if dB else float("inf")
    rate = tot / secs
    print(f"{name:12s}: dA={dA:6d} dB={dB:6d} ratio={ratio:5.2f} net_pos={net:6d} "
          f"pos[{min(P):6d},{max(P):6d}] rev={rev:3d} coh={coh:.3f} rate={rate:7.0f}/s")
    return dict(name=name, dA=dA, dB=dB, ratio=ratio, net=net, rev=rev, coh=coh, rate=rate)


def coast_profile(sm, split_t):
    """Edge totals per half-second bucket after split_t (S sent) — decay check."""
    buckets = {}
    prev = None
    for a, b, p, t in sm:
        if prev is not None and t >= split_t:
            k = int((t - split_t) / 0.5)
            buckets[k] = buckets.get(k, 0) + (a - prev[0]) + (b - prev[1])
        prev = (a, b)
    return [buckets.get(k, 0) for k in range(sorted(buckets)[-1] + 1)] if buckets else []


s = open_port()
results = []
print(f"log: {LOG}")
print("=== quad-diag battery ===")

results.append(analyze("1 REST", phase(s, "REST", None, 8.0), 8.0))
for duty in (40, 80, 120):
    results.append(analyze(f"2 NOISE-F{duty}", phase(s, f"NOISE-F{duty}", f"F {duty}", 2.5), 2.5))
for i in (1, 2, 3):
    results.append(analyze(f"3 SPIN-{i}", phase(s, f"SPIN-{i}", "F 200", 3.0), 3.0))
coast_sm = phase(s, "COAST", "F 220", 2.0, tail_secs=3.0, tail_cmd="S")
results.append(analyze("4 COAST", coast_sm, 5.0))
print(f"{'':12s}  coast decay (edges per 0.5s after S): {coast_profile(coast_sm, 2.0)}")
results.append(analyze("5 BRAKE-REST", phase(s, "BRAKE-REST", "B", 5.0), 5.0))
results.append(analyze("6 REVERSE", phase(s, "REVERSE", "R 200", 3.0), 3.0))

s.write(b"S\n")
s.close()
log_fh.close()
print("=== done — motor stopped ===")
