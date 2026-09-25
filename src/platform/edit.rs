//! 内嵌文本输入：在编辑框的位置上挂一个真正的系统 EDIT 控件。
//!
//! 自己画界面 + 自己解析 WM_IME_* 这条路对各种输入法都不稳，所以这里不再碰输入法：
//! 组字、候选窗、上屏、光标、选区、Ctrl+A/C/V、鼠标选字全部交给系统控件，
//! 我们只做三件事——摆位置、按主题上色、把文本同步回 rt.edit。

use std::cell::Cell;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, FillRect, SetBkColor, SetTextColor, CLEARTYPE_QUALITY,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, FW_NORMAL, HBRUSH, HDC, HGDIOBJ, OUT_TT_PRECIS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Controls::{EM_SCROLLCARET, EM_SETSEL};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, SetFocus, VK_ESCAPE, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::*;

/// 控件里的文本变了（wParam = 会话号）
pub const WM_EDIT_CHANGED: u32 = WM_APP + 12;
/// 回车或失焦：该提交了（wParam = 会话号）
pub const WM_EDIT_COMMIT: u32 = WM_APP + 13;
/// 转义：该取消了（wParam = 会话号）
pub const WM_EDIT_CANCEL: u32 = WM_APP + 14;

const HOST_CLASS: &str = "PomodoroTimerEditHost";
const FONT_FAMILY: &str = "Microsoft YaHei UI";
const EDIT_ID: usize = 1;

/// 一次编辑会话要告诉控件的东西
pub struct Ctx {
    /// 通知发给哪个窗口
    pub owner: HWND,
    /// 会话号（= rt.edit 的 id）：通知带上它，过期的通知宿主直接丢掉
    pub session: u64,
    /// 摆放位置（屏幕像素）
    pub rect: RECT,
    pub text: String,
    pub font_px: f32,
    pub select_all: bool,
    /// 底色 / 文字色（COLORREF）
    pub surface: u32,
    pub text_color: u32,
    /// 悬浮窗要盖在置顶的小图标上面
    pub topmost: bool,
}

thread_local! {
    static HOST: Cell<isize> = const { Cell::new(0) };
    static EDIT: Cell<isize> = const { Cell::new(0) };
    static OLD_PROC: Cell<WNDPROC> = const { Cell::new(None) };
    static OWNER: Cell<isize> = const { Cell::new(0) };
    static SESSION: Cell<u64> = const { Cell::new(0) };
    /// 已经推进控件的会话号：同一个会话不再 SetWindowText，否则光标会被顶回开头
    static PUSHED: Cell<u64> = const { Cell::new(0) };
    /// 控件当前是不是正显示着（只有「刚显示 / 换了会话」才去抢焦点）
    static VISIBLE: Cell<bool> = const { Cell::new(false) };
    static SURFACE: Cell<u32> = const { Cell::new(0x00FF_FFFF) };
    static TEXT_COLOR: Cell<u32> = const { Cell::new(0) };
    static BRUSH: Cell<isize> = const { Cell::new(0) };
    static BRUSH_COLOR: Cell<u32> = const { Cell::new(u32::MAX) };
    static FONT: Cell<isize> = const { Cell::new(0) };
    static FONT_PX: Cell<u32> = const { Cell::new(0) };
    static PLACED: Cell<RECT> = const { Cell::new(RECT { left: 0, top: 0, right: 0, bottom: 0 }) };
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn host() -> HWND {
    HWND(HOST.with(|c| c.get()) as *mut core::ffi::c_void)
}

fn edit() -> HWND {
    HWND(EDIT.with(|c| c.get()) as *mut core::ffi::c_void)
}

fn owner() -> HWND {
    HWND(OWNER.with(|c| c.get()) as *mut core::ffi::c_void)
}

fn surface_brush() -> HBRUSH {
    HBRUSH(BRUSH.with(|c| c.get()) as *mut core::ffi::c_void)
}

/// 通知宿主窗口，wParam 带上会话号
fn notify(message: u32) {
    let target = owner();
    if target.is_invalid() {
        return;
    }
    unsafe {
        let _ = PostMessageW(Some(target), message, WPARAM(SESSION.with(|c| c.get()) as usize), LPARAM(0));
    }
}

unsafe extern "system" fn host_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => {
            let mut rect = RECT::default();
            let _ = unsafe { GetClientRect(hwnd, &mut rect) };
            unsafe { FillRect(HDC(wp.0 as *mut core::ffi::c_void), &rect, surface_brush()) };
            LRESULT(1)
        }
        WM_CTLCOLOREDIT => {
            let dc = HDC(wp.0 as *mut core::ffi::c_void);
            unsafe {
                SetBkColor(dc, COLORREF(SURFACE.with(|c| c.get())));
                SetTextColor(dc, COLORREF(TEXT_COLOR.with(|c| c.get())));
            }
            LRESULT(surface_brush().0 as isize)
        }
        WM_COMMAND => {
            match ((wp.0 >> 16) & 0xFFFF) as u32 {
                EN_CHANGE => notify(WM_EDIT_CHANGED),
                EN_KILLFOCUS => notify(WM_EDIT_COMMIT),
                _ => {}
            }
            LRESULT(0)
        }
        // 宿主自己那圈 2px 边框别吃点击：让底下的界面照常收到消息
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as i32 as isize),
        // 滚轮丢给宿主窗口，编辑着也能滚动列表
        WM_MOUSEWHEEL => {
            let target = owner();
            if !target.is_invalid() {
                unsafe {
                    let _ = PostMessageW(Some(target), msg, wp, lp);
                }
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wp, lp) },
    }
}

