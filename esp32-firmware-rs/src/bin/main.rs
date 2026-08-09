#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

// The staandard esp_hal library
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::uart::{Config as UartConfig, Uart};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Onboard LED heartbeat (GPIO2), same pin as the C++ firmware.
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    // UART0 on the default USB-serial pins (GPIO1=TX, GPIO3=RX) at 115200 baud.
    let mut uart = Uart::new(
        peripherals.UART0,
        UartConfig::default().with_baudrate(115200),
    )
    .unwrap()
    .with_tx(peripherals.GPIO1)
    .with_rx(peripherals.GPIO3);

    uart.write(b"\r\n[neato-esp32-rs] milestone 1: heartbeat + echo online.\r\n")
        .ok();

    let mut last_beat = Instant::now();
    loop {
        // Non-blocking heartbeat.
        if last_beat.elapsed() >= Duration::from_millis(1000) {
            last_beat = Instant::now();
            led.toggle();
            uart.write(b".").ok();
        }

        // Drain and echo whatever the host sent, without blocking the heartbeat.
        while uart.read_ready() {
            let mut byte = [0u8; 1];
            if let Ok(1) = uart.read(&mut byte) {
                uart.write(&byte).ok();
            }
        }
    }
}
