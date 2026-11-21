# Tắt Windows Test Mode (Test Signing)
# Lưu ý: Sau khi tắt Test Mode và khởi động lại, Windows sẽ chặn nạp Kernel Driver tự ký LotusAirDriver.sys (kênh L2CAP PSM 0x1001).
# Các tính năng user-mode (Pin BLE, Cướp kết nối A2DP từ iPhone, Đầu ra âm thanh, Media/Volume) vẫn hoạt động bình thường.

$ErrorActionPreference = "Stop"

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[!] Dang xin quyen Administrator de tat Test Mode..." -ForegroundColor Yellow
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    exit 0
}

bcdedit.exe /set testsigning off
Write-Host "[OK] Da tat Windows Test Mode (testsigning off). Khoi dong lai may de mat chu Test Mode o goc man hinh." -ForegroundColor Green
