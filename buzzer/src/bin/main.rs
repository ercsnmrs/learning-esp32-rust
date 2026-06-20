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
    gpio::{DriveMode, Input, InputConfig, Pull},
    ledc::{
        channel::{self, ChannelIFace},
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

    // button=Pin(21,Pin.IN,Pin.PULL_UP)
    let button = Input::new(peripherals.GPIO21, InputConfig::default().with_pull(Pull::Up));

    // passiveBuzzer=PWM(Pin(14),2000)
    // LEDC PWM at the base 2000 Hz; a passive buzzer sounds when driven with a ~50% duty square wave.
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            // Duty5Bit (from the 24 kHz docs example) makes the clock divisor overflow at 2000 Hz
            // (divisor must stay in 256..0x3FFFF). 13-bit keeps it in range and gives finer duty control.
            duty: timer::config::Duty::Duty13Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(2000),
        })
        .unwrap();

    let mut passive_buzzer = ledc.channel(channel::Number::Channel0, peripherals.GPIO14);
    passive_buzzer
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0, // start silent
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // try:  -> there is no direct equivalent of the Python try/except cleanup. main() is `-> !`
    //          and a panic traps in the esp-backtrace handler, which halts the program (and with it
    //          the PWM output), standing in for `except: passiveBuzzer.deinit()`.
    loop {
        // if not button.value():
        if button.is_low() {
            // passiveBuzzer.init()  -> drive at 50% duty so the buzzer sounds
            passive_buzzer.set_duty(50).unwrap();
            // alert()
            alert();
        } else {
            // passiveBuzzer.deinit()  -> 0% duty == silent
            passive_buzzer.set_duty(0).unwrap();
        }
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}

// PI=3.14
const PI: f32 = 3.14;

// def alert():
fn alert() {
    // for x in range(0,36):
    for x in 0..36 {
        // sinVal=math.sin(x*10*PI/180)
        let sin_val = libm::sinf(x as f32 * 10.0 * PI / 180.0);
        // toneVal=2000+int(sinVal*500)
        let tone_val = 2000 + (sin_val * 500.0) as i32;
        // passiveBuzzer.freq(toneVal)
        // NOTE: esp-hal 1.x LEDC is fixed-frequency, and the channel holds a shared (&) borrow of the
        // timer for its whole lifetime, so the timer frequency cannot be re-set here while the channel
        // is live. The intended tone is logged instead of being applied to the hardware. See the prose
        // note for how to actually sweep pitch on this HAL.
        info!("alert tone = {=i32} Hz", tone_val);
        // time.sleep_ms(10)
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(10) {}
    }
}