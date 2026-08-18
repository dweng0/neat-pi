# hardware-scripts / test-scripts

Small host-side scripts for bench-testing the Neato D10 transplant hardware
over USB serial. These run on the **Mac/host**, not on the microcontroller —
the firmware they talk to lives in `esp32-firmware/`.

They depend on `pyserial`, which is installed in the project venv at
`.esp-venv/`. Run them with that interpreter:

```sh
cd hardware-scripts/test-scripts
../../.esp-venv/bin/python3 motor-test.py --demo
```

## Scripts

| Script | What it does |
|---|---|
| `motor-test.py` | Drives the STEP-1 motor firmware (ESP32 + L293D H-bridge). Run `--demo` for the ramp-forward/reverse sequence, `--cmd "F 200"` for a one-shot, or no args for an interactive prompt. |
| `encoder-test.py` | Tests the STEP-2 wheel **encoder** (ESP32 counts A/B edges in an ISR). `--spin` drives the motor and prints an ALIVE/PARTIAL/DEAD verdict from the edge counts; `--watch` streams counts while you hand-turn the magnet disc; `--cmd "E"` reads counts once; no args = interactive. Needs the drive+encoder firmware flashed, A/B on GPIO32/33 via ~3.3k-to-GND dividers (5 V → ~3 V), all grounds common. |
| `encoder-la.py` | **The real encoder decider** — captures raw A/B off the wire with the 24 MHz logic analyzer while the motor SPINS (the only valid test for these motion-only Hall sensors). No ESP32, no divider, no firmware in the loop. Runs an x4 quadrature decode and prints an ALIVE/PARTIAL/DEAD verdict + edge counts + net position + frequency. `--secs N` capture length; `--demo` dry-runs the whole pipeline with no hardware; `--replay FILE.sr` re-analyses an old capture. Writes `.sr` (PulseView) and `.vcd` (GTKWave). Uses `sigrok-cli`, **not** the venv. |

### `encoder-la.py` — wiring

Bypass the ESP32 and the divider entirely; these FX2 clones are 5 V-tolerant:

- **CH0 (D0)** → blue = encoder **A** · **CH1 (D1)** → yellow = encoder **B**
- **analyzer GND** → **brown** = encoder ground · **orange** → **+5 V rail** (orange = Vcc)
- ⚠️ **Polarity (bench-confirmed 2026-08-18; old teardown note was backwards): orange = Vcc = +5 V, brown = GND.** Reverse it → both channels flat HIGH, looks dead. Meter orange = +5 V before trusting a flat capture.
- Encoder powered at 5 V (breadboard rig as left); motor spun by the DRV8871 rig, or hand-twisted.
- Depends on `sigrok-cli`: `brew install sigrok-cli libsigrok` (already installed).

## Visualising captures

`encoder-la.py` prints an ASCII-art preview and a WaveDrom snippet inline, and writes two files:

- **`.sr`** → **PulseView**, the full interactive viewer (zoom/pan/measure). Not in Homebrew anymore — grab the macOS DMG from <https://sigrok.org/wiki/Downloads>.
- **`.vcd`** → **GTKWave**: `brew install --cask gtkwave`, then `open -a gtkwave ~/encoder-spin.vcd`.
- **WaveDrom** → paste the printed snippet into <https://wavedrom.com/editor.html> for a clean, publication-ready square-wave figure (good for the blog).

## Finding the serial port

The ESP32's CH340C usually enumerates as `/dev/cu.usbserial-*` on macOS:

```sh
ls /dev/cu.usbserial*
```

Pass it with `--port` if it isn't the default baked into the script.
