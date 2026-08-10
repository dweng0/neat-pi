#!/usr/bin/env python3
"""
glitch-sweep.py — decisive test of the PWM-phantom-edge hypothesis.

Requires the glitch-filter firmware (main.cpp with the 'G <us>' command).
Run with the repo venv:  ../../../../.esp-venv/bin/python glitch-sweep.py

What it proves: quad-diag.py showed ~1 phantom edge per 20 kHz PWM cycle per
channel whenever PWM switches. PWM noise pulses are NARROW (sub-50 µs); real
encoder edges are 100s of µs apart even at speed. So sweeping a minimum-edge-
spacing filter should show:

  * accepted edges at duty 80 (below stiction — disc STILL) collapse toward 0
    once the filter passes the noise pulse width,
  * the firmware's 'supp' counter absorbs them instead (proof they were there),
  * at F 200 (real rotation) a real, much smaller edge rate survives — and if
    the surviving signal is clean quadrature, |pos| coherence jumps toward 1.

For each filter value (0 = today's unfiltered behavior, then 5/20/50/100/200 µs):
  NOISE phase: F 80 from rest, 2.5 s  -> disc not moving, all edges are noise
  SPIN  phase: F 200,          2.5 s  -> real rotation on top of the noise

Output columns: acc/s = accepted edges per second, supp/s = suppressed,
coh = |Δpos|/(ΔA+ΔB) (≈1 clean quadrature, ≈0 noise).
Raw serial streams land in logs/glitch-sweep-<timestamp>.log.
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
LOG = Path(__file__).parent / "logs" / f"glitch-sweep-{STAMP}.log"
ENC_RE = re.compile(r"A=(\d+) B=(\d+) pos=(-?\d+).*?supp=(\d+)")

log_fh = open(LOG, "w")

try:
    s = serial.Serial(PORT, BAUD, timeout=0.15)
except Exception as e:
    print(f"OPEN_FAILED: {e}")
    sys.exit(1)
time.sleep(0.3)
s.reset_input_buffer()


def command(cmd):
    s.write(cmd.encode() + b"\n")
    time.sleep(0.25)
    out = s.read(400).decode(errors="replace")
    log_fh.write(f">>> {cmd}\n{out}")
    return out


def run_phase(label, cmd, secs):
    s.reset_input_buffer()
    command("Z")
    s.reset_input_buffer()
    s.write(cmd.encode() + b"\n")
    t0, raw, samples, last_e = time.time(), b"", [], 0.0
    while time.time() < t0 + secs:
        if time.time() - last_e > 0.12:
            s.write(b"E\n")
            last_e = time.time()
        chunk = s.read(256)
        if chunk:
            raw += chunk
            for m in ENC_RE.finditer(chunk.decode(errors="replace")):
                samples.append(tuple(int(g) for g in m.groups()))
    command("S")
    raw += s.read(400)
    log_fh.write(f"\n===== {label}: {cmd} {secs}s =====\n" + raw.decode(errors="replace"))
    log_fh.flush()
    if len(samples) < 2:
        return None
    A = [x[0] for x in samples]; B = [x[1] for x in samples]
    P = [x[2] for x in samples]; SUP = [x[3] for x in samples]
    dA, dB, net, dS = A[-1] - A[0], B[-1] - B[0], P[-1] - P[0], SUP[-1] - SUP[0]
    tot = dA + dB
    return dict(acc=tot / secs, supp=dS / secs, coh=abs(net) / tot if tot else 0.0,
                dA=dA, dB=dB, net=net)


SWEEP = [int(v) for v in sys.argv[1:]] or [0, 5, 20, 50, 100, 200]

print(f"log: {LOG}")
print(f"{'filter':>7} | {'phase':<5} | {'acc/s':>8} | {'supp/s':>8} | {'coh':>6} | {'dA':>7} {'dB':>7} {'net_pos':>8}")
print("-" * 72)
for g_us in SWEEP:
    command(f"G {g_us}")
    for label, cmd in (("NOISE", "F 80"), ("SPIN", "F 200")):
        r = run_phase(f"G{g_us}-{label}", cmd, 2.5)
        if r:
            print(f"{g_us:>5}us | {label:<5} | {r['acc']:>8.0f} | {r['supp']:>8.0f} | "
                  f"{r['coh']:>6.3f} | {r['dA']:>7d} {r['dB']:>7d} {r['net']:>8d}")
        else:
            print(f"{g_us:>5}us | {label:<5} | <2 samples>")
        time.sleep(1.5)  # let the disc come fully to rest before the next phase

command("G 0")   # leave firmware in original (unfiltered) behavior
command("S")
s.close()
log_fh.close()
print("=== done — motor stopped, filter reset to 0 ===")
