//! A simple STD binary for the development board [ESP32-C3-DevKit-RUST](https://github.com/esp-rs/esp-rust-board).

mod led;
mod wifi;

use crate::led::LedDriver;
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
use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::delay::FreeRtos;

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

    // crete temperature sensor
    let mut temperature_sensor = shtcx::shtc3(I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio10,
        peripherals.pins.gpio8,
        &Config::default().baudrate(KiloHertz::from(400).into()),
    )?);

    // start one measurement
    let measurement = temperature_sensor
        .measure(shtcx::PowerMode::NormalMode, &mut FreeRtos)
        .expect("SHTC3 measurement failure");

    // start webserver
    let mut server = EspHttpServer::new(&Default::default())?;
    server.fn_handler("/", Method::Get, move |req| {
        req.into_ok_response()?.write_all(
            format!(
                "Temperature {} deg C<br>Humidity {} %",
                measurement.temperature.as_degrees_celsius(),
                measurement.humidity.as_percent()
            )
            .as_bytes(),
        )?;
        std::result::Result::Ok(())
    })?;

    let mut hue = 0_u8;
    loop {
        hue = hue.wrapping_add(40);
        led.set_color(led::hue_to_color(hue))?;
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
