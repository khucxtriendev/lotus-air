use thiserror::Error;
use tracing::{debug, info};

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
// FILE_DEVICE_UNKNOWN = 0x00000022
// METHOD_BUFFERED = 0
// FILE_ANY_ACCESS = 0
pub const IOCTL_LOTUSAIR_CONNECT: u32 = 0x00222000;
pub const IOCTL_LOTUSAIR_SEND: u32 = 0x00222004;
pub const IOCTL_LOTUSAIR_RECV: u32 = 0x00222008;
pub const IOCTL_LOTUSAIR_DISCONNECT: u32 = 0x0022200C;

#[repr(C, packed)]
pub struct ConnectL2capRequest {
    pub bt_address: u64,
    pub psm: u16,
}

pub struct LotusAirDriverBridge {
    #[cfg(windows)]
    handle: Option<windows::Win32::Foundation::HANDLE>,
    is_connected: bool,
}

impl LotusAirDriverBridge {
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            handle: None,
            is_connected: false,
        }
    }

    /// Khởi tạo và mở giao tiếp với Windows KMDF Driver \\.\LotusAirL2CAP
    pub fn open(&mut self) -> Result<()> {
        #[cfg(windows)]
        {
            use windows::core::PCWSTR;
            use windows::Win32::Foundation::INVALID_HANDLE_VALUE;
            use windows::Win32::Storage::FileSystem::{
                CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_NONE, OPEN_EXISTING,
            };

            let device_path: Vec<u16> = r"\\.\LotusAirL2CAP"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let handle = unsafe {
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

            match handle {
                Ok(h) if h != INVALID_HANDLE_VALUE => {
                    info!("Đã kết nối thành công tới LotusAir KMDF Driver!");
                    self.handle = Some(h);
                    Ok(())
                }
                _ => {
                    warn!("Không tìm thấy LotusAir KMDF Driver (chưa cài đặt hoặc đang ở chế độ dev).");
                    Err(DriverError::DriverNotFound)
                }
            }
        }

        #[cfg(not(windows))]
        {
            debug!("Chạy ngoài Windows, kích hoạt chế độ Driver Bridge Simulation");
            Ok(())
        }
    }

    /// Gửi yêu cầu driver mở kênh L2CAP tới AirPods với PSM 0x1001
    pub fn connect_l2cap(&mut self, bt_address: u64, psm: u16) -> Result<()> {
        info!("Yêu cầu Driver mở kết nối L2CAP tới 0x{:012X} trên PSM 0x{:04X}", bt_address, psm);

        #[cfg(windows)]
        {
            use windows::Win32::System::IO::DeviceIoControl;
            if let Some(handle) = self.handle {
                let req = ConnectL2capRequest { bt_address, psm };
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
                    return Ok(());
                } else {
                    return Err(DriverError::IoctlFailed("IOCTL_LOTUSAIR_CONNECT thất bại".into()));
                }
            }
            Err(DriverError::DriverNotFound)
        }

        #[cfg(not(windows))]
        {
            self.is_connected = true;
            Ok(())
        }
    }

    /// Gửi packet AAP qua driver
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
                    return Err(DriverError::IoctlFailed("Gửi packet qua driver thất bại".into()));
                }
            }
            Err(DriverError::DriverNotFound)
        }

        #[cfg(not(windows))]
        {
            debug!("Mock Driver send packet len: {}", data.len());
            Ok(())
        }
    }

    /// Nhận packet AAP từ driver
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
                    return Err(DriverError::IoctlFailed("Nhận packet qua driver thất bại".into()));
                }
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
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::CloseHandle;
            if let Some(handle) = self.handle.take() {
                unsafe { let _ = CloseHandle(handle); }
            }
        }
        Ok(())
    }
}
