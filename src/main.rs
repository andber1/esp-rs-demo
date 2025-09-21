//! A simple STD binary for the development board [ESP32-C3-DevKit-RUST](https://github.com/esp-rs/esp-rust-board).

mod led;
mod plot;
mod wifi;

use std::sync::{Arc, Mutex};

use esp_idf_hal::rmt::TxRmtDriver;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::rmt::config::TransmitConfig;
use esp_idf_svc::hal::{
    i2c::{config::Config, I2cDriver},
    units::KiloHertz,
};
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::systime::EspSystemTime;

use led::LedDriver;
use rgb::RGB8;
use ringbuffer::{AllocRingBuffer, RingBuffer};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // setup RGB LED
    let peripherals = Peripherals::take().unwrap();
    let config = TransmitConfig::new().clock_divider(1);
    let mut led = TxRmtDriver::new(peripherals.rmt.channel0, peripherals.pins.gpio2, &config)?;
    led.set_color(RGB8::new(10_u8, 0_u8, 0_u8))?;

    // connect to wifi
    let mut wifi = wifi::connect(peripherals.modem)?;

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
    const BUFFER_SIZE: usize = 2048;
    let buffer = Arc::new(Mutex::new(AllocRingBuffer::new(BUFFER_SIZE)));

    // start webserver to plot data
    let server_config = Configuration {
        stack_size: 16000,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;
    const HTML_HEADER: &str = r#"<html><head><meta charset="UTF-8"></head><body>"#;
    const HTML_FOOTER: &str = r"</body></html>";
    let buffer2 = buffer.clone();
    server.fn_handler("/temperature", Method::Get, move |req| {
        let mut res = req.into_ok_response()?;
        res.write_all(HTML_HEADER.as_bytes())?;
        match plot::create_svg_plot(&buffer2.lock().unwrap(), 0, "Temperature") {
            Ok(svg_plot) => {
                res.write_all(svg_plot.as_bytes())?;
            }
            Err(err) => {
                res.write_all(b"Error creating svg plot: ")?;
                res.write_all(err.to_string().as_bytes())?;
            }
        }
        res.write_all(HTML_FOOTER.as_bytes())
    })?;
    let buffer3 = buffer.clone();
    server.fn_handler("/humidity", Method::Get, move |req| {
        let mut res = req.into_ok_response()?;
        res.write_all(HTML_HEADER.as_bytes())?;
        match plot::create_svg_plot(&buffer3.lock().unwrap(), 1, "Humidity") {
            Ok(svg_plot) => {
                res.write_all(svg_plot.as_bytes())?;
            }
            Err(err) => {
                res.write_all(b"Error creating svg plot: ")?;
                res.write_all(err.to_string().as_bytes())?;
            }
        }
        res.write_all(HTML_FOOTER.as_bytes())
    })?;

    while sntp.get_sync_status() != SyncStatus::Completed {
        FreeRtos::delay_ms(100);
    }
    led.set_color(RGB8::new(0_u8, 10_u8, 0_u8))?;
    log::info!("Start measurement loop");
    loop {
        // start one measurement
        let now = poloto_chrono::UnixTime(EspSystemTime.now().as_secs() as i64);
        let measurement = temperature_sensor
            .measure(shtcx::PowerMode::NormalMode, &mut FreeRtos)
            .expect("SHTC3 measurement failure");
        buffer.lock().unwrap().enqueue((
            now,
            [
                measurement.temperature.as_degrees_celsius(),
                measurement.humidity.as_percent(),
            ],
        ));

        if let Ok(false) = wifi.is_up() {
            log::warn!("Lost wifi connection, try to connect again...");
            wifi.connect()?;
        }
        FreeRtos::delay_ms(2000);
    }
}
