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
    ledc::{channel, timer, LSGlobalClkSource, Ledc, LowSpeed},
    main,
    rng::Rng,
    time::{Duration, Instant, Rate},
};
use esp_hal::ledc::channel::{ChannelHW, ChannelIFace};
use esp_hal::ledc::timer::TimerIFace;

use esp_println as _;

use esp_backtrace as _;


// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

/// Move `current` toward `target` by at most `step` duty-counts.
/// Returns `target` once the remaining distance is within one step.
fn step_toward(current: i32, target: i32, step: i32) -> i32 {
    if current < target {
        (current + step).min(target)
    } else {
        (current - step).max(target)
    }
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

    // from random import randint  -> hardware RNG peripheral (esp-hal 1.1.x: Rng::new takes no handle)
    let mut rng = Rng::new();

    // pins=[38,39,40]  -> GPIO38 (R), GPIO39 (G), GPIO40 (B)
    // pwm0=PWM(Pin(pins[0]),10000) / pwm1 / pwm2  -> one LEDC timer (10 kHz) shared by 3 channels.
    // MicroPython's duty() is 10-bit (0..1023), so use Duty10Bit and set duty by raw HW value.
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(10),
        })
        .unwrap();

    // pwm0=PWM(Pin(38),10000)
    let mut channel_r = ledc.channel(channel::Number::Channel0, peripherals.GPIO38);
    channel_r
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // pwm1=PWM(Pin(39),10000)
    let mut channel_g = ledc.channel(channel::Number::Channel1, peripherals.GPIO39);
    channel_g
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // pwm2=PWM(Pin(40),10000)
    let mut channel_b = ledc.channel(channel::Number::Channel2, peripherals.GPIO40);
    channel_b
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // --- gradient state (replaces the original "snap to a new random color each tick") ---
    // How fast the fade moves: bigger STEP or smaller STEP_DELAY_MS = faster transitions.
    const STEP: i32 = 4; // duty-counts moved per tick (out of 0..1023)
    const STEP_DELAY_MS: u64 = 15; // pause between steps

    // Current color, and the random target we're easing toward. randint(0,1023) -> rng.random() % 1024.
    let mut red = (rng.random() % 1024) as i32;
    let mut green = (rng.random() % 1024) as i32;
    let mut blue = (rng.random() % 1024) as i32;

    let mut target_r = (rng.random() % 1024) as i32;
    let mut target_g = (rng.random() % 1024) as i32;
    let mut target_b = (rng.random() % 1024) as i32;

    loop {
        // Ease each channel one step toward its target.
        red = step_toward(red, target_r, STEP);
        green = step_toward(green, target_g, STEP);
        blue = step_toward(blue, target_b, STEP);

        // setColor(red,green,blue) with inverted duty (1023-x: common-anode RGB LED).
        channel_r.set_duty_hw((1023 - red) as u32);
        channel_g.set_duty_hw((1023 - green) as u32);
        channel_b.set_duty_hw((1023 - blue) as u32);

        // Arrived at the target color -> pick a fresh random target to fade toward.
        if red == target_r && green == target_g && blue == target_b {
            target_r = (rng.random() % 1024) as i32;
            target_g = (rng.random() % 1024) as i32;
            target_b = (rng.random() % 1024) as i32;
        }

        // time.sleep_ms(...) -> pace one fade step.
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(STEP_DELAY_MS) {}
    }

    // The MicroPython `except: pwm*.deinit()` cleanup path has no equivalent here:
    // `main` is `-> !` and never returns, so the loop runs forever (no exception/exit).
}