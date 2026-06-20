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

    // adc0=ADC(Pin(12)); adc1=ADC(Pin(13)); adc2=ADC(Pin(14))
    // adcN.atten(ADC.ATTN_11DB); adcN.width(ADC.WIDTH_12BIT)  -> oneshot is 12-bit (0..=4095)
    // GPIO12/13/14 are all ADC2 channels on the ESP32-S3, so they share one Adc instance.
    let mut adc_config = AdcConfig::new();
    let mut adc_pin0 = adc_config.enable_pin(peripherals.GPIO12, Attenuation::_11dB);
    let mut adc_pin1 = adc_config.enable_pin(peripherals.GPIO13, Attenuation::_11dB);
    let mut adc_pin2 = adc_config.enable_pin(peripherals.GPIO14, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC2, adc_config);

    // pwm0=PWM(Pin(40),10000); pwm1=PWM(Pin(39),10000); pwm2=PWM(Pin(38),10000)
    // -> one LEDC low-speed timer at 10 kHz / 10-bit duty, shared by three channels.
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

    // pwm0 -> GPIO40
    let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO40);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    // pwm1 -> GPIO39
    let mut channel1 = ledc.channel(channel::Number::Channel1, peripherals.GPIO39);
    channel1
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    // pwm2 -> GPIO38
    let mut channel2 = ledc.channel(channel::Number::Channel2, peripherals.GPIO38);
    channel2
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // def remap(value,oldMin,oldMax,newMin,newMax):
    //     return int((value)*(newMax-newMin)/(oldMax-oldMin))
    // -> inlined below as remap(adcN.read(),0,4095,0,1023) == adc_value * 1023 / 4095

    loop {
        // pwm0.duty(1023-remap(adc0.read(),0,4095,0,1023))
        let v0: u16 = nb::block!(adc.read_oneshot(&mut adc_pin0)).unwrap();
        channel0.set_duty_hw(1023 - (v0 as u32 * 1023 / 4095));
        // pwm1.duty(1023-remap(adc1.read(),0,4095,0,1023))
        let v1: u16 = nb::block!(adc.read_oneshot(&mut adc_pin1)).unwrap();
        channel1.set_duty_hw(1023 - (v1 as u32 * 1023 / 4095));
        // pwm2.duty(1023-remap(adc2.read(),0,4095,0,1023))
        let v2: u16 = nb::block!(adc.read_oneshot(&mut adc_pin2)).unwrap();
        channel2.set_duty_hw(1023 - (v2 as u32 * 1023 / 4095));
        // time.sleep_ms(100)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }

    // except: pwm0.deinit() / pwm1.deinit() / pwm2.deinit()
    // No equivalent: main is `-> !` and never returns, so there is no exit path to clean up on.
}