/// 给 EDIT 挂一层子类化：回车 / 转义自己处理，顺手记一下输入法消息（诊断用）
unsafe extern "system" fn edit_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_KEYDOWN | WM_SYSKEYDOWN => match wp.0 as u32 {
            value if value == VK_RETURN.0 as u32 => {
                notify(WM_EDIT_COMMIT);
                return LRESULT(0);
            }
            value if value == VK_ESCAPE.0 as u32 => {
                notify(WM_EDIT_CANCEL);
                return LRESULT(0);
            }
            _ => {}
        },
        WM_CHAR => {
            let ch = wp.0 as u32;
            if ch == 0x0D || ch == 0x1B {
                // 回车 / 转义上面已经处理，别让 EDIT 再响一声
                return LRESULT(0);
            }
            if crate::ime_log_on() {
                crate::ime_log(&format!("EDIT WM_CHAR U+{ch:04X}"));
            }
        }
        WM_IME_STARTCOMPOSITION => crate::ime_log("EDIT STARTCOMPOSITION"),
        WM_IME_ENDCOMPOSITION => crate::ime_log("EDIT ENDCOMPOSITION"),
        WM_IME_CHAR => {
            if crate::ime_log_on() {
                crate::ime_log(&format!("EDIT WM_IME_CHAR U+{:04X}", wp.0));
            }
        }
        _ => {}
    }
    let old = OLD_PROC.with(|c| c.get());
    unsafe { CallWindowProcW(old, hwnd, msg, wp, lp) }
}

fn ensure() -> bool {
    if !host().is_invalid() && !edit().is_invalid() {
        return true;
    }
    unsafe {
        let Ok(instance) = GetModuleHandleW(None) else { return false };
        let hinstance = HINSTANCE(instance.0);
        let class = wide(HOST_CLASS);
        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(host_proc),
            hInstance: hinstance,
            hCursor: LoadCursorW(None, IDC_IBEAM).unwrap_or_default(),
            lpszClassName: PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&window_class);
        let title = wide("番茄钟输入框");
        let Ok(host) = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_POPUP,
            0,
            0,
            10,
            10,
            None,
            None,
            Some(hinstance),
            None,
        ) else {
            return false;
        };
        HOST.with(|c| c.set(host.0 as isize));
        let kind = wide("EDIT");
        let Ok(field) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(kind.as_ptr()),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE((ES_LEFT | ES_AUTOHSCROLL | ES_NOHIDESEL) as u32),
            0,
            0,
            10,
            10,
            Some(host),
            Some(HMENU(EDIT_ID as usize as *mut core::ffi::c_void)),
            Some(hinstance),
            None,
        ) else {
            return false;
        };
        EDIT.with(|c| c.set(field.0 as isize));
        let proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT = edit_proc;
        let old = SetWindowLongPtrW(field, GWLP_WNDPROC, proc as usize as isize);
        OLD_PROC.with(|c| c.set(core::mem::transmute::<isize, WNDPROC>(old)));
        true
    }
}

fn set_colors(surface: u32, text_color: u32) {
    SURFACE.with(|c| c.set(surface));
    TEXT_COLOR.with(|c| c.set(text_color));
    if BRUSH_COLOR.with(|c| c.get()) == surface && BRUSH.with(|c| c.get()) != 0 {
        return;
    }
    let brush = unsafe { CreateSolidBrush(COLORREF(surface)) };
    let old = BRUSH.with(|c| c.replace(brush.0 as isize));
    BRUSH_COLOR.with(|c| c.set(surface));
    if old != 0 {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(old as *mut core::ffi::c_void));
        }
    }
}

