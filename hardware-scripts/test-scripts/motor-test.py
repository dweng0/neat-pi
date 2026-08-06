#!/usr/bin/env python3
"""
motor-test.py — bench test harness for the Neato D10 brain-transplant ESP32 rig.

Talks to the ESP32 co-processor over USB serial and drives the STEP-1 motor
firmware (esp32-firmware/src/main.cpp). Used to prove the control chain:
    serial command -> ESP32 -> L293D H-bridge -> motor.

The firmware's serial protocol (115200 baud, newline-terminated):
    F <0-255>   forward at PWM duty   (e.g. "F 180")
    R <0-255>   reverse at PWM duty
    S           stop / coast
    B           brake

Usage:
    # Run the standard demo (ramp forward, stop, reverse) — what the blog shows:
    python3 motor-test.py --demo

    # Send one command and watch the reply for a couple of seconds:
    python3 motor-test.py --cmd "F 200"

    # Drop into an interactive prompt (type commands, blank line / Ctrl-C to quit):
    python3 motor-test.py

    # Override the port if the CH340 enumerates elsewhere:
    python3 motor-test.py --port /dev/cu.usbserial-110 --demo

Run it with the project venv so pyserial is available:
    ../../.esp-venv/bin/python3 motor-test.py --demo

Note: opening the port toggles DTR/RTS and reboots the ESP32 — expected. The
script waits ~1.5 s for the banner before sending anything.
"""

import argparse
import sys
import time

try:
    import serial  # pyserial — installed in the project's .esp-venv
except ImportError:
    sys.exit("pyserial not found. Run with the project venv:\n"
             "  ../../.esp-venv/bin/python3 motor-test.py ...")

DEFAULT_PORT = "/dev/cu.usbserial-110"
DEFAULT_BAUD = 115200

# The demo sequence used to confirm full control on the bench:
# (command, seconds to hold). Gentle start so a loose fan blade doesn't fling off.
DEMO = [
    ("F 120", 2.0),   # gentle forward
    ("F 180", 2.0),   # ramp
    ("F 255", 2.5),   # full blast
    ("S",     1.2),   # stop, let it wind down
    ("R 180", 2.0),   # reverse airflow
    ("R 255", 2.5),   # full reverse
    ("S",     1.0),   # stop
]


def open_port(port, baud):
    s = serial.Serial(port, baud, timeout=1)
    time.sleep(1.5)          # let the ESP32 reboot after the port opens
    s.reset_input_buffer()
    return s


def send(s, cmd, hold):
    """Send one command and echo whatever the board replies for `hold` seconds."""
    print(">>>", cmd)
    s.write((cmd + "\n").encode())
    end = time.time() + hold
    while time.time() < end:
        line = s.readline().decode(errors="replace").strip()
        if line:
            print("   ", line)


def run_demo(s):
    for cmd, hold in DEMO:
        send(s, cmd, hold)


def run_interactive(s):
    print("Interactive mode. Commands: F <0-255> | R <0-255> | S | B. "
          "Blank line or Ctrl-C to quit.")
    try:
        while True:
            cmd = input("motor> ").strip()
            if not cmd:
                break
            send(s, cmd, 0.6)
    except (EOFError, KeyboardInterrupt):
        print()
    finally:
        s.write(b"S\n")     # safety: always stop on exit
        time.sleep(0.3)


def main():
    ap = argparse.ArgumentParser(description="Neato D10 ESP32 motor bench test.")
    ap.add_argument("--port", default=DEFAULT_PORT, help="serial port (default %(default)s)")
    ap.add_argument("--baud", type=int, default=DEFAULT_BAUD, help="baud (default %(default)s)")
    ap.add_argument("--demo", action="store_true", help="run the ramp+reverse demo sequence")
    ap.add_argument("--cmd", help="send a single command, e.g. 'F 200'")
    args = ap.parse_args()

    try:
        s = open_port(args.port, args.baud)
    except serial.SerialException as e:
        sys.exit(f"Could not open {args.port}: {e}\n"
                 "Tip: list ports with  ls /dev/cu.usbserial*")

    try:
        if args.demo:
            run_demo(s)
        elif args.cmd:
            send(s, args.cmd, 2.5)
            send(s, "S", 0.8)      # stop after a one-shot command
        else:
            run_interactive(s)
    finally:
        s.close()


if __name__ == "__main__":
    main()
