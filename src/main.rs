use anyhow::{anyhow, Result};
use objc2::rc::autoreleasepool;
use objc2_core_location::{CLAuthorizationStatus, CLLocationManager};
use objc2_core_wlan::{CWInterface, CWWiFiClient};
use serde::Serialize;
use std::fs;
use std::thread::sleep;
use std::time::{Duration, Instant};

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
    unsafe fn from_interface(interface: &CWInterface) -> Self {
        Wifi {
            ssid: interface.ssid().map(|s| s.to_string()),
            power: interface.powerOn(),
            rssi: interface.rssiValue(),
            noise: interface.noiseMeasurement(),
            tx_rate: interface.transmitRate(),
            tx_power: interface.transmitPower(),
        }
    }

    fn current(interface: &CWInterface) -> Self {
        autoreleasepool(|_| unsafe { Self::from_interface(interface) })
    }

    fn to_json(&self) -> Result<String> {
        let json = serde_json::to_string_pretty(&self)?;
        Ok(json)
    }
}

fn ensure_location_authorization() -> Result<()> {
    let manager = unsafe { CLLocationManager::new() };
    unsafe {
        manager.requestWhenInUseAuthorization();
        manager.startUpdatingLocation();
    }

    let authorization_deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let status = autoreleasepool(|_| unsafe { manager.authorizationStatus() });
        match status {
            CLAuthorizationStatus::kCLAuthorizationStatusAuthorizedAlways
            | CLAuthorizationStatus::kCLAuthorizationStatusAuthorizedWhenInUse => {
                unsafe { manager.stopUpdatingLocation() };
                return Ok(());
            }
            CLAuthorizationStatus::kCLAuthorizationStatusDenied
            | CLAuthorizationStatus::kCLAuthorizationStatusRestricted => {
                unsafe { manager.stopUpdatingLocation() };
                return Err(anyhow!(
                    "Location access is required to read the Wi-Fi SSID"
                ));
            }
            _ if Instant::now() >= authorization_deadline => {
                unsafe { manager.stopUpdatingLocation() };
                return Err(anyhow!(
                    "Timed out waiting for location access; grant access and relaunch Spaceport"
                ));
            }
            _ => sleep(Duration::from_millis(100)),
        }
    }
}

fn main() -> Result<()> {
    ensure_location_authorization()?;

    let interface = autoreleasepool(|_| unsafe { CWWiFiClient::sharedWiFiClient().interface() })
        .ok_or_else(|| anyhow!("Unable to get wifi interface"))?;

    loop {
        // extract required data
        let wifi = Wifi::current(&interface);

        // dump to file
        fs::write("/tmp/spaceport.json", wifi.to_json()?)?;

        sleep(Duration::from_millis(400))
    }
}
