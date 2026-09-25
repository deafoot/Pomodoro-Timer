@echo off
chcp 65001 >nul
setlocal
cd /d "%~dp0"

set "EXE=target\release\pomodoro-timer.exe"
if /i "%~1"=="rebuild" set "EXE="

if exist "%EXE%" goto run

echo [番茄钟] 正在编译（离线模式，约 10 秒）...
where cargo >nul 2>nul
if errorlevel 1 (
    echo [番茄钟] 未检测到 Rust 工具链，请先安装：https://rustup.rs
    pause
    exit /b 1
)
cargo build --release --offline
if errorlevel 1 (
    echo [番茄钟] 编译失败，请查看上面的错误信息。
    pause
    exit /b 1
)

:run
start "" "%EXE%"
exit /b 0
