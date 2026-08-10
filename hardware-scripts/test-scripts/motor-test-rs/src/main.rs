//! motor-test-rs — the Rust port of motor-test.py.
//!
//! Same job, same wire protocol: talk to the ESP32 co-processor over USB serial
//! and drive the STEP-1 motor firmware. This exists so you can compare the two
//! line by line and see how the Python idioms map onto Rust ones.
//!
//! The firmware's serial protocol (115200 baud, newline-terminated):
//!     F <0-255>   forward at PWM duty   (e.g. "F 180")
//!     R <0-255>   reverse at PWM duty
//!     S           stop / coast
//!     B           brake
//!
//! Usage (from this directory):
//!     cargo run -- --demo                       # ramp forward, stop, reverse
//!     cargo run -- --cmd "F 200"                # one-shot, then auto-stop
//!     cargo run                                 # interactive prompt
//!     cargo run -- --port /dev/cu.usbserial-110 --demo
//!
//! Note: opening the port toggles DTR/RTS and reboots the ESP32 — expected.
//! We wait ~1.5 s for the banner before sending anything, same as the Python.

use std::error::Error;
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

use clap::Parser;
use serialport::{ClearBuffer, SerialPort};

const DEFAULT_PORT: &str = "/dev/cu.usbserial-110";
const DEFAULT_BAUD: u32 = 115_200;

/// The demo sequence used to confirm full control on the bench:
/// (command, how long to hold it). Gentle start so a loose fan blade doesn't
/// fling off. This is the Rust version of the Python `DEMO` list — an array of
/// tuples. `&str` is a borrowed string slice (the text lives in the binary),
/// which is all we need for these constants.
const DEMO: &[(&str, Duration)] = &[
    ("F 120", Duration::from_millis(2000)), // gentle forward
    ("F 180", Duration::from_millis(2000)), // ramp
    ("F 255", Duration::from_millis(2500)), // full blast
    ("S", Duration::from_millis(1200)),     // stop, let it wind down
    ("R 180", Duration::from_millis(2000)), // reverse airflow
    ("R 255", Duration::from_millis(2500)), // full reverse
    ("S", Duration::from_millis(1000)),     // stop
];

/// Command-line arguments. `clap`'s derive macro turns this struct into a full
/// parser (with `--help`) at compile time — the equivalent of the argparse
/// setup in `main()` of the Python version.
#[derive(Parser)]
#[command(about = "Neato D10 ESP32 motor bench test (Rust port of motor-test.py).")]
struct Args {
    /// Serial port to open.
    #[arg(long, default_value = DEFAULT_PORT)]
    port: String,

    /// Baud rate.
    #[arg(long, default_value_t = DEFAULT_BAUD)]
    baud: u32,

    /// Run the ramp+reverse demo sequence.
    #[arg(long)]
    demo: bool,

    /// Send a single command, e.g. "F 200", then stop.
    #[arg(long)]
    cmd: Option<String>,
}

/// Open the port and wait for the ESP32 to reboot.
///
/// Returns `Box<dyn SerialPort>` — a trait object, i.e. "some type that
/// implements the SerialPort trait, boxed on the heap." That's what
/// `serialport::open()` hands back so it can work across OSes.
fn open_port(port: &str, baud: u32) -> Result<Box<dyn SerialPort>, Box<dyn Error>> {
    // The `?` operator: if `open()` returns an error, return it from this
    // function immediately. This is Rust's version of "let the exception
    // propagate" — no try/except needed, but the error type is checked.
    let port = serialport::new(port, baud)
        // A short per-read timeout. Reads return a TimedOut error when no bytes
        // arrive in this window; `pump_replies` treats that as "nothing yet,
        // keep waiting" rather than a real failure.
        .timeout(Duration::from_millis(100))
        .open()?;

    // Opening the port asserted DTR and rebooted the board; give it time to
    // come up and print its banner before we clear the buffer.
    std::thread::sleep(Duration::from_millis(1500));
    port.clear(ClearBuffer::Input)?; // discard the boot banner / noise

    Ok(port)
}

