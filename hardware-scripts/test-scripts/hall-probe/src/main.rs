// hall-probe — a learn-Rust-from-scratch bench tool for the Neato ESP32.

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

    // ── STEP 4: translate direction, then send the line ──────────────────
    // A `match` is an EXPRESSION; `let fw =` binds whatever arm wins.
    // .as_str() turns &String -> &str so the "F"/"R" patterns line up.
    let fw = match direction.as_str() {
        "B" => "R",
        _ => "F",
    };

    let line = format!("{} {}\n", fw, speed_number);
    port.write_all(line.as_bytes())
        .expect("failed to write command to port");

    // ── STEP 5: read the robot's replies (is the hall sensor alive?) ──────
    // Once it starts driving, the firmware streams text lines back, e.g.
    //   "[motor] fwd duty=180"   and   "[enc] A=.. B=.. pos=.."
    let mut buf = [0u8; 1024];

    // ✗ `loop` stands alone — no `for _ in`. It should just be:
    //   loop {

    loop {
        port.write_all("E\n".as_bytes())
            .expect("Failed to write out");

        // port.read FILLS buf with fresh bytes and returns how many (n).
        match port.read(&mut buf) {
            // got n bytes -> print just those (&buf[..n]), decoded as text.
            // from_utf8_lossy turns raw bytes into a string, swapping any
            // invalid byte for a placeholder instead of panicking.
            Ok(n) => print!("{}", String::from_utf8_lossy(&buf[..n])),
            // a read TIMEOUT lands here (no data this pass) -> just try again.
            Err(_) => continue,
        }
    }
}
