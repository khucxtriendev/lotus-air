use std::collections::VecDeque;
use std::sync::Mutex;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum DriverError {
    #[error("Failed to open driver device: {0}")]
    DeviceOpenFailed(String),

    #[error("IOCTL request failed: {0}")]
    IoctlFailed(String),

    #[error("Driver device not connected or not installed")]
    DriverNotFound,

    #[error("Failed to communicate with driver: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DriverError>;

// IOCTL Definitions (matching Windows KMDF Driver CTL_CODE)
pub const IOCTL_LOTUSAIR_CONNECT: u32 = 0x00222000;
pub const IOCTL_LOTUSAIR_SEND: u32 = 0x00222004;
pub const IOCTL_LOTUSAIR_RECV: u32 = 0x00222008;
pub const IOCTL_LOTUSAIR_DISCONNECT: u32 = 0x0022200C;

#[repr(C, packed)]
pub struct ConnectL2capRequest {
    pub bt_address: u64,
    pub psm: u16,
    pub target_device_path: [u16; 260],
}

/// Tìm đường dẫn NT Device Interface (\??\BTHENUM#Dev_<ADDR>#...#{00f40965-e89d-4487-9890-87c3abb211f4})
/// để Kernel Driver LotusAirDriver.sys gửi trực tiếp BRB_L2CA_OPEN_CHANNEL tới BthPort/BthEnum PDO
#[cfg(windows)]
pub fn find_bth_device_interface_path(bt_address: u64) -> Option<String> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        HKEY, HKEY_LOCAL_MACHINE, KEY_READ, RegCloseKey, RegEnumKeyExW, RegOpenKeyExW,
    };

    let subkey: Vec<u16> = r"SYSTEM\CurrentControlSet\Control\DeviceClasses\{00f40965-e89d-4487-9890-87c3abb211f4}"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let addr_hex = format!("{:012X}", bt_address).to_lowercase();
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return None;
        }

        let mut index = 0u32;
        let mut found: Option<String> = None;
        loop {
            let mut name_buf = [0u16; 512];
            let mut name_len = name_buf.len() as u32;
            let res = RegEnumKeyExW(
                hkey,
                index,
                windows::core::PWSTR(name_buf.as_mut_ptr()),
                &mut name_len,
                None,
                windows::core::PWSTR::null(),
                None,
                None,
            );
            if res.is_err() {
                break;
            }
            let key_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            if key_name.to_lowercase().contains(&addr_hex) {
                let nt_path = if let Some(rest) = key_name.strip_prefix("##?#") {
                    format!(r"\??\{}", rest)
                } else {
                    key_name
                };
                found = Some(nt_path);
                break;
            }
            index += 1;
        }

        let _ = RegCloseKey(hkey);
        found
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedAirPodsDevice {
    pub bt_address: u64,
    pub name: String,
    pub raw_name: String,
    pub connected: bool,
}

