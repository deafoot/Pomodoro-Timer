use std::cell::{Cell, RefCell};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetMonitorInfoW, MonitorFromPoint,
    SelectObject, ValidateRect, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION,
    DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::action::Action;
use crate::clock;
use crate::color::Color;
use crate::model::Effects;
use crate::runtime::WidgetMode;
use crate::ui::{Rect, Ui};
use crate::views::widget_view;
use crate::{app_actions, query_host, store, with_host};

const CLASS_NAME: &str = "PomodoroTimerWidget";
/// 长按判定：超过这个时间且移动超过阈值才算拖动
const DRAG_DELAY_MS: u64 = 160;
/// 悬浮窗自己的刷新定时器：不依赖主窗口，数字才会实时跳
const TIMER_ID: usize = 2;
const REFRESH_MS: u32 = 100;
const DRAG_SLOP: i32 = 6;
/// 展开后多久没人碰就自动回待机
const STANDBY_MS: u64 = 15_000;

struct Press {
    cursor: POINT,
    window: (i32, i32),
    at: u64,
    action: Option<(Action, Rect)>,
}

struct Widget {
    hwnd: HWND,
    ui: Ui,
    dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
    width: u32,
    height: u32,
    press: Option<Press>,
    dragging: bool,
    tracking_leave: bool,
    /// 最近一次鼠标活动的时刻（15 秒没动静自动待机）
    last_active: u64,
}

thread_local! {
    static WIDGET: RefCell<Option<Widget>> = RefCell::new(None);
    static RENDERING: Cell<bool> = Cell::new(false);
}

fn new_dib(dc: HDC, width: u32, height: u32) -> Option<(HBITMAP, HGDIOBJ, *mut u8)> {
    let header = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width as i32,
        // 负高度 = 自上而下，行序和鼠标坐标一致
        biHeight: -(height as i32),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };
    let info = BITMAPINFO { bmiHeader: header, ..Default::default() };
    let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
    let bitmap = unsafe { CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) }.ok()?;
    let old = unsafe { SelectObject(dc, HGDIOBJ(bitmap.0)) };
    Some((bitmap, old, bits as *mut u8))
}

fn work_area(point: POINT) -> RECT {
    unsafe {
        let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            info.rcWork
        } else {
            RECT { left: 0, top: 0, right: 1920, bottom: 1080 }
        }
    }
}

/// 位置回夹到所在显示器工作区，避免换屏之后跑到屏幕外面
fn clamp_pos(x: i32, y: i32, width: i32, height: i32) -> POINT {
    let probe = POINT { x, y };
    let work = work_area(probe);
    let max_x = (work.right - width).max(work.left);
    let max_y = (work.bottom - height).max(work.top);
    POINT { x: x.clamp(work.left, max_x), y: y.clamp(work.top, max_y) }
}

pub fn is_open() -> bool {
    WIDGET.with(|cell| cell.borrow().is_some())
}

pub fn hwnd() -> Option<HWND> {
    WIDGET.with(|cell| cell.borrow().as_ref().map(|widget| widget.hwnd))
}

/// 悬浮窗当前的 DIP -> 像素缩放（含「图标大小」档位），输入法定位要用
pub fn ui_scale() -> Option<f32> {
    WIDGET.with(|cell| cell.borrow().as_ref().map(|widget| widget.ui.scale))
}

