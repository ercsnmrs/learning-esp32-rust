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
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
};

use esp_println as _;

use esp_backtrace as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

// pins = [21, 47, 48, 38, 39, 40, 41, 42, 2, 1]

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32s3 -o unstable-hal -o defmt -o esp-backtrace

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let delay = Delay::new();

    // The MicroPython code re-creates `Pin(pins[i], Pin.OUT)` on every pass. In esp-hal a GPIO is a
    // move-only resource and can't be indexed by a runtime integer, so we build every Output once,
    // up front, in the same order as the original `pins` list, and iterate over the array instead.
    let mut leds = [
        Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO47, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO48, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO38, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO39, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO40, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO41, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO42, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO1, Level::Low, OutputConfig::default()),
    ];

    // def showled():  -> the body of the while True loop
    loop {
        // for i in range(0, length):   (forward sweep)
        for led in leds.iter_mut() {
            // led.value(1)
            led.set_high();
            // time.sleep_ms(100)
            delay.delay_millis(100);
            // led.value(0)
            led.set_low();
        }
        // for i in range(0, length):   (backward sweep: pins[length - i - 1])
        for led in leds.iter_mut().rev() {
            // led.value(1)
            led.set_high();
            // time.sleep_ms(100)
            delay.delay_millis(100);
            // led.value(0)
            led.set_low();
        }
    }
}