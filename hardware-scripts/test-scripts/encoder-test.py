#!/usr/bin/env python3
"""
encoder-test.py — bench test harness for the Neato D10 wheel ENCODER.

Talks to the ESP32 co-processor over USB serial and exercises the STEP-2
drive+encoder firmware (esp32-firmware/src/main.cpp). Where motor-test.py just
proves the motor turns, this proves the magnetic quadrature encoder actually
*counts* — the whole point of build step 2 (odometry).

Why this beats a multimeter: the encoder's A/B outputs toggle hundreds of times
a second at full motor speed (the magnet sits on the motor shaft, before the
gearbox). A slow DMM can only average that to a mid-rail blur. The ESP32 counts
every edge in an ISR, so a healthy sensor prints RISING numbers during a spin
and a dead one stays at zero — no probe-balancing, no ambiguity.

Firmware serial protocol (115200 baud, newline-terminated):
    F <0-255>   forward at PWM duty        S   stop / coast
    R <0-255>   reverse at PWM duty         B   brake
    E           print encoder counts once   Z   zero the counts
The firmware also STREAMS "[enc] A=.. B=.. pos=.." (~2/s) while counts change.

Hardware note: A/B are open-drain, idle-high through an internal ~2.4k pull-up
to a 5 V supply (>=4.5 V required). The ESP32 is NOT 5 V tolerant, so each
output needs a single ~3.3k resistor to GND (forms a divider with that internal
2.4k). Blue = A -> GPIO32, Yellow = B -> GPIO33. Tune the resistor so the GPIO
pin idles ~2.8-3.2 V. All grounds common.

Usage (run with the project venv so pyserial is available):
    ../../.esp-venv/bin/python3 encoder-test.py --spin
    ../../.esp-venv/bin/python3 encoder-test.py --spin --duty 200 --secs 6
    ../../.esp-venv/bin/python3 encoder-test.py --watch          # hand-turn mode
    ../../.esp-venv/bin/python3 encoder-test.py --cmd "E"
    ../../.esp-venv/bin/python3 encoder-test.py                  # interactive
    ../../.esp-venv/bin/python3 encoder-test.py --port /dev/cu.usbserial-10 --spin

Note: opening the port toggles DTR/RTS and reboots the ESP32 — expected. The
script waits ~1.5 s for the banner, then zeroes the counts before testing.
"""

import argparse
import glob
import re
import sys
import time

try:
    import serial  # pyserial — installed in the project's .esp-venv
except ImportError:
    sys.exit("pyserial not found. Run with the project venv:\n"
             "  ../../.esp-venv/bin/python3 encoder-test.py ...")

DEFAULT_PORT = "/dev/cu.usbserial-10"
DEFAULT_BAUD = 115200

ENC_RE = re.compile(r"A=(\d+)\s+B=(\d+)\s+pos=(-?\d+)")
LEVELS_RE = re.compile(r"levels: A=(\d)\s+B=(\d)")


def find_port(preferred):
    """Use the preferred port if it exists, else the first usbserial device."""
    candidates = glob.glob("/dev/cu.usbserial*") + glob.glob("/dev/tty.usbserial*")
    if preferred in candidates or __import__("os").path.exists(preferred):
        return preferred
    if candidates:
        print(f"[info] {preferred} not found; using {candidates[0]}")
        return candidates[0]
    return preferred  # let the open fail with a helpful message


def open_port(port, baud):
    s = serial.Serial(port, baud, timeout=1)
    time.sleep(1.5)          # let the ESP32 reboot after the port opens
    s.reset_input_buffer()
    return s


def send(s, cmd, hold, echo=True):
    """Send one command; collect + optionally echo replies for `hold` seconds."""
    if echo:
        print(">>>", cmd)
    s.write((cmd + "\n").encode())
    lines = []
    end = time.time() + hold
    while time.time() < end:
        line = s.readline().decode(errors="replace").strip()
        if line:
            lines.append(line)
            if echo:
                print("   ", line)
    return lines