pub fn show() {
    if is_open() {
        if let Some(hwnd) = hwnd() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOWNA);
            }
            render(hwnd);
        }
        return;
    }
    let Some((stored, size, ui)) = query_host(|host| {
        Some((
            host.app.visual.widget_pos,
            widget_view::size(&host.app, &host.rt, clock::now_ms()),
            host.ui.fork(),
        ))
    })
    .flatten() else {
        return;
    };
    let dpi = unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem().max(96) } as f32;
    let scale = dpi / 96.0 * query_host(|host| host.app.visual.widget_scale.clamp(0.6, 1.8)).unwrap_or(1.0);
    let width = (size.0 * scale).round() as i32;
    let height = (size.1 * scale).round() as i32;
    let pos = match stored {
        Some((x, y)) => clamp_pos(x, y, width, height),
        None => {
            let work = work_area(POINT { x: 0, y: 0 });
            POINT {
                x: work.right - width - 28,
                y: work.top + (work.bottom - work.top) / 4,
            }
        }
    };
    let class = crate::platform::window::wide(CLASS_NAME);
    let title = crate::platform::window::wide("番茄钟小图标");
    let hwnd = unsafe {
        let Ok(instance) = GetModuleHandleW(None) else { return };
        let hinstance = HINSTANCE(instance.0);
        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(widget_proc),
            hInstance: hinstance,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            lpszClassName: PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&window_class);
        let topmost = query_host(|host| host.app.visual.widget_topmost).unwrap_or(true);
        let ex_style = WS_EX_LAYERED
            | WS_EX_TOOLWINDOW
            | if topmost { WS_EX_TOPMOST } else { WINDOW_EX_STYLE(0) };
        let Ok(hwnd) = CreateWindowExW(
            ex_style,
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_POPUP,
            pos.x,
            pos.y,
            width,
            height,
            None,
            None,
            Some(hinstance),
            None,
        ) else {
            return;
        };
        hwnd
    };
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
        return;
    }
    let Some((bitmap, old, _)) = new_dib(dc, width as u32, height as u32) else {
        unsafe {
            let _ = DeleteDC(dc);
            let _ = DestroyWindow(hwnd);
        }
        return;
    };
    let widget = Widget {
        hwnd,
        ui,
        dc,
        bitmap,
        old,
        width: width as u32,
        height: height as u32,
        press: None,
        dragging: false,
        tracking_leave: false,
        last_active: clock::now_ms(),
    };
    WIDGET.with(|cell| *cell.borrow_mut() = Some(widget));
    with_host(|host| {
        host.app.visual.widget_visible = true;
    });
    with_host(|host| store::save(&host.app));
    apply_topmost();
    apply_click_through();
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNA);
        SetTimer(Some(hwnd), TIMER_ID, REFRESH_MS, None);
    }
    render(hwnd);
}

pub fn hide() {
    if let Some(hwnd) = hwnd() {
        unsafe {
            let _ = KillTimer(Some(hwnd), TIMER_ID);
            let _ = ShowWindow(hwnd, SW_HIDE);
            let _ = DestroyWindow(hwnd);
        }
    }
    WIDGET.with(|cell| {
        if let Some(mut widget) = cell.borrow_mut().take() {
            widget.dispose();
        }
    });
    with_host(|host| {
        host.app.visual.widget_visible = false;
        // 悬浮窗都没了，正在改的活动名就取消掉（别留一个收不回的输入框）
        if host.rt.widget_editing {
            crate::app_actions::cancel_edit(host);
        }
        store::save(&host.app);
    });
    crate::platform::edit::end();
}

pub fn toggle() {
    if is_open() {
        hide();
    } else {
        show();
    }
}

/// 切换是否始终置顶
pub fn apply_topmost() {
    let Some(hwnd) = hwnd() else { return };
    let topmost = query_host(|host| host.app.visual.widget_topmost).unwrap_or(true);
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            Some(if topmost { HWND_TOPMOST } else { HWND_NOTOPMOST }),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

/// 切换整窗点击穿透：开着的时候鼠标事件直接落到桌面 / 下面的程序。
/// 分层窗口单靠 alpha 命中测试挡不住「不透明像素」，必须用 WS_EX_TRANSPARENT。
/// 注意穿透开着时悬浮窗自己也点不到，只能从托盘右键菜单关掉。
pub fn apply_click_through() {
    let Some(hwnd) = hwnd() else { return };
    let on = query_host(|host| host.app.visual.widget_click_through).unwrap_or(false);
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let next = if on {
            style | WS_EX_TRANSPARENT.0
        } else {
            style & !WS_EX_TRANSPARENT.0
        };
        if next != style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next as isize);
        }
    }
}

