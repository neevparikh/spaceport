use anyhow::{anyhow, Result};
use objc2::rc::Retained;
use objc2_core_location::{CLAuthorizationStatus, CLLocationManager};
use objc2_core_wlan::{CWInterface, CWWiFiClient};
use serde::Serialize;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Default, Serialize)]
struct Wifi {
    ssid: Option<String>,
    power: bool,
    rssi: isize,
    noise: isize,
    tx_rate: f64,
    tx_power: isize,
}

impl Wifi {
    unsafe fn from_interface(interface: &Retained<CWInterface>) -> Self {
        Wifi {
            ssid: interface.ssid().map(|s| s.to_string()),
            power: interface.powerOn(),
            rssi: interface.rssiValue(),
            noise: interface.noiseMeasurement(),
            tx_rate: interface.transmitRate(),
            tx_power: interface.transmitPower(),
        }
    }

    unsafe fn update(&mut self, interface: &Retained<CWInterface>) {
        self.ssid = interface.ssid().map(|s| s.to_string());
        self.power = interface.powerOn();
        self.rssi = interface.rssiValue();
        self.noise = interface.noiseMeasurement();
        self.tx_rate = interface.transmitRate();
        self.tx_power = interface.transmitPower();
    }

    fn to_json(&self) -> Result<String> {
        let json = serde_json::to_string_pretty(&self)?;
        Ok(json)
    }
}

fn main() -> Result<()> {
    // access interface
    let interface = unsafe {
        let manager = CLLocationManager::new();
        manager.requestAlwaysAuthorization();
        manager.startUpdatingLocation();

        while manager.authorizationStatus() != CLAuthorizationStatus(3) {
            sleep(Duration::from_millis(10))
        }

        CWWiFiClient::sharedWiFiClient().interface()
    }
    .ok_or(anyhow!("Unable to get wifi interface"))?;

    // extract required data
    let wifi = unsafe { Wifi::from_interface(&interface) };

    // dump to file
    fs::write("/tmp/spaceport.json", wifi.to_json()?)?;

    Ok(())
}
