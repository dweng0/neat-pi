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

## Finding the serial port

The ESP32's CH340C usually enumerates as `/dev/cu.usbserial-*` on macOS:

```sh
ls /dev/cu.usbserial*
```

Pass it with `--port` if it isn't the default baked into the script.
