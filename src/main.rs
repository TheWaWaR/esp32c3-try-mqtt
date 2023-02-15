use std::net::Ipv4Addr;
use std::thread::{self, sleep};
use std::time::Duration;

use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::modem::WifiModem;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_svc::mqtt::client::{Client, Connection, Event, Message, Publish, QoS};
use embedded_svc::wifi::{ClientConfiguration, Configuration};

use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

include!(concat!(env!("OUT_DIR"), "/envs.rs"));

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().expect("peripherals");
    let timer_config = TimerConfig::default().frequency(1.kHz().into());
    let timer_driver0 =
        LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config).expect("create timer 0");
    let timer_driver1 =
        LedcTimerDriver::new(peripherals.ledc.timer1, &timer_config).expect("create timer 0");
    let mut driver4 = LedcDriver::new(
        peripherals.ledc.channel0,
        timer_driver0,
        peripherals.pins.gpio12,
        &timer_config,
    )
    .expect("create driver4");
    let mut driver5 = LedcDriver::new(
        peripherals.ledc.channel1,
        timer_driver1,
        peripherals.pins.gpio13,
        &timer_config,
    )
    .expect("create driver5");

    // ==== Wifi ====
    let mut wifi = EspWifi::new(
        unsafe { WifiModem::new() },
        EspSystemEventLoop::take().unwrap(),
        Some(EspDefaultNvsPartition::take().unwrap()),
    )
    .unwrap();
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PASS.into(),
        ..Default::default()
    }))
    .unwrap();
    wifi.start().unwrap();
    wifi.scan().unwrap();
    wifi.connect().unwrap();
    let netif = wifi.sta_netif();
    println!("Waiting for wifi......");
    loop {
        sleep(Duration::from_millis(200));
        let info = netif.get_ip_info().unwrap();
        if info.ip != Ipv4Addr::new(0, 0, 0, 0) {
            break;
        }
    }
    let mac = netif.get_mac().unwrap();
    let ip = netif.get_ip_info().unwrap();
    println!("wifi mac: {:?}, ip: {:?}", mac, ip);

    // ==== MQTT ====
    let mqtt_conf = MqttClientConfiguration {
        client_id: Some("my-esp32"),
        network_timeout: Duration::from_secs(5),
        username: Some(MQTT_USER),
        password: Some(MQTT_PASS),
        ..Default::default()
    };
    println!("Connecting to MQTT broker....");
    let (mut mqtt_client, mut mqtt_conn) =
        EspMqttClient::new_with_conn(format!("mqtt://{}", MQTT_HOST), &mqtt_conf).unwrap();
    thread::spawn(move || {
        println!("MQTT Listening for messages");
        while let Some(msg) = mqtt_conn.next() {
            match msg {
                Err(e) => println!("MQTT Message ERROR: {}", e),
                Ok(event) => {
                    println!("MQTT Event: {:?}", event);
                    // Wait for a received event
                    match event {
                        Event::Received(rcv) => {
                            let s = match std::str::from_utf8(rcv.data()) {
                                Ok(v) => v,
                                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                            };
                            println!("rcv: {:?}", s);
                        }
                        // Ignore all other events for now
                        _ => {}
                    }
                }
            }
        }
        println!("MQTT connection loop exit");
    });

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
        driver4.set_duty(duty4).expect("set duty4");
        driver5.set_duty(duty5).expect("set duty5");
        sleep(Duration::from_millis(20));
        if i % 25 == 0 {
            println!(
                "> tick #{:003}, parts: {:003}, duty4: {:003}, duty5: {:003}",
                i, duty_parts, duty4, duty5,
            );
        }
        if i % 100 == 0 {
            mqtt_client
                .publish(
                    "testtopic/thewawar/esp32c3",
                    QoS::AtMostOnce,
                    false,
                    format!("D4.duty: {}, D5.duty: {}", duty4, duty5).as_bytes(),
                )
                .unwrap();
        }
    }
}
