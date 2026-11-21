@echo off
setlocal
echo ========================================================
echo     LotusAir Windows KMDF L2CAP Driver Installer
echo ========================================================

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [LOI] Vui long chay script nay voi quyen Administrator!
    pause
    exit /b 1
)

if not exist "%~dp0LotusAirDriver.sys" (
    echo [THONG BAO] Chua tim thay file LotusAirDriver.sys da build bang WDK.
    echo App LotusAir se tu dong su dung Windows Native Bluetooth Radio ^& SDP Bridge.
    exit /b 0
)

echo [1/3] Bat che do Windows Test Signing...
bcdedit /set testsigning on

if exist "%~dp0certificates\lotusair_test.cer" (
    echo [2/3] Cai dat chung chi Test Certificate...
    certutil -addstore -f "Root" "%~dp0certificates\lotusair_test.cer"
    certutil -addstore -f "TrustedPublisher" "%~dp0certificates\lotusair_test.cer"
)

echo [3/3] Dang ky Driver Service LotusAirL2CAP (PSM 0x1001)...
sc create LotusAirL2CAP type= kernel binPath= "%~dp0LotusAirDriver.sys"
sc start LotusAirL2CAP

echo.
echo ========================================================
echo Cai dat driver thanh cong!
echo ========================================================
pause
