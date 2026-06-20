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
    gpio::DriveMode,
    main,
    time::Rate,
};

use esp_hal::ledc::{
    channel::{self, ChannelIFace},
    timer::{self, TimerIFace},
    LSGlobalClkSource, Ledc, LowSpeed,
};

use esp_println as _;

use esp_backtrace as _;


// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

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

    // from machine import Pin, PWM
    // pwm = PWM(Pin(2), 10000)
    // -> GPIO2 is driven by the LEDC PWM controller. The 10000 (10 kHz) carrier
    //    frequency is set on the timer below; the pin is bound to a channel.
    let led = peripherals.GPIO2;

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    // 10 kHz carrier. Duty10Bit mirrors MicroPython's 0..1023 (10-bit) duty range,
    // and 80 MHz APB / (2^10 * 10 kHz) is a valid divider on the S3.
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(10),
        })
        .unwrap();

    let mut channel0 = ledc.channel(channel::Number::Channel0, led);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0, // start fully off, matching the first pwm.duty(0)
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // try:
    //     while True:
    loop {
        // for i in range(0, 1023): pwm.duty(i); time.sleep_ms(1)
        // ramp 0% -> 100% over ~1023 ms (hardware fade, no CPU involvement)
        channel0.start_duty_fade(0, 100, 1023).unwrap();
        while channel0.is_duty_fade_running() {}

        // for i in range(0, 1023): pwm.duty(1023 - i); time.sleep_ms(1)
        // ramp 100% -> 0% over ~1023 ms
        channel0.start_duty_fade(100, 0, 1023).unwrap();
        while channel0.is_duty_fade_running() {}
    }

    // except:
    //     pwm.deinit()
    // -> No equivalent in no_std Rust: `main` is `-> !` and never unwinds, so there
    //    is no teardown path. The LEDC peripheral simply lives for the whole program.

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}