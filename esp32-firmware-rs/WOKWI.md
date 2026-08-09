# Running esp32-firmware-rs in the Wokwi ESP32 simulator

This project ships a Wokwi configuration so the firmware can run (and be
step-debugged) on a virtual ESP32 with no physical hardware.

Files involved (all in the project root):

- `wokwi.toml` — points Wokwi at the debug ELF, targets the ESP32, and opens a
  GDB server on port 3333.
- `diagram.json` — a virtual ESP32-DevKitC-V4 board with a blue LED on GPIO2
  (through a 220 Ω resistor) to visualise the 1 Hz heartbeat, plus UART0
  (`TX`/`RX`) wired to the Wokwi serial monitor at 115200 baud.

Wokwi loads the ELF directly, so **you must `cargo build` first** — the
simulator reads `target/xtensa-esp32-none-elf/debug/esp32-firmware-rs`.

## 0. Build the firmware (always do this first)

```sh
export PATH="$HOME/.cargo/bin:$PATH"
. "$HOME/export-esp.sh"
cd /Users/jay/projects/Neato/esp32-firmware-rs
cargo build
```

Re-run `cargo build` after every code change, then restart the simulator.

## (a) Run it: Wokwi VS Code extension

1. Install the **Wokwi Simulator** extension from the VS Code marketplace
   (publisher: Wokwi, id `wokwi.wokwi-vscode`).
2. First run only: press `F1` → **Wokwi: Request a new License**, and complete
   the (free) activation in the browser.
3. Open this project folder in VS Code.
4. `cargo build` (see step 0).
5. Press `F1` → **Wokwi: Start Simulator**.

The board appears with the GPIO2 LED blinking at 1 Hz. Click the serial monitor
panel and type — the firmware echoes your input back at 115200 baud.

## (b) Run it: headless CLI (`wokwi-cli`)

### Install

```sh
curl -L https://wokwi.com/ci/install.sh | sh
```

This installs a static binary (on this machine it landed at
`~/bin/wokwi-cli` — make sure that dir is on your `PATH`). Verify:

```sh
wokwi-cli --help
```

### Token

Running a simulation needs a free API token. Create one at
<https://wokwi.com/dashboard/ci>, then export it (a valid token starts with
`wok_` and is 44 chars long):

```sh
export WOKWI_CLI_TOKEN="wok_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

### Run

```sh
cargo build                                     # step 0 above
wokwi-cli /Users/jay/projects/Neato/esp32-firmware-rs
```

The CLI boots the ESP32, streams the serial output to your terminal, and exits
automatically after 30 s. Bound it explicitly with `--timeout 10000` (ms) if you
just want a quick smoke test, or `--interactive` to keep it running and type
into the serial monitor.

### Validate the config without a token (no network sim)

`wokwi-cli lint` checks `wokwi.toml` / `diagram.json` for bad part types and pin
names. It fetches board definitions from the Wokwi registry (so it needs
network) but does **not** need `WOKWI_CLI_TOKEN`:

```sh
wokwi-cli lint /Users/jay/projects/Neato/esp32-firmware-rs
```

> Already verified during setup: lint passes with 0 errors / 0 warnings. It
> prints one informational note that `board-esp32-devkit-c-v4` is an
> "undocumented" registry type — that is expected; it is the standard board
> part and renders correctly.

## (c) Step-debugging in the simulator (no JTAG probe)

`wokwi.toml` sets `gdbServerPort = 3333`. When a simulation is running (via the
VS Code extension **or** `wokwi-cli`), Wokwi exposes a GDB stub on
`localhost:3333` that you can attach an Xtensa GDB / DAP client to. Set
breakpoints, step, and inspect variables against the running virtual chip.

You need the Xtensa GDB that ships with the esp toolchain:
`xtensa-esp32-elf-gdb` (installed by `espup`; it is on your `PATH` after
`. "$HOME/export-esp.sh"`).

### Quick path: raw GDB

In one terminal, start the sim (keep it running):

```sh
wokwi-cli --interactive /Users/jay/projects/Neato/esp32-firmware-rs
```

In another terminal:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
. "$HOME/export-esp.sh"
xtensa-esp32-elf-gdb \
  target/xtensa-esp32-none-elf/debug/esp32-firmware-rs \
  -ex "target remote :3333"
# then: break main / continue / step / info locals ...
```

### VS Code extension path

The extension wires this up for you: after **Wokwi: Start Simulator**, use the
bundled Wokwi debug launch configuration (it attaches to the `gdbServerPort`
automatically). See <https://docs.wokwi.com/vscode/debugging>.

### nvim-dap (suggested config — NOT applied to your setup)

If you drive Neovim with `nvim-dap`, you can attach to the same GDB server.
This is a **suggestion to copy into your own dap config** — nothing here edits
your Neovim setup. It uses the `cpptools`/gdb DAP adapter pointed at the Xtensa
GDB:

```lua
local dap = require("dap")

-- Adapter: cpptools' gdb-backed DAP (install cpptools via Mason).
-- Point miDebuggerPath at the Xtensa GDB from the esp toolchain.
dap.adapters.wokwi_gdb = {
  type = "executable",
  command = vim.fn.stdpath("data")
    .. "/mason/packages/cpptools/extension/debugAdapters/bin/OpenDebugAD7",
}

dap.configurations.rust = dap.configurations.rust or {}
table.insert(dap.configurations.rust, {
  name = "Attach to Wokwi (ESP32 :3333)",
  type = "wokwi_gdb",
  request = "launch",              -- MIMode attach to a remote gdbserver
  MIMode = "gdb",
  miDebuggerPath = "xtensa-esp32-elf-gdb",  -- must be on PATH (export-esp.sh)
  miDebuggerServerAddress = "localhost:3333",
  program = "${workspaceFolder}/target/xtensa-esp32-none-elf/debug/esp32-firmware-rs",
  cwd = "${workspaceFolder}",
  stopAtEntry = false,
})
```

Workflow: `cargo build` → start the sim (`wokwi-cli --interactive .`, leave it
running) → in Neovim run this dap configuration to attach → set breakpoints and
step. Stop the sim with `Ctrl-C` in its terminal when done.
