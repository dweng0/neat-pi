// hall-probe — a learn-Rust-from-scratch bench tool for the Neato ESP32.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const PORT: &str = "/dev/cu.usbserial-10";
const BAUD: u32 = 115_200;
const READ_TIMEOUT_MS: u64 = 100;

fn get_direction(direction: &str) -> &str {
    match direction {
        "F" => "forward",
        "B" => "backward",
        _ => "unknown",
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let direction = &args[1];
    let speed = &args[2];

    println!("Driving {} at speed {}", get_direction(direction), speed);

    let speed_number: u8 = match speed.parse::<u8>() {
        Ok(number) => number,
        Err(error) => {
            eprintln!("Couldn't read speed '{speed}' ({error}); defaulting to 0");
            0
        }
    };

    println!("parsed duty = {}", speed_number);

    // ── STEP 3: open the ESP32 serial port ───────────────────────────────
    let opened = serialport::new(PORT, BAUD)
        .timeout(Duration::from_millis(READ_TIMEOUT_MS))
        .open();

    let mut port = match opened {
        Ok(p) => p,
        Err(error) => {
            eprintln!("Couldn't open port {PORT}: {error}");
            std::process::exit(1)
        }
    };

    println!("port opened");

    port.write_all("Z\n".as_bytes())
        .expect("Failed to clear sensormoto");
    let running = Arc::new(AtomicBool::new(true));
    let handler_flag = running.clone(); // the handler gets its OWN Arc handle
    ctrlc::set_handler(move || {
        handler_flag.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl-C handler");

    let fw = match direction.as_str() {
        "B" => "R",
        _ => "F",
    };

    let line = format!("{} {}\n", fw, speed_number);
    port.write_all(line.as_bytes())
        .expect("failed to write command to port");

    let mut buf = [0u8; 1024];

    while running.load(Ordering::SeqCst) {
        port.write_all("E\n".as_bytes())
            .expect("Failed to write out");

        // port.read FILLS buf with fresh bytes and returns how many (n).
        match port.read(&mut buf) {
            Ok(n) => print!("{}", String::from_utf8_lossy(&buf[..n])),
            Err(_) => continue,
        }
    }

    port.write_all("S\n".as_bytes())
        .expect("Failed to clear sensormotormotors");
}
