//! Connects to a WiFi. The environment variables ESP32_DEMO_WIFI_SSID and ESP32_DEMO_WIFI_PASS are needed.

use anyhow::bail;
use core::time::Duration;
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::netif::{EspNetif, EspNetifWait};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{EspWifi, WifiWait};
use std::net::Ipv4Addr;

pub fn connect_wifi(modem: Modem) -> anyhow::Result<EspWifi<'static>> {
    let sysloop = EspSystemEventLoop::take()?;
    let mut wifi = EspWifi::new(
        modem,
        sysloop.clone(),
        Some(EspDefaultNvsPartition::take()?),
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: env!("ESP32_DEMO_WIFI_SSID").into(),
        password: env!("ESP32_DEMO_WIFI_PASS").into(),
        ..Default::default()
    }))?;

    wifi.start()?;

    println!("Starting wifi...");

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    println!("Connecting wifi...");

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    println!("Wifi DHCP info: {:?}", ip_info);

    Ok(wifi)
}