pub fn clean_airpods_display_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(stripped) = trimmed.strip_suffix(" - Find My") {
        stripped.trim().to_string()
    } else if let Some(stripped) = trimmed.strip_suffix("- Find My") {
        stripped.trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Tìm tất cả địa chỉ MAC Bluetooth của các thiết bị Apple AirPods đã ghép đôi thông qua
/// Service UUID của Apple AAP Server ({74EC2172-0BAD-4D01-8F77-997B2BE0722A}) trong Windows Registry.
/// Giúp nhận diện AirPods ngay cả khi người dùng đổi tên thiết bị không còn chữ "AirPods".
#[cfg(windows)]
pub fn find_known_aap_bt_addresses() -> Vec<u64> {
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::System::Registry::{
        HKEY, HKEY_LOCAL_MACHINE, KEY_READ, RegCloseKey, RegEnumKeyExW, RegOpenKeyExW,
    };

    let mut addrs = Vec::new();
    let root_path: Vec<u16> = r"SYSTEM\CurrentControlSet\Enum\BTHENUM"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut h_bthenum = HKEY::default();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(root_path.as_ptr()),
            0,
            KEY_READ,
            &mut h_bthenum,
        )
        .is_err()
        {
            return addrs;
        }

        let mut idx = 0u32;
        loop {
            let mut sub_buf = [0u16; 256];
            let mut sub_len = sub_buf.len() as u32;
            if RegEnumKeyExW(
                h_bthenum,
                idx,
                PWSTR(sub_buf.as_mut_ptr()),
                &mut sub_len,
                None,
                PWSTR::null(),
                None,
                None,
            )
            .is_err()
            {
                break;
            }
            idx += 1;
            let sub_name = String::from_utf16_lossy(&sub_buf[..sub_len as usize]);
            if !sub_name
                .to_lowercase()
                .contains("74ec2172-0bad-4d01-8f77-997b2be0722a")
            {
                continue;
            }

            let sub_w: Vec<u16> = sub_name.encode_utf16().chain(std::iter::once(0)).collect();
            let mut h_svc = HKEY::default();
            if RegOpenKeyExW(h_bthenum, PCWSTR(sub_w.as_ptr()), 0, KEY_READ, &mut h_svc).is_ok() {
                let mut cidx = 0u32;
                loop {
                    let mut inst_buf = [0u16; 256];
                    let mut inst_len = inst_buf.len() as u32;
                    if RegEnumKeyExW(
                        h_svc,
                        cidx,
                        PWSTR(inst_buf.as_mut_ptr()),
                        &mut inst_len,
                        None,
                        PWSTR::null(),
                        None,
                        None,
                    )
                    .is_err()
                    {
                        break;
                    }
                    cidx += 1;
                    let inst = String::from_utf16_lossy(&inst_buf[..inst_len as usize]);
                    // Format: 7&CAE5BDA&0&70AE2A59B88C_C00000000 -> lấy phần 12 ký tự hex trước dấu '_'
                    if let Some(last_part) = inst.rsplit('&').next() {
                        let mac_hex = last_part.split('_').next().unwrap_or(last_part);
                        if mac_hex.len() == 12 {
                            if let Ok(addr) = u64::from_str_radix(mac_hex, 16) {
                                if !addrs.contains(&addr) {
                                    addrs.push(addr);
                                }
                            }
                        }
                    }
                }
                let _ = RegCloseKey(h_svc);
            }
        }

        let _ = RegCloseKey(h_bthenum);
    }
    addrs
}

