use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::rmt::config::TransmitConfig;
use esp_idf_hal::rmt::TxRmtDriver;

mod led;
use crate::led::LedDriver;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // setup RGB LED
    let peripherals = Peripherals::take().unwrap();
    let config = TransmitConfig::new().clock_divider(1);
    let mut led = TxRmtDriver::new(peripherals.rmt.channel0, peripherals.pins.gpio2, &config)?;

    let mut hue = 0_u8;
    loop {
        hue = hue.wrapping_add(40);
        led.set_color(led::hue_to_color(hue))?;
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