/// Send one command, then print whatever the board replies for `hold`.
///
/// `&mut dyn SerialPort` is a mutable borrow of the port: we hand `send` access
/// to the same port `main` owns, without giving away ownership, so `main` can
/// keep using it afterwards.
fn send(port: &mut dyn SerialPort, cmd: &str, hold: Duration) -> Result<(), Box<dyn Error>> {
    println!(">>> {cmd}");

    // Commands are newline-terminated. `format!` builds the string; `.as_bytes()`
    // gives the raw bytes to write over the wire.
    port.write_all(format!("{cmd}\n").as_bytes())?;
    port.flush()?;

    pump_replies(port, hold)?;
    Ok(())
}

/// Read and print any complete lines the board sends us for `duration`.
///
/// This is the fiddly bit the Python got for free from `serial.readline()`. We
/// read a byte at a time, buffer until we hit a newline, then print the line —
/// mirroring how the firmware itself assembles lines. A TimedOut read just
/// means "no data this instant," so we keep looping until the deadline.
fn pump_replies(port: &mut dyn SerialPort, duration: Duration) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + duration;
    let mut line = String::new();
    let mut byte = [0u8; 1]; // a one-byte read buffer

    while Instant::now() < deadline {
        match port.read(&mut byte) {
            Ok(0) => break, // port closed / EOF
            Ok(_) => {
                let c = byte[0];
                if c == b'\n' {
                    let trimmed = line.trim_end(); // drop trailing '\r'
                    if !trimmed.is_empty() {
                        println!("    {trimmed}");
                    }
                    line.clear();
                } else {
                    line.push(c as char);
                }
            }
            // The expected "no bytes right now" case — not a real error.
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
            // Anything else is a genuine failure; propagate it.
            Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(())
}

/// Run the fixed demo table.
fn run_demo(port: &mut dyn SerialPort) -> Result<(), Box<dyn Error>> {
    // `for (cmd, hold) in DEMO` destructures each tuple as we iterate. `*hold`
    // dereferences the `&Duration` we borrowed from the array (Duration is
    // Copy, so this is a cheap value copy).
    for (cmd, hold) in DEMO {
        send(port, cmd, *hold)?;
    }
    Ok(())
}

/// Interactive prompt: type commands, blank line to quit.
fn run_interactive(port: &mut dyn SerialPort) -> Result<(), Box<dyn Error>> {
    println!(
        "Interactive mode. Commands: F <0-255> | R <0-255> | S | B. \
         Blank line to quit."
    );

    let stdin = io::stdin();
    loop {
        print!("motor> ");
        io::stdout().flush()?; // prompt has no newline, so force it out

        let mut input = String::new();
        // read_line returns Ok(0) at end-of-input (Ctrl-D).
        if stdin.lock().read_line(&mut input)? == 0 {
            break;
        }

        let cmd = input.trim();
        if cmd.is_empty() {
            break;
        }
        send(port, cmd, Duration::from_millis(600))?;
    }

    // Safety: always stop on a clean exit.
    // (Heads-up: Ctrl-C kills the process before this runs, unlike the Python's
    //  `finally`. Use a blank line to quit, or the --demo/--cmd paths which
    //  already end with a stop. We can wire up a Ctrl-C handler later if you
    //  want the belt-and-braces version.)
    port.write_all(b"S\n")?;
    port.flush()?;
    std::thread::sleep(Duration::from_millis(300));
    Ok(())
}

/// `main` returning `Result` lets us use `?` throughout: any error bubbles up
/// here and Rust prints it and exits non-zero. That replaces the Python's
/// scattered `sys.exit(...)` calls.
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut port = match open_port(&args.port, args.baud) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Could not open {}: {e}", args.port);
            eprintln!("Tip: list ports with  ls /dev/cu.usbserial*");
            std::process::exit(1);
        }
    };

    if args.demo {
        run_demo(&mut *port)?;
    } else if let Some(cmd) = args.cmd.as_deref() {
        send(&mut *port, cmd, Duration::from_millis(2500))?;
        send(&mut *port, "S", Duration::from_millis(800))?; // stop after one-shot
    } else {
        run_interactive(&mut *port)?;
    }

    Ok(())
}
