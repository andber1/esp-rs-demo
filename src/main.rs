//! A simple STD binary for the development board [ESP32-C3-DevKit-RUST](https://github.com/esp-rs/esp-rust-board).

mod led;
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
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::delay::FreeRtos;
use led::LedDriver;
use rgb::RGB8;
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};

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

    // create temperature sensor
    let mut temperature_sensor = shtcx::shtc3(I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio10,
        peripherals.pins.gpio8,
        &Config::default().baudrate(KiloHertz::from(400).into()),
    )?);

    // create buffer for sensor data
    let buffer = Arc::new(Mutex::new(AllocRingBuffer::with_capacity(512)));
    let buffer2 = buffer.clone();

    // start webserver to plot data
    let server_config = Configuration {
        stack_size: 16000,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;
    server.fn_handler("/", Method::Get, move |req| {
        let data: Vec<_> = buffer2
            .lock()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(index, value)| [index as f64, *value as f64])
            .collect();
        let a = poloto::build::plot("temperature").line(data);
        let plot = poloto::data(a)
            .build_and_label(("Temperature", "x", "y"))
            .append_to(poloto::header().light_theme())
            .render_string()?;

        req.into_ok_response()?.write_all(plot.as_bytes())?;
        std::result::Result::Ok(())
    })?;

    led.set_color(RGB8::new(0_u8, 1_u8, 0_u8))?;
    loop {
        // start one measurement
        let measurement = temperature_sensor
            .measure(shtcx::PowerMode::NormalMode, &mut FreeRtos)
            .expect("SHTC3 measurement failure");
        buffer
            .lock()
            .unwrap()
            .push(measurement.temperature.as_degrees_celsius());

        FreeRtos::delay_ms(2000);
    }
}