fn set_font(px: f32) {
    let px = px.round().clamp(9.0, 96.0) as u32;
    if FONT_PX.with(|c| c.get()) == px && FONT.with(|c| c.get()) != 0 {
        return;
    }
    let field = edit();
    if field.is_invalid() {
        return;
    }
    let face = wide(FONT_FAMILY);
    let font = unsafe {
        CreateFontW(
            -(px as i32),
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_TT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            PCWSTR(face.as_ptr()),
        )
    };
    let old = FONT.with(|c| c.replace(font.0 as isize));
    FONT_PX.with(|c| c.set(px));
    unsafe {
        if old != 0 {
            let _ = DeleteObject(HGDIOBJ(old as *mut core::ffi::c_void));
        }
        let _ = SendMessageW(field, WM_SETFONT, Some(WPARAM(font.0 as usize)), Some(LPARAM(1)));
    }
}

/// 把焦点和前台都交给输入控件（弱进程抢不到前台，借一下前台线程的输入队列）
fn grab_focus(host: HWND, field: HWND) {
    unsafe {
        let foreground = GetForegroundWindow();
        let target = GetWindowThreadProcessId(foreground, None);
        let current = GetCurrentThreadId();
        let attached = target != 0 && target != current && AttachThreadInput(current, target, true).as_bool();
        let _ = SetForegroundWindow(host);
        let _ = SetFocus(Some(field));
        if attached {
            let _ = AttachThreadInput(current, target, false);
        }
    }
}

/// 摆到 rect 上、按需聚焦；换了会话就把文本推给控件
pub fn show(ctx: &Ctx) {
    if !ensure() {
        return;
    }
    let host = host();
    let field = edit();
    OWNER.with(|c| c.set(ctx.owner.0 as isize));
    SESSION.with(|c| c.set(ctx.session));
    set_colors(ctx.surface, ctx.text_color);
    set_font(ctx.font_px);

    let width = (ctx.rect.right - ctx.rect.left).max(8);
    let height = (ctx.rect.bottom - ctx.rect.top).max(8);
    let placed = PLACED.with(|c| c.get());
    let pushed_new = PUSHED.with(|c| c.get()) != ctx.session;
    let same_place = placed.left == ctx.rect.left
        && placed.top == ctx.rect.top
        && placed.right == ctx.rect.right
        && placed.bottom == ctx.rect.bottom;
    unsafe {
        if pushed_new {
            let data = wide(&ctx.text);
            let _ = SetWindowTextW(field, PCWSTR(data.as_ptr()));
            let len = ctx.text.encode_utf16().count() as isize;
            let (start, end) = if ctx.select_all { (0isize, -1isize) } else { (len, len) };
            let _ = SendMessageW(field, EM_SETSEL, Some(WPARAM(start as usize)), Some(LPARAM(end)));
            let _ = SendMessageW(field, EM_SCROLLCARET, Some(WPARAM(0)), Some(LPARAM(0)));
            PUSHED.with(|c| c.set(ctx.session));
        }
        if !same_place {
            PLACED.with(|c| c.set(ctx.rect));
            let z = if ctx.topmost { HWND_TOPMOST } else { HWND_TOP };
            let _ = SetWindowPos(host, Some(z), ctx.rect.left, ctx.rect.top, width, height, SWP_NOACTIVATE | SWP_SHOWWINDOW);
            // EDIT 铺满宿主，上下留一点内边距，看着像我们自己的输入框
            let text_h = ((ctx.font_px * 1.7).round() as i32).clamp(14, (height - 4).max(14));
            let top = (height - text_h) / 2;
            let _ = SetWindowPos(field, None, 6, top, (width - 12).max(8), text_h, SWP_NOZORDER | SWP_NOACTIVATE);
        }
        // 只在「刚显示出来 / 换了会话」时抢焦点：用户点到别处之后别再抢回来
        if !VISIBLE.with(|c| c.get()) || pushed_new {
            grab_focus(host, field);
        }
        VISIBLE.with(|c| c.set(true));
    }
}

/// 收起控件，把焦点 / 前台还给宿主
pub fn end() {
    let host = host();
    if host.is_invalid() {
        return;
    }
    let field = edit();
    unsafe {
        if !field.is_invalid() && GetFocus() == field {
            let _ = SetFocus(Some(owner()));
        }
        if GetForegroundWindow() == host {
            let _ = SetForegroundWindow(owner());
        }
        let _ = ShowWindow(host, SW_HIDE);
    }
    VISIBLE.with(|c| c.set(false));
    PLACED.with(|c| c.set(RECT { left: 0, top: 0, right: 0, bottom: 0 }));
}

/// 读回控件里的文本
pub fn text() -> Option<String> {
    let field = edit();
    if field.is_invalid() {
        return None;
    }
    unsafe {
        let len = GetWindowTextLengthW(field);
        if len <= 0 {
            return Some(String::new());
        }
        let mut buffer = vec![0u16; len as usize + 1];
        let copied = GetWindowTextW(field, &mut buffer);
        Some(String::from_utf16_lossy(&buffer[..copied as usize]))
    }
}
