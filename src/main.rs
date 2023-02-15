use std::thread::sleep;
use std::time::Duration;

use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    println!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let timer_config = TimerConfig::default().frequency(1.kHz().into());
    let timer_driver0 = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config).unwrap();
    let timer_driver1 = LedcTimerDriver::new(peripherals.ledc.timer1, &timer_config).unwrap();
    let mut driver4 = LedcDriver::new(
        peripherals.ledc.channel0,
        timer_driver0,
        peripherals.pins.gpio12,
        &timer_config,
    )
    .unwrap();
    let mut driver5 = LedcDriver::new(
        peripherals.ledc.channel1,
        timer_driver1,
        peripherals.pins.gpio13,
        &timer_config,
    )
    .unwrap();

    const POW: u32 = 2;
    const TOTAL: u32 = 100;
    fn map_duty(max_duty: f64, parts: u32) -> u32 {
        let total_p3: f64 = f64::from(TOTAL.pow(POW));
        (max_duty * f64::from(parts).powi(POW as i32) / total_p3) as u32
    }
    let max_duty = f64::from(driver4.get_max_duty()) * 0.618;
    let mut increase = true;
    let mut duty_parts = 0;
    for i in 0u32.. {
        if increase {
            duty_parts += 1;
        } else {
            duty_parts -= 1;
        }
        if duty_parts == TOTAL || duty_parts == 0 {
            increase = !increase;
        }
        let duty4 = map_duty(max_duty, duty_parts);
        let duty5 = map_duty(max_duty, TOTAL - duty_parts);
        driver4.set_duty(duty4).unwrap();
        driver5.set_duty(duty5).unwrap();
        sleep(Duration::from_millis(20));
        if i % 5 == 0 {
            println!(
                "> tick #{:003}, parts: {:003}, duty4: {:003}, duty5: {:003}",
                i, duty_parts, duty4 as u32, duty5 as u32,
            );
        }
    }
}
