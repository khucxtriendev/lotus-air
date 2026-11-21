use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use crate::ble::beacon::{AppleBeaconInfo, AppleBeaconParser, APPLE_COMPANY_ID};

pub struct AirPodsBleScanner {
    event_tx: broadcast::Sender<AppleBeaconInfo>,
}

impl AirPodsBleScanner {
    pub fn new() -> (Self, broadcast::Receiver<AppleBeaconInfo>) {
        let (tx, rx) = broadcast::channel(32);
        (Self { event_tx: tx }, rx)
    }

    /// Chạy tiến trình quét BLE background để theo dõi pin AirPods từ quảng bá Continuity
    pub async fn start_scanning(&self) {
        let tx = self.event_tx.clone();
        tokio::spawn(async move {
            info!("Bắt đầu quét Bluetooth Low Energy cho AirPods...");
            let manager = match Manager::new().await {
                Ok(m) => m,
                Err(e) => {
                    warn!("Không thể khởi tạo BLE Manager: {:?}", e);
                    return;
                }
            };

            let adapters = match manager.adapters().await {
                Ok(a) if !a.is_empty() => a,
                _ => {
                    warn!("Không tìm thấy Bluetooth adapter nào sẵn sàng.");
                    return;
                }
            };

            let central = &adapters[0];
            if let Err(e) = central.start_scan(ScanFilter::default()).await {
                error!("Lỗi khi bắt đầu BLE scan: {:?}", e);
                return;
            }

            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                if let Ok(peripherals) = central.peripherals().await {
                    for peripheral in peripherals {
                        if let Ok(Some(props)) = peripheral.properties().await {
                            if let Some(data) = props.manufacturer_data.get(&APPLE_COMPANY_ID) {
                                if let Some(info) = AppleBeaconParser::parse_manufacturer_data(APPLE_COMPANY_ID, data) {
                                    let _ = tx.send(info);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
