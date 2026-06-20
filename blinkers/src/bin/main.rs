#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Level;
use esp_hal::main;
use esp_hal::rmt::{PulseCode, Rmt, TxChannelConfig, TxChannelCreator};
use esp_hal::time::{Duration, Instant, Rate};
use esp_println as _;

esp_bootloader_esp_idf::esp_app_desc!();

// WS2812 timing at 80MHz base, clk_divider = 1 → 12.5ns per tick
const T0H: u16 = 32; // 0.40us  high for bit 0
const T0L: u16 = 68; // 0.85us  low  for bit 0
const T1H: u16 = 64; // 0.80us  high for bit 1
const T1L: u16 = 36; // 0.45us  low  for bit 1

fn encode_rgb(r: u8, g: u8, b: u8) -> [PulseCode; 25] {
    // PulseCode is Copy; end_marker terminates the sequence
    let mut pulses = [PulseCode::end_marker(); 25];
    // WS2812 expects GRB order
    let data: u32 = ((g as u32) << 16) | ((r as u32) << 8) | (b as u32);

    for i in 0..24 {
        let bit = (data >> (23 - i)) & 1;
        pulses[i] = if bit == 1 {
            PulseCode::new(Level::High, T1H, Level::Low, T1L)
        } else {
            PulseCode::new(Level::High, T0H, Level::Low, T0L)
        };
    }
    // pulses[24] stays as end_marker
    pulses
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // ESP32-S3 DevKitC-1 onboard RGB LED is on GPIO48
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    let tx_config = TxChannelConfig::default().with_clk_divider(1);
    let mut channel = rmt
        .channel0
        .configure_tx(&tx_config)
        .unwrap()
        .with_pin(peripherals.GPIO48);

    let colors: [(u8, u8, u8); 6] = [
        (255, 0,   0),   // Red
        (0,   255, 0),   // Green
        (0,   0,   255), // Blue
        (255, 255, 0),   // Yellow
        (0,   255, 255), // Cyan
        (255, 0,   255), // Magenta
    ];

    let mut idx = 0;
    loop {
        let (r, g, b) = colors[idx];
        info!("LED color: R={} G={} B={}", r, g, b);

        let pulses = encode_rgb(r, g, b);
        let tx = channel.transmit(&pulses).unwrap();
        channel = tx.wait().unwrap();

        idx = (idx + 1) % colors.len();

        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(500) {}
    }
}