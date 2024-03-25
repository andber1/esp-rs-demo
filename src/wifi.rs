//! Connects to a WiFi. The environment variables ESP32_DEMO_WIFI_SSID and ESP32_DEMO_WIFI_PASS are needed.

use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi};

pub fn connect(modem: Modem) -> anyhow::Result<BlockingWifi<EspWifi<'static>>> {
    let sysloop = EspSystemEventLoop::take()?;
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(
            modem,
            sysloop.clone(),
            Some(EspDefaultNvsPartition::take()?),
        )?,
        sysloop,
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: env!("ESP32_DEMO_WIFI_SSID").try_into().unwrap(),
        password: env!("ESP32_DEMO_WIFI_PASS").try_into().unwrap(),
        ..Default::default()
    }))?;

    println!("Starting Wifi...");
    wifi.start()?;
    println!("Wifi started");

    wifi.connect()?;
    println!("Wifi connected");

    wifi.wait_netif_up()?;
    println!("Wifi netif up");

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    println!("Wifi DHCP info: {:?}", ip_info);

    Ok(wifi)
}
