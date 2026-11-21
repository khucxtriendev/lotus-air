# 🪷 LotusAir

> **Apple AirPods (đặc biệt AirPods 4 ANC) Controller cho Windows** viết hoàn toàn bằng **Rust** + **Tauri v2** + **Svelte**, giao diện **Apple Liquid Glass** đẳng cấp.

---

## ✨ Điểm Nổi Bật

1. **Chuẩn thiết kế Apple Liquid Glass**:
   - Sử dụng hiệu ứng khúc xạ ánh sáng (refraction) và dynamic blur `backdrop-filter: blur(40px)`.
   - Viền specular highlight phản xạ ánh sáng tự nhiên.
   - Hỗ trợ Windows Mica / Acrylic nền trong suốt qua `window-vibrancy`.
2. **Popup kết nối 3D sống động (Bottom-Right)**:
   - Khi AirPods kết nối, tự động hiển thị popup ở góc phải bên dưới màn hình với mô hình AirPods 4 xoay xoay 3D mềm mại, hiển thị chi tiết mức pin L/R/Case và tự động đóng sau 5 giây.
3. **Đầy đủ tính năng như trên MacBook**:
   - **Noise Control**: Chống ồn chủ động (ANC), Xuyên âm (Transparency), Âm thanh thích ứng (Adaptive), Tắt (Off).
   - **In-Ear Detection**: Nhận biết tai nghe đang đeo trong tai hay đã tháo ra, tự động dừng/tiếp tục bài hát.
   - **Conversational Awareness**: Tự động hạ âm lượng nhạc khi người dùng mở miệng nói chuyện.
   - **Personalized Volume & Spatial Audio**: Quản lý âm thanh vòm không gian.
   - **Mức pin chính xác 1%**: Qua kênh Apple Accessory Protocol (AAP) L2CAP.
4. **Kiến trúc Rust mở rộng cao**:
   - Tách biệt rõ ràng giữa Core Protocol (`lotusair-core`), Bluetooth Transport (`lotusair-bluetooth`), Kernel Driver Bridge (`lotusair-driver`), và Desktop App (`src-tauri` + `frontend`).
   - Sẵn sàng mở rộng cho AirPods Pro 2, AirPods Max, hoặc các thiết bị Beats trong tương lai.

---

## 🏛️ Cấu Trúc Dự Án

```
LotusAir/
├── Cargo.toml                          # Workspace manifest (Rust 2024 edition)
├── driver/                             # Windows KMDF Kernel Driver (C)
│   ├── src/
│   │   ├── driver.c                    # Driver entry & L2CAP I/O dispatch
│   │   └── ioctl.h                     # Control codes IOCTL_LOTUSAIR_*
│   └── install.cmd                     # Script kích hoạt test signing & cài driver
├── crates/
│   ├── lotusair-core/                  # Platform-agnostic AAP protocol engine
│   │   ├── src/aap/                    # Opcodes, Packets, Commands, Parsers, State Machine
│   │   ├── src/models/                 # Device, Battery, NoiseMode, Features
│   │   └── src/crypto/                 # AES-128 BLE continuity decryption
│   ├── lotusair-driver/                # User-mode bridge tới \\.\LotusAirL2CAP
│   └── lotusair-bluetooth/             # Bluetooth Classic L2CAP + BLE scanner (btleplug)
├── src-tauri/                          # Tauri v2 Desktop App
│   ├── src/main.rs                     # Mica window effects & event loop
│   ├── src/tray.rs                     # System Tray Icon & Context Menu
│   └── src/commands.rs                 # Tauri IPC handlers
└── frontend/                           # Svelte + TypeScript + Vite
    └── src/
        ├── lib/styles/liquid-glass.css # CSS Liquid Glass Apple Style
        ├── lib/components/
        │   ├── ConnectionPopup.svelte  # Popup kết nối 3D xoay góc phải dưới
        │   └── ControlFlyout.svelte    # Control Center giống hệt macOS
        └── App.svelte
```

---

## 🚀 Hướng Dẫn Build & Chạy

### 1. Yêu cầu hệ thống
- **Rust**: Phiên bản 1.80+ (khuyên dùng 1.82+)
- **Node.js**: Phiên bản 18+ (khuyên dùng Node 20+)
- **Windows Build Tools** hoặc **Visual Studio 2022** (với C++ desktop workload)

### 2. Cài đặt Driver KMDF trên Windows
Do Windows hạn chế quyền truy cập user-mode vào cổng Bluetooth L2CAP PSM `0x1001`:
1. Mở Command Prompt (Admin) tại thư mục `driver/`.
2. Chạy `install.cmd` (script sẽ bật `bcdedit /set testsigning on` và nạp driver service).

### 3. Build & Run Desktop App
```bash
# Cài đặt frontend dependencies
cd frontend
npm install

# Chạy ứng dụng chế độ dev
cd ..
cargo tauri dev
```

---

## 📜 Giấy Phép
Dự án được phân phối dưới giấy phép **GNU General Public License v3.0 (GPL-3.0)**.
