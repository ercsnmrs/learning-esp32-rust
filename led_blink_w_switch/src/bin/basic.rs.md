#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    main,
};

use esp_println as _;

use esp_backtrace as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.3.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // PIN_LED 2  -> output, start LOW (off)
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    // PIN_BUTTON 13 -> input
    let button_config = InputConfig::default().with_pull(Pull::None);
    let button = Input::new(peripherals.GPIO13, button_config);

    loop {
        if button.is_low() {
            led.set_low();
        } else {
            led.set_high();
        }
    }
}