/// 「刷新当前状态」：重挂心跳、按当前状态重算窗口尺寸、重设样式并立刻重绘。
/// 切模块 / 改设置后小图标没跟上时点一下。
pub fn refresh() {
    let Some(hwnd) = hwnd() else { return };
    apply_topmost();
    apply_click_through();
    unsafe {
        let _ = SetTimer(Some(hwnd), TIMER_ID, REFRESH_MS, None);
    }
    render(hwnd);
}

/// 主窗口定时器里调用：让悬浮窗跟着刷新
pub fn tick() {
    if let Some(hwnd) = hwnd() {
        render(hwnd);
    }
}

fn render(hwnd: HWND) {
    if RENDERING.with(|flag| flag.replace(true)) {
        return;
    }
    render_inner(hwnd);
    RENDERING.with(|flag| flag.set(false));
}

fn render_inner(hwnd: HWND) {
    let now = clock::now_ms();
    let dpi = crate::platform::window::dpi_of(hwnd);
    // 「图标大小」直接乘进渲染 DPI：整套布局（含按钮和文字）等比缩放
    let (px_w, px_h, dpi) = query_host(|host| {
        let (width, height) = widget_view::size(&host.app, &host.rt, now);
        let dpi = (dpi * host.app.visual.widget_scale.clamp(0.6, 1.8)).max(48.0);
        let scale = dpi / 96.0;
        (
            ((width * scale).round() as u32).max(1),
            ((height * scale).round() as u32).max(1),
            dpi,
        )
    })
    .unwrap_or((1, 1, dpi));
    let updated = WIDGET.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(widget) = guard.as_mut() else { return false };
        if widget.width != px_w || widget.height != px_h {
            if !widget.resize(px_w, px_h) {
                return false;
            }
            // 展开 / 缩放后可能超出工作区，重新夹一下位置
            let mut frame = RECT::default();
            unsafe {
                let _ = GetWindowRect(hwnd, &mut frame);
            }
            let pos = clamp_pos(frame.left, frame.top, px_w as i32, px_h as i32);
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    pos.x,
                    pos.y,
                    px_w as i32,
                    px_h as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        if widget.ui.ensure_dc_target(widget.dc, px_w, px_h, dpi).is_err() {
            return false;
        }
        widget.ui.begin(Color::rgba(0, 0, 0, 0.0));
        with_host(|host| {
            widget_view::draw(&mut widget.ui, &host.app, &mut host.rt, now);
        });
        widget.ui.end();
        widget.ui.commit_hits();
        // pptDst 不能瞎传：传 (0,0) 会把窗口挪到屏幕左上角，必须给当前屏幕坐标
        let mut frame = RECT::default();
        unsafe {
            let _ = GetWindowRect(hwnd, &mut frame);
        }
        let dst = POINT { x: frame.left, y: frame.top };
        let point = POINT { x: 0, y: 0 };
        let size = SIZE { cx: px_w as i32, cy: px_h as i32 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        unsafe {
            let screen = windows::Win32::Graphics::Gdi::GetDC(None);
            let result = UpdateLayeredWindow(
                hwnd,
                Some(screen),
                Some(&dst),
                Some(&size),
                Some(widget.dc),
                Some(&point),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
            windows::Win32::Graphics::Gdi::ReleaseDC(None, screen);
            result.is_ok()
        }
    });
    let _ = updated;
    // 视图画完才知道编辑框在哪：把内嵌输入控件摆过去
    crate::sync_inline_edit(hwnd);
}

/// 整个窗口矩形都算命中：按在空白处也要能拖动。
/// 真正的「点击穿透」靠 WS_EX_TRANSPARENT 窗口样式（见 apply_click_through）：
/// HTTRANSPARENT 只对同线程的窗口有效，管不到桌面和其它程序。
fn hit_test() -> LRESULT {
    LRESULT(HTCLIENT as i32 as isize)
}

fn client_pos(widget: &mut Widget, lp: LPARAM) -> POINT {
    let point = POINT {
        x: (lp.0 & 0xFFFF) as u16 as i16 as i32,
        y: ((lp.0 >> 16) & 0xFFFF) as u16 as i16 as i32,
    };
    let scale = widget.ui.scale.max(0.1);
    widget.ui.mouse = (point.x as f32 / scale, point.y as f32 / scale);
    point
}

fn on_down(hwnd: HWND, lp: LPARAM) {
    let mut cursor = POINT::default();
    let mut rect = RECT::default();
    unsafe {
        let _ = GetCursorPos(&mut cursor);
        let _ = GetWindowRect(hwnd, &mut rect);
        SetCapture(hwnd);
    }
    let at = clock::now_ms();
    WIDGET.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(widget) = guard.as_mut() else { return };
        widget.last_active = at;
        client_pos(widget, lp);
        let action = widget.ui.hit_action();
        widget.press = Some(Press {
            cursor,
            window: (rect.left, rect.top),
            at,
            action,
        });
        widget.dragging = false;
    });
}

