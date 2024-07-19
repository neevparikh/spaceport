use anyhow::{anyhow, Result};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::ProtocolObject;
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_core_location::{CLLocationManager, CLLocationManagerDelegate};
use objc2_core_wlan::{CWInterface, CWWiFiClient};
use objc2_foundation::{NSObject, NSObjectProtocol};
use serde::Serialize;

declare_class!(
    struct LocationManagerDelegate;

    unsafe impl ClassType for LocationManagerDelegate {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "LocationManagerDelegate";
    }

    impl DeclaredClass for LocationManagerDelegate {}

    unsafe impl LocationManagerDelegate {
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(());
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for LocationManagerDelegate {}

    unsafe impl CLLocationManagerDelegate for LocationManagerDelegate {}
);

impl LocationManagerDelegate {
    pub fn new() -> Retained<Self> {
        unsafe { msg_send_id![Self::alloc(), init] }
    }
}

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
    unsafe fn from_interface(interface: Retained<CWInterface>) -> Self {
        Wifi {
            ssid: interface.ssid().map(|s| s.to_string()),
            power: interface.powerOn(),
            rssi: interface.rssiValue(),
            noise: interface.noiseMeasurement(),
            tx_rate: interface.transmitRate(),
            tx_power: interface.transmitPower(),
        }
    }
}

fn main() -> Result<()> {
    let wifi = unsafe {
        let manager = CLLocationManager::new();
        let delegate = LocationManagerDelegate::new();
        let protocol = ProtocolObject::from_ref(delegate.as_ref());

        manager.setDelegate(Some(protocol));
        manager.startUpdatingLocation();

        dbg!(manager.delegate());

        let interface = CWWiFiClient::sharedWiFiClient().interface();
        interface.map(|interface| Wifi::from_interface(interface))
    }
    .ok_or(anyhow!("Unable to get wifi interface"))?;
    let json = serde_json::to_string_pretty(&wifi)?;
    println!("{}", json);
    Ok(())
}
