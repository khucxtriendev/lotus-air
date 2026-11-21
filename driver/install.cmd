@echo off
echo ========================================================
echo     LotusAir Windows KMDF Driver Installer
echo ========================================================

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [LOI] Vui long chay script nay voi quyen Administrator!
    pause
    exit /b 1
)

echo [1/3] Bat che do Windows Test Signing...
bcdedit /set testsigning on

echo [2/3] Cai dat chung chi Test Certificate...
certutil -addstore -f "Root" "%~dp0certificates\lotusair_test.cer"
certutil -addstore -f "TrustedPublisher" "%~dp0certificates\lotusair_test.cer"

echo [3/3] Dang ky Driver Service...
sc create LotusAirL2CAP type= kernel binPath= "%~dp0LotusAirDriver.sys"
sc start LotusAirL2CAP

echo.
echo ========================================================
echo Cai dat driver thanh cong! Neu can, hay khoi dong lai Windows
echo de che do Test Mode co hieu luc.
echo ========================================================
pause
