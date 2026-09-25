use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO, NIM_ADD, NIM_DELETE, NIM_MODIFY,
};
use windows::Win32::UI::WindowsAndMessaging::HICON;

pub const WM_TRAY: u32 = 0x8000 + 10;
pub const ID_TRAY: u32 = 1;
pub const CMD_SHOW: usize = 100;
pub const CMD_TOGGLE: usize = 101;
pub const CMD_SKIP: usize = 102;
pub const CMD_QUIT: usize = 103;
pub const CMD_WIDGET: usize = 104;
pub const CMD_CLICK_THROUGH: usize = 105;

fn fill(buffer: &mut [u16], text: &str) {
    let data: Vec<u16> = text.encode_utf16().collect();
    let count = data.len().min(buffer.len().saturating_sub(1));
    buffer[..count].copy_from_slice(&data[..count]);
    buffer[count] = 0;
}

fn base(hwnd: HWND, icon: HICON, tip: &str) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: ID_TRAY,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY,
        hIcon: icon,
        ..Default::default()
    };
    fill(&mut data.szTip, tip);
    data
}

pub fn add(hwnd: HWND, icon: HICON, tip: &str) {
    let data = base(hwnd, icon, tip);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_ADD, &data);
    }
}

pub fn set_tip(hwnd: HWND, icon: HICON, tip: &str) {
    let data = base(hwnd, icon, tip);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

pub fn balloon(hwnd: HWND, icon: HICON, title: &str, body: &str) {
    let mut data = base(hwnd, icon, "");
    data.uFlags = NIF_INFO;
    fill(&mut data.szInfoTitle, title);
    fill(&mut data.szInfo, body);
    data.dwInfoFlags = NIIF_INFO;
    unsafe {
        data.Anonymous.uTimeout = 10000;
        let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

pub fn remove(hwnd: HWND, icon: HICON) {
    let data = base(hwnd, icon, "");
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}
