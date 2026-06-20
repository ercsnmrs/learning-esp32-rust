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
    main,
    time::{Duration, Instant},
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

    // adc=ADC(Pin(1))                 -> GPIO1 is ADC1 channel 0 on the ESP32-S3
    // adc.atten(ADC.ATTN_11DB)        -> Attenuation::_11dB
    // adc.width(ADC.WIDTH_12BIT)      -> 12-bit (0..=4095) is the oneshot default on the S3
    let mut adc_config = AdcConfig::new();
    let mut adc_pin = adc_config.enable_pin(peripherals.GPIO1, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    // try:  (the bare except just swallows errors; an infinite loop has no equivalent)
    loop {
        // adcVal=adc.read()
        let adc_val: u16 = loop {
            if let Ok(v) = adc.read_oneshot(&mut adc_pin) {
                break v;
            }
        };
        // voltage = adcVal / 4095.0 * 3.3
        let voltage = adc_val as f32 / 4095.0 * 3.3;
        // print("ADC Val:",adcVal,"Voltage:",voltage,"V")
        info!("ADC Val: {} Voltage: {} V", adc_val, voltage);
        // time.sleep_ms(100)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}