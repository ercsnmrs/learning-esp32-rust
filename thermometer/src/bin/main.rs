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

    // adc=ADC(Pin(1))
    // adc.atten(ADC.ATTN_11DB)
    // adc.width(ADC.WIDTH_12BIT)   -- 12-bit (0..=4095) is the oneshot default, so /4095 below holds
    let mut adc_config = AdcConfig::new();
    let mut adc_pin = adc_config.enable_pin(peripherals.GPIO1, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    loop {
        // adcValue=adc.read()
        let adc_value: u16 = nb::block!(adc.read_oneshot(&mut adc_pin)).unwrap();

        // voltage=adcValue/4095*3.3
      let voltage = adc_value as f32 / 4095.0 * 3.3;

        if adc_value == 0 || adc_value >= 4095 {
            // Rail reading -> divider open or saturated, not a real temperature
            info!("ADC railed: value = {}  (check thermistor wiring on GPIO1)", adc_value);
        } else {
            let rt = 10.0 * voltage / (3.3 - voltage);
            let temp_k = 1.0 / (1.0 / (273.15 + 25.0) + libm::logf(rt / 10.0) / 3950.0);
            let temp_c = temp_k - 273.15;
            info!("ADC value: {}\tVoltage: {}\tTemperature: {}", adc_value, voltage, temp_c);
        }

        // Rt=10*voltage/(3.3-voltage)
        let rt = 10.0 * voltage / (3.3 - voltage);

        // tempK=(1/(1/(273.15+25)+(math.log(Rt/10))/3950))
        let temp_k = 1.0 / (1.0 / (273.15 + 25.0) + libm::logf(rt / 10.0) / 3950.0);

        // tempC=tempK-273.15
        let temp_c = temp_k - 273.15;

        // print("ADC value:",adcValue,"\tVoltage :",voltage,"\tTemperature :",tempC);
        info!(
            "ADC value: {}\tVoltage : {}\tTemperature : {}",
            adc_value, voltage, temp_c
        );

        // time.sleep_ms(1000)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }
}