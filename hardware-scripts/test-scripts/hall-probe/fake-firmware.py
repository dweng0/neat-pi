#!/usr/bin/env python3
"""Fake Neato ESP32 firmware on a pseudo-terminal.

Creates a PTY and symlinks it to /tmp/cu.usbserial-0001 so hall-probe can
open it like a real device (with PORT pointed at /tmp/cu.usbserial-).
Speaks the same dialect as esp32-firmware/src/main.cpp handleCommand():
  Z -> [enc] zeroed        S -> [motor] stop      B -> [motor] brake
  F/R <duty> -> [motor] fwd|rev duty=N
  E -> [enc] A=.. B=.. pos=.. (levels: A=0 B=1) filt=0us supp=0
While "driving", the encoder pos drifts so E replies change over time.
"""

import os
import pty
import sys
import termios
import tty

LINK = "/tmp/cu.usbserial-0001"


def main():
    master, slave = pty.openpty()
    tty.setraw(slave)  # no echo, no line cooking on the device end
    slave_path = os.ttyname(slave)

    if os.path.islink(LINK) or os.path.exists(LINK):
        os.unlink(LINK)
    os.symlink(slave_path, LINK)
    print(f"[fake-firmware] pty {slave_path} -> {LINK}", flush=True)

    a = b = 0
    pos = 0
    duty = 0
    direction = 1

    def send(line):
        os.write(master, (line + "\n").encode())
        print(f"[fake-firmware] >> {line}", flush=True)

    buf = b""
    while True:
        data = os.read(master, 256)
        if not data:
            break
        buf += data
        while b"\n" in buf:
            raw, buf = buf.split(b"\n", 1)
            line = raw.decode(errors="replace").strip()
            if not line:
                continue
            print(f"[fake-firmware] << {line!r}", flush=True)
            c = line[0].upper()
            if c == "Z":
                a = b = pos = 0
                send("[enc] zeroed")
            elif c == "S":
                duty = 0
                send("[motor] stop")
            elif c == "B":
                duty = 0
                send("[motor] brake")
            elif c == "E":
                # spinning motor => edges accumulate between polls
                if duty > 0:
                    step = max(1, duty // 8)
                    a += step
                    b += step
                    pos += step * direction
                send(f"[enc] A={a} B={b} pos={pos} (levels: A=0 B=1) filt=0us supp=0")
            elif c in ("F", "R"):
                direction = 1 if c == "F" else -1
                try:
                    duty = int(line[1:].strip() or "0")
                except ValueError:
                    duty = 0
                send(f"[motor] {'fwd' if c == 'F' else 'rev'} duty={duty}")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(0)
