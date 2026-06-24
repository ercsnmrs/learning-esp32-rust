#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    gpio::DriveMode,
    ledc::{
        channel::{self, ChannelHW, ChannelIFace},
        timer::{self, TimerIFace},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
    main,
    time::{Duration, Instant, Rate},
};

use esp_println as _;
use defmt::info;

use esp_backtrace as _;


// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// def remap(value,oldMin,oldMax,newMin,newMax):
//     return int((value)*(newMax-newMin)/(oldMax-oldMin))
// Translated faithfully, including the original's quirk of ignoring oldMin/newMin.
fn remap(value: i32, old_min: i32, old_max: i32, new_min: i32, new_max: i32) -> i32 {
    value * (new_max - new_min) / (old_max - old_min)
}

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

    // --- setup: MicroPython module-level code ---

    // pwm = PWM(Pin(14,Pin.OUT),1000)
    // Bring up LEDC with a low-speed timer at 1000 Hz and 10-bit duty resolution
    // (0..=1023, matching MicroPython's legacy PWM.duty range), then a channel on GPIO14.
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(1000),
        })
        .unwrap();

    let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO14);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // adc = ADC(Pin(1))
    // adc.atten(ADC.ATTN_11DB)
    // adc.width(ADC.WIDTH_12BIT)   // 12-bit (0..=4095) is the ESP32-S3 oneshot default
    let mut adc_config = AdcConfig::new();
    let mut adc_pin = adc_config.enable_pin(peripherals.GPIO1, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    // --- loop: while True: ---
    loop {
        // adcValue=adc.read()
        // read_oneshot is non-blocking (Err == conversion still in progress); spin
        // until it yields a value, mirroring MicroPython's blocking adc.read().
        let adc_value: u16 = loop {
            if let Ok(value) = adc.read_oneshot(&mut adc_pin) {
                break value;
            }
        };

        // pwmValue=remap(adcValue,0,4095,0,1023)
        // Only drive the channel at full ADC (4095); any lower reading -> 0 (off).
        // On a 10-bit LEDC timer full-on is 1024 (not 1023).
        let pwm_value = if adc_value < 4095 {
            0
        } else {
            remap(adc_value as i32, 0, 4095, 0, 1024)
        };

        // pwm.duty(pwmValue)
        channel0.set_duty_hw(pwm_value as u32);

        // print(adcValue,pwmValue)
        info!("{} {}", adc_value, pwm_value);

        // time.sleep_ms(100)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }

    // The MicroPython `except: pwm.deinit()` cleanup has no equivalent: `main` is
    // `-> !` and never returns; hardware faults are handled by esp-backtrace.
}