fn on_move(hwnd: HWND, lp: LPARAM) {
    let mut cursor = POINT::default();
    // 固定位置开着：按下去照样能点按钮 / 收放下拉，就是别把窗口拖走
    let locked = query_host(|host| host.app.visual.widget_locked).unwrap_or(false);
    unsafe {
        let _ = GetCursorPos(&mut cursor);
    }
    let dragging = WIDGET.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(widget) = guard.as_mut() else { return false };
        client_pos(widget, lp);
        if !widget.tracking_leave {
            widget.tracking_leave = true;
            let mut track = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            unsafe {
                let _ = TrackMouseEvent(&mut track);
            }
        }
        let now = clock::now_ms();
        widget.last_active = now;
        let Some(press) = widget.press.as_mut() else { return false };
        let dx = cursor.x - press.cursor.x;
        let dy = cursor.y - press.cursor.y;
        let moved = dx.abs() > DRAG_SLOP || dy.abs() > DRAG_SLOP;
        // 长按之后拖动，或者直接快拖一段距离，都算拖动（不然有时候拖不动）
        let decided = now.saturating_sub(press.at) >= DRAG_DELAY_MS || dx.abs() > 18 || dy.abs() > 18;
        if !locked && !widget.dragging && moved && decided {
            widget.dragging = true;
        }
        if widget.dragging {
            let x = press.window.0 + dx;
            let y = press.window.1 + dy;
            unsafe {
                let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
        widget.dragging
    });
    render(hwnd);
    let _ = dragging;
}

fn on_up(hwnd: HWND, lp: LPARAM) {
    unsafe {
        let _ = ReleaseCapture();
    }
    let (dragging, hit) = WIDGET.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(widget) = guard.as_mut() else { return (false, None) };
        let dragging = widget.dragging;
        widget.dragging = false;
        widget.last_active = clock::now_ms();
        client_pos(widget, lp);
        let hit = widget.press.take().and_then(|press| press.action);
        (dragging, hit)
    });
    if dragging {
        let mut rect = RECT::default();
        unsafe {
            let _ = GetWindowRect(hwnd, &mut rect);
        }
        with_host(|host| {
            host.app.visual.widget_pos = Some((rect.left, rect.top));
            store::save(&host.app);
        });
        return;
    }
    // 按在按钮上就在原位执行；按在空白 / 按钮外只收起下拉和编辑，不动运行-待机模式
    match hit {
        Some((action, rect)) => {
            let inside = WIDGET.with(|cell| {
                let guard = cell.borrow();
                guard
                    .as_ref()
                    .map(|widget| rect.contains(widget.ui.mouse.0, widget.ui.mouse.1))
                    .unwrap_or(false)
            });
            if inside {
                run_action(action);
            } else {
                dismiss();
            }
        }
        None => dismiss(),
    }
    render(hwnd);
}

