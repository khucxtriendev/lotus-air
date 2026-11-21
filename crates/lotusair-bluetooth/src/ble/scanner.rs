use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::StreamExt;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use crate::ble::beacon::{AppleBeaconInfo, AppleBeaconParser, APPLE_COMPANY_ID};

pub struct AirPodsBleScanner {
    event_tx: broadcast::Sender<AppleBeaconInfo>,
}

impl AirPodsBleScanner {
    pub fn new() -> (Self, broadcast::Receiver<AppleBeaconInfo>) {
        let (tx, rx) = broadcast::channel(64);
        (Self { event_tx: tx }, rx)
    }

    /// Chạy tiến trình nghe sự kiện BLE thời gian thực (ManufacturerDataAdvertisement)
    /// để bắt tức thì khoảnh khắc mở nắp hộp / rút tai nghe khỏi hộp giống hệt iPhone.
    pub async fn start_scanning(&self) {
        let tx = self.event_tx.clone();
        tokio::spawn(async move {
            info!("Bắt đầu lắng nghe BLE Continuity (0x07) thời gian thực cho AirPods...");
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

            let central = adapters.into_iter().next().unwrap();
            let mut events = match central.events().await {
                Ok(stream) => stream,
                Err(e) => {
                    error!("Không đăng ký được BLE event stream: {:?}", e);
                    return;
                }
            };

            if let Err(e) = central.start_scan(ScanFilter::default()).await {
                error!("Lỗi khi bắt đầu BLE scan: {:?}", e);
                return;
            }

            while let Some(event) = events.next().await {
                if let CentralEvent::ManufacturerDataAdvertisement {
                    id,
                    manufacturer_data,
                } = event
                {
                    if let Some(data) = manufacturer_data.get(&APPLE_COMPANY_ID) {
                        if let Some(mut beacon) =
                            AppleBeaconParser::parse_manufacturer_data(APPLE_COMPANY_ID, data)
                        {
                            let rssi = match central.peripheral(&id).await {
                                Ok(p) => p.properties().await.ok().flatten().and_then(|pr| pr.rssi),
                                Err(_) => None,
                            };
                            // Chỉ nhận AirPods ở gần laptop (RSSI >= -78 dBm) để tránh bắt nhầm tai nghe phòng bên
                            if let Some(dbm) = rssi {
                                if dbm < -78 {
                                    continue;
                                }
                            }
                            beacon.rssi = rssi;
                            let _ = tx.send(beacon);
                        }
                    }
                }
            }
        });
    }
}
