// hall-probe — a learn-Rust-from-scratch bench tool for the Neato ESP32.

use protocol::command::{Command, get_command_action};
use protocol::serial::LineReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const PORT: &str = "/dev/cu.usbserial-"; // real hardware; use /tmp/cu.usbserial- for the fake-firmware.py pty

const BAUD: u32 = 115_200;
const READ_TIMEOUT_MS: u64 = 100;

fn bail<T>(msg: String) -> T {
    eprintln!("{msg}");
    std::process::exit(2)
}

fn main() {
    // Collect args from the user....
    let args: Vec<String> = std::env::args().collect();

    // Gate on it, check we actually have args
    if args.len() != 4 {
        bail::<()>("Usage: hall-probe <serial-number> <F|R> <speed>".to_string());
    }

    // grab serial
    let serial_number = &args[1];

    // grab direction and speed
    let speed = &args[3];

    let speed_number: u8 = speed
        .parse()
        .unwrap_or_else(|e| bail(format!("Unable to parse Speed: {e}")));

    let full_path = format!("{PORT}{serial_number}");

    // Turn "<F|R> <speed>" straight into a typed Command. The Command enum is the
    // single source of truth — no separate Direction enum to keep in sync.
    let command = get_command_action(&args[2], Some(speed_number))
        .unwrap_or_else(|e| bail(format!("Bad direction {:?}: {e:?}", args[2])));

    let direction_word = match command {
        Command::Forward(_) => "forward",
        Command::Reverse(_) => "backward",
        _ => "unknown",
    };

    println!(
        "Running {} at speed {} on  {}",
        direction_word, speed, full_path
    );

    let opened = serialport::new(full_path, BAUD)
        .timeout(Duration::from_millis(READ_TIMEOUT_MS))
        .open();

    let mut port = match opened {
        Ok(p) => p,
        Err(error) => {
            eprintln!("Couldn't open port: {error}");
            std::process::exit(1)
        }
    };

    println!("port opened...");

    port.write_all("Z\n".as_bytes())
        .expect("Failed to clear sensormoto");
    let running = Arc::new(AtomicBool::new(true));
    let handler_flag = running.clone(); // the handler gets its OWN Arc handle
    ctrlc::set_handler(move || {
        handler_flag.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl-C handler");

    let line = format!("{command}\n");
    port.write_all(line.as_bytes())
        .expect("failed to write command to port");

    let mut buf = [0u8; 1024];

    let mut reader = LineReader::new();

    while running.load(Ordering::SeqCst) {
        port.write_all("E\n".as_bytes())
            .expect("Failed to write out");

        match port.read(&mut buf) {
            Ok(n) => {
                for &byte in &buf[..n] {
                    if let Some(line) = reader.feed(byte) {
                        println!("{line}");
                    }
                }
            }
            Err(_) => continue,
        }
    }

    port.write_all("S\n".as_bytes())
        .expect("Failed to clear sensormotormotors");
}