/// 点在按钮之外（数字 / 图形 / 活动 / 空白）：
/// - 有下拉或改名在进行：只收起来，不动状态；
/// - 否则在「待机·未点击 ↔ 待机·已点击」之间切换；运行模式不动（防误触）。
fn dismiss() {
    let mut main = None;
    with_host(|host| {
        main = Some(host.hwnd);
        let now = clock::now_ms();
        let had_overlay = host.rt.open_drop.is_some() || host.rt.edit.is_some();
        let mut effects = Effects::default();
        app_actions::click_away(host, &mut effects, now);
        crate::apply_effects(host, effects, now);
        if !had_overlay {
            host.rt.widget_mode = match host.rt.widget_mode {
                WidgetMode::Idle => WidgetMode::Standby,
                WidgetMode::Standby => WidgetMode::Idle,
                WidgetMode::Running => WidgetMode::Running,
            };
        }
    });
    if let Some(hwnd) = main {
        crate::invalidate(hwnd);
    }
}

fn run_action(action: Action) {
    let now = clock::now_ms();
    let rect = Rect::new(0.0, 0.0, 0.0, 0.0);
    let mut main = None;
    with_host(|host| {
        main = Some(host.hwnd);
        if matches!(action, Action::DropToggle(_)) {
            host.rt.widget_drop = true;
        }
        let mut effects = Effects::default();
        app_actions::apply(host, action, rect, &mut effects, now);
        crate::apply_effects(host, effects, now);
    });
    if let Some(hwnd) = main {
        crate::invalidate(hwnd);
        // 建 / 拆悬浮窗这种动窗口的动作，Post 回主窗口的消息循环里执行：
        // 主窗口收进托盘时也不会漏掉（否则左上角的关闭按钮看着像没用）
        if query_host(|host| host.rt.pending.is_some()).unwrap_or(false) {
            unsafe {
                let _ = PostMessageW(Some(hwnd), crate::WM_PENDING, WPARAM(0), LPARAM(0));
            }
        }
    }
}

impl Widget {
    fn resize(&mut self, width: u32, height: u32) -> bool {
        let Some((bitmap, _old, _)) = new_dib(self.dc, width, height) else { return false };
        // new_dib 已经把新位图选进 DC，旧位图随之被换出，这里直接删掉它就行。
        // 千万不能把 DC 原来那张 1x1 位图再选回去：DC 里一旦不是我们的位图，
        // D2D 和 UpdateLayeredWindow 就都画不出东西，只能重建悬浮窗才恢复。
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
        }
        self.bitmap = bitmap;
        self.width = width;
        self.height = height;
        true
    }

    fn dispose(&mut self) {
        unsafe {
            let _ = SelectObject(self.dc, self.old);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.dc);
        }
    }
}

unsafe extern "system" fn widget_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        // 悬浮窗自己的心跳：100ms 重画一次，数字 / 圆环实时跟着走
        WM_TIMER => {
            let now = clock::now_ms();
            let idle = WIDGET.with(|cell| {
                let guard = cell.borrow();
                let Some(widget) = guard.as_ref() else { return false };
                now.saturating_sub(widget.last_active) >= STANDBY_MS
            });
            let mode = query_host(|host| host.rt.widget_mode).unwrap_or(WidgetMode::Idle);
            let editing = query_host(|host| host.rt.widget_editing).unwrap_or(false);
            if mode.buttons() && !editing && idle {
                // 15 秒没人碰：自动待机，按钮全部收起
                with_host(|host| {
                    host.rt.widget_mode = WidgetMode::Idle;
                    host.rt.open_drop = None;
                });
            }
            render(hwnd);
            LRESULT(0)
        }
        WM_PAINT => {
            render(hwnd);
            let _ = ValidateRect(Some(hwnd), None);
            LRESULT(0)
        }
        WM_MOUSEACTIVATE => {
            let editing = query_host(|host| host.rt.widget_editing).unwrap_or(false);
            LRESULT(if editing { MA_ACTIVATE as i32 as isize } else { MA_NOACTIVATE as i32 as isize })
        }
        WM_NCHITTEST => hit_test(),
        WM_LBUTTONDOWN => {
            on_down(hwnd, lp);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            on_move(hwnd, lp);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            on_up(hwnd, lp);
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            WIDGET.with(|cell| {
                let mut guard = cell.borrow_mut();
                if let Some(widget) = guard.as_mut() {
                    widget.tracking_leave = false;
                    widget.ui.mouse = (-1.0, -1.0);
                }
            });
            render(hwnd);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wp, lp) },
    }
}
