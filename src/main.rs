//! A simple STD binary for the development board [ESP32-C3-DevKit-RUST](https://github.com/esp-rs/esp-rust-board).

mod led;
mod plot;
mod wifi;

use std::sync::{Arc, Mutex};

use crate::wifi::connect_wifi;
use embedded_svc::http::Method;
use embedded_svc::io::Write;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::rmt::config::TransmitConfig;
use esp_idf_hal::rmt::TxRmtDriver;
use esp_idf_hal::{
    i2c::{config::Config, I2cDriver},
    units::KiloHertz,
};
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::delay::FreeRtos;
use led::LedDriver;
use rgb::RGB8;
use ringbuffer::{AllocRingBuffer, RingBufferWrite};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // setup RGB LED
    let peripherals = Peripherals::take().unwrap();
    let config = TransmitConfig::new().clock_divider(1);
    let mut led = TxRmtDriver::new(peripherals.rmt.channel0, peripherals.pins.gpio2, &config)?;

    // connect wo wifi
    let _wifi = connect_wifi(peripherals.modem)?;

    // sync time
    let sntp = EspSntp::new_default()?;

    // create temperature sensor
    let mut temperature_sensor = shtcx::shtc3(I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio10,
        peripherals.pins.gpio8,
        &Config::default().baudrate(KiloHertz::from(400).into()),
    )?);

    // create buffer for sensor data
    const BUFFER_SIZE: usize = 512;
    let buffer_temp = Arc::new(Mutex::new(AllocRingBuffer::with_capacity(BUFFER_SIZE)));
    let buffer_humidity = Arc::new(Mutex::new(AllocRingBuffer::with_capacity(BUFFER_SIZE)));

    // start webserver to plot data
    let server_config = Configuration {
        stack_size: 16000,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;
    let buffer_temp2 = buffer_temp.clone();
    let buffer_humidity2 = buffer_humidity.clone();
    server.fn_handler("/", Method::Get, move |req| {
        let svg_plot = plot::create_svg_plot(
            &buffer_temp2.lock().unwrap(),
            &buffer_humidity2.lock().unwrap(),
        )?;
        req.into_ok_response()?.write_all(svg_plot.as_bytes())?;
        std::result::Result::Ok(())
    })?;

    led.set_color(RGB8::new(0_u8, 10_u8, 0_u8))?;
    loop {
        if sntp.get_sync_status() == SyncStatus::InProgress {
            continue;
        }
        // start one measurement
        let now = poloto::num::timestamp::UnixTime(EspSystemTime.now().as_secs() as i64);
        let measurement = temperature_sensor
            .measure(shtcx::PowerMode::NormalMode, &mut FreeRtos)
            .expect("SHTC3 measurement failure");
        buffer_temp
            .lock()
            .unwrap()
            .push((now, measurement.temperature.as_degrees_celsius() as f64));
        buffer_humidity
            .lock()
            .unwrap()
            .push((now, measurement.humidity.as_percent() as f64));

        FreeRtos::delay_ms(2000);
    }
}