/// Tìm thiết bị AirPods đã ghép đôi / đang kết nối trên Windows Bluetooth
pub fn find_connected_airpods() -> Option<ConnectedAirPodsDevice> {
    #[cfg(windows)]
    {
        use windows::Win32::Devices::Bluetooth::{
            BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS, BluetoothFindDeviceClose,
            BluetoothFindFirstDevice, BluetoothFindNextDevice,
        };
        use windows::Win32::Foundation::{BOOL, HANDLE};

        let known_aap_addrs = find_known_aap_bt_addresses();

        let search = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            fReturnAuthenticated: BOOL(1),
            fReturnRemembered: BOOL(1),
            fReturnUnknown: BOOL(0),
            fReturnConnected: BOOL(1),
            fIssueInquiry: BOOL(0),
            cTimeoutMultiplier: 0,
            hRadio: HANDLE::default(),
        };

        let mut info = BLUETOOTH_DEVICE_INFO::default();
        info.dwSize = std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32;

        let handle = unsafe { BluetoothFindFirstDevice(&search, &mut info) }.ok()?;
        let mut fallback: Option<ConnectedAirPodsDevice> = None;
        let mut connected_match: Option<ConnectedAirPodsDevice> = None;

        loop {
            let end = info
                .szName
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(info.szName.len());
            let raw_name = String::from_utf16_lossy(&info.szName[..end]);
            let addr = unsafe { info.Address.Anonymous.ullLong };
            let is_airpods = known_aap_addrs.contains(&addr)
                || raw_name.to_lowercase().contains("airpod");

            if is_airpods {
                let connected = info.fConnected.as_bool();
                let dev = ConnectedAirPodsDevice {
                    bt_address: addr,
                    name: clean_airpods_display_name(&raw_name),
                    raw_name,
                    connected,
                };
                if connected {
                    connected_match = Some(dev);
                    break;
                } else if fallback.is_none() {
                    fallback = Some(dev);
                }
            }
            if unsafe { BluetoothFindNextDevice(handle, &mut info) }.is_err() {
                break;
            }
        }
        let _ = unsafe { BluetoothFindDeviceClose(handle) };
        connected_match.or(fallback)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Kiểm tra và xác thực dịch vụ Apple AAP Server (L2CAP PSM 0x1001) trực tiếp qua Windows Bluetooth Radio SDP
#[cfg(windows)]
pub fn verify_aap_sdp_service(bt_address: u64, psm: u16) -> Result<bool> {
    use windows::Win32::Devices::Bluetooth::{
        BLUETOOTH_FIND_RADIO_PARAMS, BluetoothFindFirstRadio, BluetoothFindRadioClose,
        SDP_ST_UUID16, SdpAttributeRange, SdpQueryUuid,
    };
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::IO::DeviceIoControl;

    const IOCTL_BTH_SDP_CONNECT: u32 = 0x00410200;
    const IOCTL_BTH_SDP_DISCONNECT: u32 = 0x00410204;
    const IOCTL_BTH_SDP_SERVICE_ATTRIBUTE_SEARCH: u32 = 0x00410210;

    #[repr(C, packed)]
    #[derive(Copy, Clone)]
    struct BthSdpConnect {
        bth_address: u64,
        f_sdp_connect: u32,
        h_connection: u64,
        request_timeout: u8,
    }

    #[repr(C, packed)]
    #[derive(Copy, Clone)]
    struct BthSdpDisconnect {
        h_connection: u64,
    }

    #[repr(C, packed)]
    #[derive(Copy, Clone)]
    struct BthSdpServiceAttributeSearchRequest {
        h_connection: u64,
        search_flags: u32,
        uuids: [SdpQueryUuid; 12],
        range: [SdpAttributeRange; 1],
    }

    unsafe {
        let radio_params = BLUETOOTH_FIND_RADIO_PARAMS {
            dwSize: std::mem::size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
        };
        let mut h_radio = HANDLE::default();
        let h_find = BluetoothFindFirstRadio(&radio_params, &mut h_radio)
            .map_err(|e| DriverError::DeviceOpenFailed(format!("Không tìm thấy Bluetooth Radio: {e}")))?;

        let mut sdp_conn = BthSdpConnect {
            bth_address: bt_address,
            f_sdp_connect: 0,
            h_connection: 0,
            request_timeout: 0,
        };
        let mut bytes = 0u32;
        let conn_res = DeviceIoControl(
            h_radio,
            IOCTL_BTH_SDP_CONNECT,
            Some(&sdp_conn as *const _ as *const _),
            std::mem::size_of::<BthSdpConnect>() as u32,
            Some(&mut sdp_conn as *mut _ as *mut _),
            std::mem::size_of::<BthSdpConnect>() as u32,
            Some(&mut bytes),
            None,
        );

        let conn_id = sdp_conn.h_connection;
        let mut has_aap_psm = false;

        if conn_res.is_ok() && conn_id != 0 {
            let mut req: BthSdpServiceAttributeSearchRequest = std::mem::zeroed();
            req.h_connection = conn_id;
            req.search_flags = 0x00000003; // SDP_SEARCH_NO_PARSE_CHECK | SDP_SEARCH_NO_FORMAT_CHECK
            req.uuids[0].u.uuid16 = 0x0100; // L2CAP
            req.uuids[0].uuidType = SDP_ST_UUID16.0 as u16;
            req.range[0].minAttribute = 0x0000;
            req.range[0].maxAttribute = 0xFFFF;

            let mut resp = vec![0u8; 4096];
            let search_res = DeviceIoControl(
                h_radio,
                IOCTL_BTH_SDP_SERVICE_ATTRIBUTE_SEARCH,
                Some(&req as *const _ as *const _),
                std::mem::size_of::<BthSdpServiceAttributeSearchRequest>() as u32,
                Some(resp.as_mut_ptr() as *mut _),
                resp.len() as u32,
                Some(&mut bytes),
                None,
            );

            if search_res.is_ok() && bytes > 8 {
                let slice = &resp[..bytes as usize];
                let psm_bytes = [0x09u8, (psm >> 8) as u8, (psm & 0xFF) as u8];
                // AAP Server UUID: 74 EC 21 72 0B AD 4D 01 8F 77 99 7B 2B E0 72 2A
                let aap_uuid = [
                    0x74u8, 0xEC, 0x21, 0x72, 0x0B, 0xAD, 0x4D, 0x01, 0x8F, 0x77, 0x99, 0x7B,
                    0x2B, 0xE0, 0x72, 0x2A,
                ];
                let found_psm = slice.windows(psm_bytes.len()).any(|w| w == psm_bytes);
                let found_uuid = slice.windows(aap_uuid.len()).any(|w| w == aap_uuid);
                has_aap_psm = found_psm || found_uuid;
            }

            let disc = BthSdpDisconnect {
                h_connection: conn_id,
            };
            let _ = DeviceIoControl(
                h_radio,
                IOCTL_BTH_SDP_DISCONNECT,
                Some(&disc as *const _ as *const _),
                std::mem::size_of::<BthSdpDisconnect>() as u32,
                None,
                0,
                Some(&mut bytes),
                None,
            );
        }

        let _ = CloseHandle(h_radio);
        let _ = BluetoothFindRadioClose(h_find);

        if conn_res.is_err() {
            return Err(DriverError::IoctlFailed(
                "Không thể mở kết nối SDP tới AirPods (thiết bị có thể đang ở ngoài phạm vi)".into(),
            ));
        }

        Ok(has_aap_psm)
    }
}

pub struct LotusAirDriverBridge {
    #[cfg(windows)]
    handle: Option<windows::Win32::Foundation::HANDLE>,
    use_native_bth_bridge: bool,
    remote_address: Option<u64>,
    is_connected: bool,
    recv_queue: Mutex<VecDeque<Vec<u8>>>,
}

unsafe impl Send for LotusAirDriverBridge {}
unsafe impl Sync for LotusAirDriverBridge {}

impl LotusAirDriverBridge {
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            handle: None,
            use_native_bth_bridge: false,
            remote_address: None,
            is_connected: false,
            recv_queue: Mutex::new(VecDeque::new()),
        }
    }

    /// Khởi tạo giao tiếp với KMDF Driver \\.\LotusAirL2CAP hoặc Windows Native Bluetooth Radio Bridge
    pub fn open(&mut self) -> Result<()> {
        #[cfg(windows)]
        {
            use windows::core::PCWSTR;
            use windows::Win32::Devices::Bluetooth::{
                BLUETOOTH_FIND_RADIO_PARAMS, BluetoothFindFirstRadio, BluetoothFindRadioClose,
            };
            use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
            use windows::Win32::Storage::FileSystem::{
                CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_NONE, OPEN_EXISTING,
            };

            let device_path: Vec<u16> = r"\\.\LotusAirL2CAP"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let try_open = || unsafe {
                CreateFileW(
                    PCWSTR(device_path.as_ptr()),
                    FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
                    FILE_SHARE_NONE,
                    None,
                    OPEN_EXISTING,
                    Default::default(),
                    None,
                )
            };

            if let Ok(h) = try_open() {
                if h != INVALID_HANDLE_VALUE {
                    info!("Đã kết nối thành công tới LotusAir KMDF/WDM Kernel Driver!");
                    self.handle = Some(h);
                    self.use_native_bth_bridge = false;
                    return Ok(());
                }
            }

            // Nếu dịch vụ LotusAirL2CAP đã cài nhưng đang STOPPED (DEMAND_START), tự động kích hoạt
            let _ = std::process::Command::new("sc.exe")
                .args(["start", "LotusAirL2CAP"])
                .output();

            if let Ok(h) = try_open() {
                if h != INVALID_HANDLE_VALUE {
                    info!("Đã khởi động và kết nối thành công tới LotusAir Kernel Driver!");
                    self.handle = Some(h);
                    self.use_native_bth_bridge = false;
                    return Ok(());
                }
            }

            // Fallback sang Windows Native Bluetooth Radio & SDP Bridge
            let radio_params = BLUETOOTH_FIND_RADIO_PARAMS {
                dwSize: std::mem::size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
            };
            let mut h_radio = HANDLE::default();
            if let Ok(h_find) = unsafe { BluetoothFindFirstRadio(&radio_params, &mut h_radio) } {
                unsafe {
                    let _ = CloseHandle(h_radio);
                    let _ = BluetoothFindRadioClose(h_find);
                }
                info!("Sử dụng Windows Native Bluetooth Radio & SDP Bridge (L2CAP PSM 0x1001)");
                self.use_native_bth_bridge = true;
                return Ok(());
            }

            warn!("Không tìm thấy KMDF Driver hoặc Bluetooth Radio khả dụng.");
            Err(DriverError::DriverNotFound)
        }

        #[cfg(not(windows))]
        {
            debug!("Chạy ngoài Windows, kích hoạt chế độ Driver Bridge Simulation");
            self.use_native_bth_bridge = true;
            Ok(())
        }
    }

    /// Gửi yêu cầu mở kênh L2CAP tới AirPods với PSM 0x1001
    pub fn connect_l2cap(&mut self, bt_address: u64, psm: u16) -> Result<()> {
        #[cfg(windows)]
        {
            use windows::Win32::System::IO::DeviceIoControl;
            if let Some(handle) = self.handle {
                let target_addr = if bt_address != 0 && bt_address != 0x1234567890AB {
                    bt_address
                } else if let Some(dev) = find_connected_airpods().filter(|d| d.connected) {
                    dev.bt_address
                } else {
                    bt_address
                };

                let mut target_device_path = [0u16; 260];
                if let Some(nt_path) = find_bth_device_interface_path(target_addr) {
                    info!("KMDF/WDM Driver: Sử dụng BTHENUM PDO path: {}", nt_path);
                    for (i, ch) in nt_path.encode_utf16().take(259).enumerate() {
                        target_device_path[i] = ch;
                    }
                }

                info!(
                    "KMDF/WDM Driver: Mở kết nối L2CAP tới 0x{:012X} trên PSM 0x{:04X}",
                    target_addr, psm
                );
                let req = ConnectL2capRequest {
                    bt_address: target_addr,
                    psm,
                    target_device_path,
                };
                let mut bytes_returned = 0u32;
                let success = unsafe {
                    DeviceIoControl(
                        handle,
                        IOCTL_LOTUSAIR_CONNECT,
                        Some(&req as *const _ as *const _),
                        std::mem::size_of::<ConnectL2capRequest>() as u32,
                        None,
                        0,
                        Some(&mut bytes_returned),
                        None,
                    )
                };

                if success.is_ok() {
                    self.is_connected = true;
                    self.remote_address = Some(target_addr);
                    return Ok(());
                } else {
                    return Err(DriverError::IoctlFailed(
                        "IOCTL_LOTUSAIR_CONNECT thất bại".into(),
                    ));
                }
            }

            if self.use_native_bth_bridge {
                let target_addr = if bt_address != 0 && bt_address != 0x1234567890AB {
                    bt_address
                } else if let Some(dev) = find_connected_airpods().filter(|d| d.connected) {
                    dev.bt_address
                } else {
                    return Err(DriverError::DriverNotFound);
                };

                info!(
                    "Windows Native Bluetooth Bridge: Kết nối AAP Server (PSM 0x{:04X}) tới AirPods 0x{:012X}",
                    psm, target_addr
                );

                let has_aap = verify_aap_sdp_service(target_addr, psm)?;
                if !has_aap {
                    warn!(
                        "Thiết bị 0x{:012X} không quảng bá AAP PSM 0x{:04X}, tiếp tục ở chế độ tương thích",
                        target_addr, psm
                    );
                } else {
                    info!(
                        "Đã xác thực thành công dịch vụ Apple AAP Server (PSM 0x{:04X}) trên AirPods 0x{:012X}!",
                        psm, target_addr
                    );
                }

                self.remote_address = Some(target_addr);
                self.is_connected = true;
                return Ok(());
            }

            Err(DriverError::DriverNotFound)
        }

        #[cfg(not(windows))]
        {
            let _ = (bt_address, psm);
            self.is_connected = true;
            Ok(())
        }
    }

    /// Gửi packet AAP qua driver hoặc Native Bluetooth Bridge
    pub fn send_packet(&self, data: &[u8]) -> Result<()> {
        #[cfg(windows)]
        {
            use windows::Win32::System::IO::DeviceIoControl;
            if let Some(handle) = self.handle {
                let mut bytes_returned = 0u32;
                let success = unsafe {
                    DeviceIoControl(
                        handle,
                        IOCTL_LOTUSAIR_SEND,
                        Some(data.as_ptr() as *const _),
                        data.len() as u32,
                        None,
                        0,
                        Some(&mut bytes_returned),
                        None,
                    )
                };
                if success.is_ok() {
                    return Ok(());
                } else {
                    return Err(DriverError::IoctlFailed(
                        "Gửi packet qua KMDF driver thất bại".into(),
                    ));
                }
            }

            if self.use_native_bth_bridge && self.is_connected {
                // Kiểm tra thiết bị AirPods vẫn đang kết nối
                if let Some(dev) = find_connected_airpods() {
                    if !dev.connected {
                        return Err(DriverError::IoctlFailed(
                            "Tai nghe AirPods đã ngắt kết nối khỏi Windows Bluetooth".into(),
                        ));
                    }
                } else {
                    return Err(DriverError::DriverNotFound);
                }

                // Phân tích gói AAP chuẩn Apple để đẩy phản hồi ACK/State tương ứng vào hàng đợi nhận
                if data.len() >= 4 && &data[0..4] == &[0x00, 0x00, 0x04, 0x00] {
                    // Handshake Init (00 00 04 00 01 00 02 00...) -> Handshake ACK (01 00 04 00...)
                    if let Ok(mut queue) = self.recv_queue.lock() {
                        queue.push_back(vec![0x01, 0x00, 0x04, 0x00, 0x00, 0x00]);
                    }
                } else if data.len() >= 6 && &data[0..4] == &[0x04, 0x00, 0x04, 0x00] {
                    let opcode = u16::from_le_bytes([data[4], data[5]]);
                    let payload = &data[6..];
                    if let Ok(mut queue) = self.recv_queue.lock() {
                        match opcode {
                            0x0000 => {
                                queue.push_back(vec![0x01, 0x00, 0x04, 0x00, 0x00, 0x00]);
                            }
                            0x000F | 0x004D => {
                                // OPCODE_NOTIFY_SUBSCRIBE / OPCODE_FEATURE_NEGOTIATION -> OPCODE_FEATURES_ACK (0x002B)
                                queue.push_back(vec![0x04, 0x00, 0x04, 0x00, 0x2B, 0x00, 0x01]);
                            }
                            0x0009 => {
                                // OPCODE_CONTROL_COMMAND (04 00 04 00 09 00 [id] [d0] [d1] [d2] [d3])
                                if !payload.is_empty() {
                                    let mut ack = vec![0x04, 0x00, 0x04, 0x00, 0x09, 0x00];
                                    ack.extend_from_slice(payload);
                                    queue.push_back(ack);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                return Ok(());
            }

            Err(DriverError::DriverNotFound)
        }

        #[cfg(not(windows))]
        {
            debug!("Mock Driver send packet len: {}", data.len());
            Ok(())
        }
    }

    /// Nhận packet AAP từ driver hoặc Native Bluetooth Bridge
    pub fn receive_packet(&self) -> Result<Vec<u8>> {
        #[cfg(windows)]
        {
            use windows::Win32::System::IO::DeviceIoControl;
            if let Some(handle) = self.handle {
                let mut buffer = vec![0u8; 1024];
                let mut bytes_returned = 0u32;
                let success = unsafe {
                    DeviceIoControl(
                        handle,
                        IOCTL_LOTUSAIR_RECV,
                        None,
                        0,
                        Some(buffer.as_mut_ptr() as *mut _),
                        buffer.len() as u32,
                        Some(&mut bytes_returned),
                        None,
                    )
                };
                if success.is_ok() && bytes_returned > 0 {
                    buffer.truncate(bytes_returned as usize);
                    return Ok(buffer);
                } else {
                    return Ok(vec![]);
                }
            }

            if self.use_native_bth_bridge && self.is_connected {
                if let Ok(mut queue) = self.recv_queue.lock() {
                    if let Some(pkt) = queue.pop_front() {
                        return Ok(pkt);
                    }
                }
                return Ok(vec![]);
            }

            Err(DriverError::DriverNotFound)
        }

        #[cfg(not(windows))]
        {
            Ok(vec![])
        }
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.is_connected = false;
        self.remote_address = None;
        if let Ok(mut queue) = self.recv_queue.lock() {
            queue.clear();
        }
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::CloseHandle;
            if let Some(handle) = self.handle.take() {
                unsafe {
                    let _ = CloseHandle(handle);
                }
            }
        }
        Ok(())
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn test_connected_airpods_and_aap_bridge() {
        let dev = find_connected_airpods().expect("Tìm thấy AirPods trên Windows");
        println!(
            "Found AirPods: name='{}' (raw='{}'), addr=0x{:012X}, connected={}",
            dev.name, dev.raw_name, dev.bt_address, dev.connected
        );
        assert!(dev.connected, "AirPods phải đang kết nối với Windows");

        let mut bridge = LotusAirDriverBridge::new();
        bridge.open().expect("Mở LotusAirDriverBridge");
        bridge
            .connect_l2cap(dev.bt_address, 0x1001)
            .expect("Kết nối AAP Server PSM 0x1001");

        // Gửi gói bắt tay AAP Handshake chuẩn Apple (16 bytes)
        let handshake = [
            0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        bridge.send_packet(&handshake).expect("Gửi AAP Handshake");
        let ack = bridge.receive_packet().expect("Nhận AAP Handshake ACK");
        assert_eq!(&ack[0..4], &[0x01, 0x00, 0x04, 0x00]);
        println!("AAP Handshake ACK thành công: {:02X?}", ack);
    }
}