def last_counts(lines):
    """Pull the most recent (A, B, pos) tuple out of a batch of reply lines."""
    result = None
    for line in lines:
        m = ENC_RE.search(line)
        if m:
            result = (int(m.group(1)), int(m.group(2)), int(m.group(3)))
    return result


def run_spin(s, duty, secs):
    print(f"\n=== encoder spin test: F {duty} for {secs:.0f}s ===")
    send(s, "S", 0.4)
    send(s, "Z", 0.4)                    # zero the counts
    start = last_counts(send(s, "E", 0.6)) or (0, 0, 0)
    print(f"[baseline] A={start[0]} B={start[1]} pos={start[2]}")

    lines = send(s, f"F {duty}", secs)   # drive + stream while spinning
    send(s, "S", 0.8)                    # stop
    end = last_counts(send(s, "E", 0.6)) or start

    da, db = end[0] - start[0], end[1] - start[1]
    print("\n--- VERDICT " + "-" * 40)
    print(f"    A edges: {da:>8}   B edges: {db:>8}   pos: {end[2]}")
    if da > 5 and db > 5:
        print("    ALIVE — both channels toggling. Quadrature encoder works. ")
        print("    (pos moving = direction decode is sane too.)")
    elif da > 5 or db > 5:
        chan = "A (blue)" if da > 5 else "B (yellow)"
        dead = "B (yellow)" if da > 5 else "A (blue)"
        print(f"    PARTIAL — only {chan} counted; {dead} stayed flat.")
        print("    One channel/divider is suspect: check that wiring + resistor,")
        print("    re-seat the GPIO, confirm the GPIO idles ~3 V.")
    else:
        print("    NO COUNTS — did the disc actually spin? If yes, sensor is")
        print("    suspect. Check: 5 V supply >=4.5 V at the encoder, dividers")
        print("    wired to GPIO32/33, GPIO idle ~3 V, all grounds common.")
        print("    Then confirm with a strong (neodymium) magnet on the chip.")
    print("-" * 52)


def run_characterize(s, secs):
    """Full both-directions sensor characterization — the reusable test.

    baseline idle read -> zero -> FWD for `secs` -> read -> zero -> REV for `secs`
    -> read -> verdict. Per-direction A/B edge counts, plus the sign of the
    quadrature `pos` each way (should flip between FWD and REV if BOTH channels
    are live). Point this at any encoder to tell alive/dead + which channels work.
    """
    print(f"\n=== ENCODER CHARACTERIZATION: {secs:.0f}s FWD + {secs:.0f}s REV ===")
    send(s, "S", 0.5)

    # Baseline: are the outputs idling high through the divider? (wiring sanity)
    base_lines = send(s, "E", 0.7)
    lvl = None
    for line in base_lines:
        m = LEVELS_RE.search(line)
        if m:
            lvl = (int(m.group(1)), int(m.group(2)))
    print(f"[baseline] idle levels A={lvl[0] if lvl else '?'} "
          f"B={lvl[1] if lvl else '?'}  (want 1/1 = both pulled high = wiring OK)")

    # Forward run (counts from zero)
    send(s, "Z", 0.4)
    send(s, "F 255", secs)
    send(s, "S", 1.0)
    fwd = last_counts(send(s, "E", 0.7)) or (0, 0, 0)
    print(f"[forward ] A={fwd[0]} B={fwd[1]} pos={fwd[2]}")

    # Reverse run (counts from zero again)
    send(s, "Z", 0.4)
    send(s, "R 255", secs)
    send(s, "S", 1.0)
    rev = last_counts(send(s, "E", 0.7)) or (0, 0, 0)
    print(f"[reverse ] A={rev[0]} B={rev[1]} pos={rev[2]}")

    a_live = fwd[0] > 5 or rev[0] > 5
    b_live = fwd[1] > 5 or rev[1] > 5

    print("\n--- VERDICT " + "-" * 40)
    print(f"    FWD : A={fwd[0]:>6}  B={fwd[1]:>6}  pos={fwd[2]}")
    print(f"    REV : A={rev[0]:>6}  B={rev[1]:>6}  pos={rev[2]}")
    print(f"    channel A (blue)   : {'ALIVE' if a_live else 'no counts'}")
    print(f"    channel B (yellow) : {'ALIVE' if b_live else 'no counts'}")
    if a_live and b_live:
        opposite = (fwd[2] > 0) != (rev[2] > 0) and fwd[2] != 0 and rev[2] != 0
        print(f"    quadrature/direction: {'OK — pos flips sign FWD vs REV' if opposite else 'pos did not clearly invert — check phase'}")
        print("    => FULL QUADRATURE ENCODER WORKING.")
    elif a_live or b_live:
        chan = "A (blue)" if a_live else "B (yellow)"
        dead = "B (yellow)" if a_live else "A (blue)"
        print(f"    => PARTIAL: {chan} works, {dead} silent both directions.")
        print(f"       {dead} is a wiring/divider issue (re-meter its ~3 V idle,")
        print("       re-seat its resistor + GPIO) OR a genuinely dead half.")
    else:
        print("    => NO COUNTS either direction. If the disc was seen spinning,")
        print("       this sensor is suspect — re-meter its pull-ups (brown->blue,")
        print("       brown->yellow) vs the known-good ~2.4k, check idle ~3 V,")
        print("       then a strong-neodymium static test before condemning it.")
    print("-" * 52)


