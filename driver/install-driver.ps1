# LotusAir L2CAP Kernel Driver Installer (WDM Bridge cho Apple AAP PSM 0x1001)
# Chạy bằng quyền Administrator: powershell -ExecutionPolicy Bypass -File .\driver\install-driver.ps1

$ErrorActionPreference = "Stop"

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[!] Dang xin quyen Administrator de cai dat LotusAir L2CAP Kernel Driver..." -ForegroundColor Yellow
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    exit 0
}

$driverPath = Join-Path $PSScriptRoot "LotusAirDriver.sys"
$newDriverPath = Join-Path $PSScriptRoot "LotusAirDriver_new.sys"

# Dừng dịch vụ cũ trước khi ghi đè file .sys
sc.exe stop LotusAirL2CAP 2>&1 | Out-Null
Start-Sleep -Milliseconds 300
if (Test-Path $newDriverPath) {
    Move-Item -Path $newDriverPath -Destination $driverPath -Force
}
if (-not (Test-Path $driverPath)) {
    Write-Error "Khong tim thay file driver tai: $driverPath"
    exit 1
}

# 1. Tạo và tin cậy chứng chỉ Test Signing cho LotusAirDriver.sys
$cert = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Where-Object { $_.Subject -eq "CN=LotusAir Kernel Driver" } | Select-Object -First 1
if (-not $cert) {
    $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=LotusAir Kernel Driver" -CertStoreLocation "Cert:\CurrentUser\My"
}
$tempCer = Join-Path $env:TEMP "LotusAirDriver.cer"
Export-Certificate -Cert $cert -FilePath $tempCer | Out-Null
Import-Certificate -FilePath $tempCer -CertStoreLocation "Cert:\LocalMachine\Root" | Out-Null
Import-Certificate -FilePath $tempCer -CertStoreLocation "Cert:\LocalMachine\TrustedPublisher" | Out-Null
Set-AuthenticodeSignature -FilePath $driverPath -Certificate $cert | Out-Null

# 2. Bật Test Signing nếu chưa bật
bcdedit.exe /set testsigning on | Out-Null

# 3. Đăng ký và khởi chạy dịch vụ Kernel Driver LotusAirL2CAP
sc.exe stop LotusAirL2CAP 2>&1 | Out-Null
sc.exe delete LotusAirL2CAP 2>&1 | Out-Null
sc.exe create LotusAirL2CAP type= kernel start= auto binPath= "$driverPath" | Out-Null
$startOut = sc.exe start LotusAirL2CAP 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] LotusAir L2CAP Kernel Driver (\Device\LotusAirL2CAP) da khoi chay thanh cong!" -ForegroundColor Green
} else {
    Write-Host "[!] Da dang ky LotusAirL2CAP va bat Test Signing. Vui long Restart Windows 1 lan de Windows nap Kernel Driver L2CAP PSM 0x1001." -ForegroundColor Yellow
    Write-Host $startOut
}
