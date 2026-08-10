// hall-probe — a learn-Rust-from-scratch bench tool for the Neato ESP32.

use protocol::command::Command;
use protocol::serial::LineReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const PORT: &str = "/tmp/cu.usbserial-"; // TEMP: fake-firmware.py pty; restore /dev/cu.usbserial- for real hardware

const BAUD: u32 = 115_200;
const READ_TIMEOUT_MS: u64 = 100;

enum Direction {
    Forward,
    Backward,
}

fn get_direction(direction: &str) -> Result<Direction, String> {
    match direction {
        "F" => Ok(Direction::Forward),
        "B" => Ok(Direction::Backward),
        "R" => Ok(Direction::Backward),
        _ => Err(format!("Bad direction {direction}")),
    }
}

fn bail<T>(msg: String) -> T {
    eprintln!("{msg}");
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        bail::<()>("Usage: hall-probe <serial-number> <F|B> <speed>".to_string());
    }

    let serial_number = &args[1];
    let direction = get_direction(&args[2]).unwrap_or_else(bail::<Direction>);
    let speed = &args[3];
    let direction_word = match direction {
        Direction::Forward => "forward",
        Direction::Backward => "backward",
    };

    let speed_number: u8 = speed
        .parse()
        .unwrap_or_else(|| bail(format!("Unable to parse Speed: {}")));
    let full_path = format!("{PORT}{serial_number}");

    println!(
        "Running {} at speed {} on serial {}",
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

    println!("port opened");

    port.write_all("Z\n".as_bytes())
        .expect("Failed to clear sensormoto");
    let running = Arc::new(AtomicBool::new(true));
    let handler_flag = running.clone(); // the handler gets its OWN Arc handle
    ctrlc::set_handler(move || {
        handler_flag.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl-C handler");

    let command = match direction {
        Direction::Forward => Command::Forward(speed_number),
        Direction::Backward => Command::Reverse(speed_number),
    };

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
