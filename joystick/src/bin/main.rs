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

    // xVal=ADC(Pin(14))   -> GPIO14 = ADC2_CH3 on the ESP32-S3
    // yVal=ADC(Pin(13))   -> GPIO13 = ADC2_CH2 on the ESP32-S3
    // xVal.atten(ADC.ATTN_11DB)
    // yVal.atten(ADC.ATTN_11DB)
    // (width is set via AdcConfig::new(); 12-bit is the default on this HAL)
    let mut adc_config = AdcConfig::new();
    let mut x_pin = adc_config.enable_pin(peripherals.GPIO14, Attenuation::_11dB);
    let mut y_pin = adc_config.enable_pin(peripherals.GPIO13, Attenuation::_11dB);
    let mut adc2 = Adc::new(peripherals.ADC2, adc_config);

    // Sample the resting position once at startup so the centered output reads 0,0.
    // Leave the stick untouched during power-up for this to calibrate correctly.
    let x_center: i32 = nb::block!(adc2.read_oneshot(&mut x_pin)).unwrap() as i32;
    let y_center: i32 = nb::block!(adc2.read_oneshot(&mut y_pin)).unwrap() as i32;

    loop {
        // print("X,Y:",xVal.read(),",",yVal.read())
        let x_raw: i32 = nb::block!(adc2.read_oneshot(&mut x_pin)).unwrap() as i32;
        let y_raw: i32 = nb::block!(adc2.read_oneshot(&mut y_pin)).unwrap() as i32;

        // Offset from the resting center -> resting = 0,0, range roughly -2048..+2047
        let x = x_raw - x_center;
        let y = y_raw - y_center;
        info!("X,Y: {}, {}", x, y);

        // time.sleep(1)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_secs(1) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}