def run_watch(s):
    """Stream counts without driving — for HAND-TURNING the magnet slowly."""
    print("\n=== watch mode: turn the magnet disc BY HAND, watch counts. "
          "Ctrl-C to stop. ===")
    send(s, "S", 0.4)
    send(s, "Z", 0.4)
    try:
        while True:
            line = s.readline().decode(errors="replace").strip()
            if line:
                print("   ", line)
    except KeyboardInterrupt:
        print()
        print(last_counts(send(s, "E", 0.6)) and "final count above.")


def run_interactive(s):
    print("Interactive. Cmds: F <0-255> | R <0-255> | S | B | E | Z. "
          "Blank line or Ctrl-C to quit.")
    try:
        while True:
            cmd = input("enc> ").strip()
            if not cmd:
                break
            send(s, cmd, 0.6)
    except (EOFError, KeyboardInterrupt):
        print()
    finally:
        s.write(b"S\n")     # safety: always stop on exit
        time.sleep(0.3)


def main():
    ap = argparse.ArgumentParser(description="Neato D10 ESP32 wheel-encoder bench test.")
    ap.add_argument("--port", default=DEFAULT_PORT, help="serial port (default %(default)s, autodetected if missing)")
    ap.add_argument("--baud", type=int, default=DEFAULT_BAUD, help="baud (default %(default)s)")
    ap.add_argument("--spin", action="store_true", help="drive the motor and count edges -> ALIVE/DEAD verdict")
    ap.add_argument("--characterize", action="store_true", help="full test: FWD then REV, per-direction A/B counts + verdict (reusable on any encoder)")
    ap.add_argument("--duty", type=int, default=255, help="PWM duty for --spin (default %(default)s)")
    ap.add_argument("--secs", type=float, default=8.0, help="spin/characterize duration per direction in seconds (default %(default)s)")
    ap.add_argument("--watch", action="store_true", help="stream counts without driving (hand-turn the disc)")
    ap.add_argument("--cmd", help="send a single command, e.g. 'E'")
    args = ap.parse_args()

    port = find_port(args.port)
    try:
        s = open_port(port, args.baud)
    except serial.SerialException as e:
        sys.exit(f"Could not open {port}: {e}\n"
                 "Tip: list ports with  ls /dev/cu.usbserial*")

    try:
        if args.characterize:
            run_characterize(s, args.secs)
        elif args.spin:
            run_spin(s, args.duty, args.secs)
        elif args.watch:
            run_watch(s)
        elif args.cmd:
            send(s, args.cmd, 2.5)
        else:
            run_interactive(s)
    finally:
        s.write(b"S\n")     # safety: always stop on exit
        time.sleep(0.2)
        s.close()


if __name__ == "__main__":
    main()
