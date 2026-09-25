use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
use windows::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, SetForegroundWindow, SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_NOZORDER, SW_SHOW,
    WINDOW_EX_STYLE, WS_OVERLAPPEDWINDOW,
};

pub fn enable_dpi_awareness() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

pub fn dpi_of(hwnd: HWND) -> f32 {
    let dpi = unsafe { windows::Win32::UI::HiDpi::GetDpiForWindow(hwnd) };
    if dpi == 0 {
        96.0
    } else {
        dpi as f32
    }
}

pub fn client_size(hwnd: HWND) -> (u32, u32) {
    let mut rect = RECT::default();
    let ok = unsafe { GetClientRect(hwnd, &mut rect) };
    if ok.is_err() {
        return (0, 0);
    }
    ((rect.right - rect.left).max(0) as u32, (rect.bottom - rect.top).max(0) as u32)
}

pub fn set_dark_caption(hwnd: HWND, dark: bool) {
    let value: i32 = if dark { 1 } else { 0 };
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &value as *const i32 as *const core::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

pub fn set_rounded_corners(hwnd: HWND) {
    let value: i32 = DWMWCP_ROUND.0;
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &value as *const i32 as *const core::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

/// 按 `WINDOW_SIZES[index]` 调整窗口大小，并在所在显示器工作区内居中。
pub fn apply_size(hwnd: HWND, index: usize) {
    let fallback = &crate::model::WINDOW_SIZES[crate::model::DEFAULT_WINDOW_SIZE];
    let (dip_w, dip_h) = crate::model::WINDOW_SIZES
        .get(index)
        .map(|entry| (entry.0, entry.1))
        .unwrap_or((fallback.0, fallback.1));

    let dpi = dpi_of(hwnd).round().max(96.0) as u32;
    let scale = dpi as f32 / 96.0;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: (dip_w * scale).round() as i32,
        bottom: (dip_h * scale).round() as i32,
    };
    unsafe {
        let _ = AdjustWindowRectExForDpi(&mut rect, WS_OVERLAPPEDWINDOW, false, WINDOW_EX_STYLE(0), dpi);

        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let ok = GetMonitorInfoW(monitor, &mut info).as_bool();
        let work = if ok {
            info.rcWork
        } else {
            RECT { left: 0, top: 0, right: 1920, bottom: 1080 }
        };

        let work_w = (work.right - work.left).max(1);
        let work_h = (work.bottom - work.top).max(1);
        let width = (rect.right - rect.left).min(work_w - 20).max(320);
        let height = (rect.bottom - rect.top).min(work_h - 20).max(320);
        let x = work.left + (work_w - width) / 2;
        let y = work.top + (work_h - height) / 2;
        let _ = SetWindowPos(hwnd, None, x, y, width, height, SWP_NOZORDER | SWP_NOACTIVATE);
    }
}

/// 显示主窗口并前置（悬浮窗的 ⤢ 按钮）
pub fn show_and_focus(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }
}

pub fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

pub const APP_TITLE: &str = "番茄钟 · 计时";
pub const CLASS_NAME: &str = "PomodoroTimerWindow";
