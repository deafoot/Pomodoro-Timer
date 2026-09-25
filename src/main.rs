#![windows_subsystem = "windows"]

mod action;
mod app_actions;
mod clock;
mod color;
mod drops;
mod model;
mod platform;
mod runtime;
mod store;
mod ui;
mod views;

use std::cell::RefCell;

use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForSystem};
use windows::Win32::Graphics::Gdi::{BeginPaint, ClientToScreen, EndPaint, InvalidateRect, PAINTSTRUCT, UpdateWindow};
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Shell::NIN_BALLOONUSERCLICK;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::*;


use crate::model::{App, Effect, Effects, DEFAULT_WINDOW_SIZE, WINDOW_SIZES};
use crate::runtime::{PendingDialog, Runtime};
use crate::color::Color;
use crate::ui::{Rect, Ui};

const TIMER_ID: usize = 1;
const FAST_INTERVAL: u32 = 200;
const SLOW_INTERVAL: u32 = 1000;
/// 悬浮窗触发的建 / 拆窗口动作，Post 回主窗口再执行
pub(crate) const WM_PENDING: u32 = WM_APP + 1;

pub struct Host {
    pub ui: Ui,
    pub app: App,
    pub rt: Runtime,
    pub hwnd: HWND,
    pub icon: HICON,
    pub dirty: bool,
    pub fast: bool,
    pub dragging: bool,
    pub tracking_leave: bool,
    /// 当前定时器间隔（避免每 tick 都重设）
    pub interval: u32,
}

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
}

pub(crate) fn with_host(f: impl FnOnce(&mut Host)) {
    HOST.with(|cell| {
        let mut guard = cell.borrow_mut();
        if let Some(host) = guard.as_mut() {
            f(host);
        }
    });
}

/// 需要从宿主里取一个值的场合
pub(crate) fn query_host<R>(f: impl FnOnce(&mut Host) -> R) -> Option<R> {
    HOST.with(|cell| {
        let mut guard = cell.borrow_mut();
        guard.as_mut().map(f)
    })
}

pub(crate) fn invalidate(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
        // 立刻重画：拖进度条时 WM_PAINT 会被汹涌的鼠标消息饿死，看着就一顿一顿
        let _ = UpdateWindow(hwnd);
    }
    // 主窗口每次刷新都顺手刷一下桌面小图标，避免桌面显示滞后
    if platform::widget::hwnd() != Some(hwnd) {
        platform::widget::tick();
    }
}

/// 距离下一个整秒还有多久（40ms - 1s）。跑动中把定时器对齐到整秒，
/// 图形倒计时就按整秒一顿一顿地跳，每秒只唤醒一次。
fn fast_interval(now: u64) -> u32 {
    (1000 - (now % 1000)).clamp(40, 1000) as u32
}

fn now_ms() -> u64 {
    clock::now_ms()
}

/// 输入法诊断开关：设了环境变量 POMODORO_IME_LOG 才写日志，平时零开销。
pub(crate) fn ime_log_on() -> bool {
    std::env::var_os("POMODORO_IME_LOG").is_some()
}

/// 把「输入法到底有没有把消息送进来」记到 %APPDATA%\PomodoroTimer\ime.log：
/// 打一次中文拼音就能分辨是「消息根本没来」（焦点 / TSF 问题）还是「来了但我们处理错了」。
pub(crate) fn ime_log(message: &str) {
    if !ime_log_on() {
        return;
    }
    use std::io::Write;
    let path = store::state_path().with_file_name("ime.log");
    let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let focus = unsafe { GetFocus() };
    let foreground = unsafe { GetForegroundWindow() };
    let _ = writeln!(
        file,
        "{} focus={:?} foreground={:?} {}",
        clock::now_ms(),
        focus.0,
        foreground.0,
        message
    );
}


pub(crate) fn apply_effects(host: &mut Host, effects: Effects, now: u64) {
    for effect in effects.items {
        match effect {
            Effect::Toast(text) => host.rt.show_toast(text, now),
            Effect::Sound { kind, path, looping } => {
                platform::sound::play(kind, &path, looping);
                host.rt.sound_started(now, looping);
            }
            Effect::Balloon { title, body } => platform::tray::balloon(host.hwnd, host.icon, &title, &body),
        }
    }
}

fn resize(hwnd: HWND) {
    let (width, height) = platform::window::client_size(hwnd);
    let dpi = platform::window::dpi_of(hwnd);
    with_host(|host| {
        let _ = host.ui.ensure_target(hwnd, width, height, dpi);
    });
    invalidate(hwnd);
}

fn paint(hwnd: HWND) {
    let mut ps = PAINTSTRUCT::default();
    let _ = unsafe { BeginPaint(hwnd, &mut ps) };
    let now = now_ms();
    with_host(|host| {
        let (width, height) = platform::window::client_size(hwnd);
        if width == 0 || height == 0 {
            return;
        }
        if !host.ui.has_target() {
            let dpi = platform::window::dpi_of(hwnd);
            let _ = host.ui.ensure_target(hwnd, width, height, dpi);
        }
        let clear = host.rt.palette.page;
        host.ui.begin(clear);
        let ui = &mut host.ui;
        let app = &host.app;
        let rt = &mut host.rt;
        views::draw(ui, app, rt, now);
        host.ui.end();
        host.ui.commit_hits();
    });
    unsafe {
        let _ = EndPaint(hwnd, &ps);
    }
    // 视图画完才知道编辑框在哪：把内嵌输入控件摆过去
    sync_inline_edit(hwnd);
}

fn tray_tip(host: &Host) -> String {
    let now = now_ms();
    match host.app.pomodoro.status {
        crate::model::RunStatus::Running => {
            let stage = host.app.pomodoro.stage.name();
            format!(
                "{} · {} {}",
                "番茄钟",
                stage,
                crate::model::fmt_time(host.app.pomodoro.remaining_sec(now))
            )
        }
        _ => {
            if host.app.timer.status == crate::model::RunStatus::Running {
                format!("计时中 {}", crate::model::fmt_time(host.app.timer.elapsed_sec(now)))
            } else {
                "番茄钟 · 计时".to_string()
            }
        }
    }
}

fn on_timer(hwnd: HWND) {
    let now = now_ms();
    let mut new_size: Option<usize> = None;
    with_host(|host| {
        let mut effects = Effects::default();
        host.app.tick(now, &mut effects);
        if host.rt.toast_expired(now) {
            host.rt.toast = None;
        }
        apply_effects(host, effects, now);
        let tip = tray_tip(host);
        platform::tray::set_tip(host.hwnd, host.icon, &tip);
        // 谁刚开始跑就把小图标切给谁（之后用户还能在小图标上手动改）
        let running = match host.app.pomodoro.status != crate::model::RunStatus::Idle {
            true => Some(crate::action::ViewId::Pomodoro),
            false if host.app.timer.status != crate::model::RunStatus::Idle => Some(crate::action::ViewId::Timer),
            false => None,
        };
        if running != host.rt.widget_running_seen {
            host.rt.widget_running_seen = running;
            if let Some(view) = running {
                host.rt.widget_module = view;
            }
        }
        let fast = host.app.has_running_timer() || host.rt.toast.is_some();
        let interval = if fast { fast_interval(now) } else { SLOW_INTERVAL };
        if fast != host.fast || interval != host.interval {
            host.fast = fast;
            host.interval = interval;
            unsafe {
                SetTimer(Some(host.hwnd), TIMER_ID, interval, None);
            }
        }
        if host.dirty {
            store::save(&host.app);
            host.dirty = false;
        }
        if host.rt.window_dirty {
            host.rt.window_dirty = false;
            new_size = Some(host.app.visual.window_size);
        }
    });
    if let Some(index) = new_size {
        platform::window::apply_size(hwnd, index);
    }
    invalidate(hwnd);
    platform::widget::tick();
}

fn mouse_pos(host: &Host, lp: LPARAM) -> (f32, f32) {
    let x = (lp.0 & 0xFFFF) as u16 as i16 as f32;
    let y = ((lp.0 >> 16) & 0xFFFF) as u16 as i16 as f32;
    (x / host.ui.scale, y / host.ui.scale)
}

fn on_mouse_move(hwnd: HWND, lp: LPARAM) {
    let now = now_ms();
    with_host(|host| {
        let (x, y) = mouse_pos(host, lp);
        host.ui.mouse = (x, y);
        if !host.tracking_leave {
            host.tracking_leave = true;
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
        if host.dragging {
            if let Some((action, rect)) = host.rt.drag.clone() {
                let mut effects = Effects::default();
                app_actions::apply(host, action, rect, &mut effects, now);
                if !effects.items.is_empty() {
                    apply_effects(host, effects, now);
                }
            }
        }
    });
    invalidate(hwnd);
}

fn on_click(hwnd: HWND) {
    let now = now_ms();
    with_host(|host| {
        host.dragging = true;
        // 主窗口里的操作一律关掉悬浮窗打开的下拉
        host.rt.widget_drop = false;
        let hit = host.ui.hit_action();
        let mut effects = Effects::default();
        match hit {
            Some((action, rect)) => {
                if action.is_slider() {
                    host.rt.drag = Some((action.clone(), rect));
                }
                app_actions::apply(host, action, rect, &mut effects, now);
            }
            None => app_actions::click_away(host, &mut effects, now),
        }
        apply_effects(host, effects, now);
    });
    invalidate(hwnd);
}

fn on_release(hwnd: HWND) {
    with_host(|host| {
        host.dragging = false;
        host.rt.drag = None;
    });
    invalidate(hwnd);
}

fn on_wheel(hwnd: HWND, wp: WPARAM) {
    let delta = ((wp.0 >> 16) & 0xFFFF) as u16 as i16 as f32;
    with_host(|host| {
        let step = -delta / 120.0 * 60.0;
        if host.rt.open_drop.is_some() {
            host.rt.drop_scroll = (host.rt.drop_scroll + step).max(0.0);
            return;
        }
        if host.rt.calendar_open {
            return;
        }
        let view = host.rt.view;
        let current = host.rt.scroll_of(view);
        host.rt.set_scroll(view, current + step);
    });
    invalidate(hwnd);
}

/// 视图 DIP 的缩放：悬浮窗的缩放里还含「图标大小」档位
fn view_scale(hwnd: HWND) -> f32 {
    match platform::widget::hwnd() {
        Some(widget) if widget == hwnd => platform::widget::ui_scale().unwrap_or(1.0),
        _ => (platform::window::dpi_of(hwnd) / 96.0).max(1.0),
    }
}

/// 视图坐标（DIP）→ 屏幕像素
fn view_rect_screen(hwnd: HWND, rect: Rect, scale: f32) -> RECT {
    let mut top_left = POINT { x: (rect.x * scale) as i32, y: (rect.y * scale) as i32 };
    let mut bottom_right = POINT { x: ((rect.x + rect.w) * scale) as i32, y: ((rect.y + rect.h) * scale) as i32 };
    unsafe {
        let _ = ClientToScreen(hwnd, &mut top_left);
        let _ = ClientToScreen(hwnd, &mut bottom_right);
    }
    RECT { left: top_left.x, top: top_left.y, right: bottom_right.x, bottom: bottom_right.y }
}

/// 0.0-1.0 的颜色 → COLORREF
fn colorref(color: Color) -> u32 {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u32;
    channel(color.r) | (channel(color.g) << 8) | (channel(color.b) << 16)
}

/// 每帧把内嵌输入控件摆到当前编辑框的位置上（位置要等视图画完才知道）
pub(crate) fn sync_inline_edit(hwnd: HWND) {
    // 三个结果：摆控件 / 收起控件 / 这个窗口别管（另一个窗口正在用）
    enum Sync {
        Show(platform::edit::Ctx),
        End,
        Skip,
    }
    let mut sync = Sync::End;
    with_host(|host| {
        let Some(edit) = host.rt.edit.as_ref() else { return };
        // rt.edit 是两个窗口共用的：谁在编辑看 widget_editing，位置也各记各的
        let is_widget = platform::widget::hwnd() == Some(hwnd);
        let (rect, mine) = if is_widget {
            (host.rt.widget_edit_rect, host.rt.widget_editing)
        } else {
            (host.rt.edit_rect, !host.rt.widget_editing)
        };
        if !mine {
            // 这次编辑不归这个窗口：别去动别人正在用的输入框
            sync = Sync::Skip;
            return;
        }
        if !rect.valid() {
            // 视图还没画出编辑框，等下一帧
            return;
        }
        let scale = view_scale(hwnd);
        let mut screen = view_rect_screen(hwnd, rect, scale);
        // 往里收 2px：底下的输入框边框还能看见，控件只盖住文字区
        let pad = (2.0 * scale).round() as i32;
        screen.left += pad;
        screen.right -= pad;
        screen.top += pad;
        screen.bottom -= pad;
        sync = Sync::Show(platform::edit::Ctx {
            owner: host.hwnd,
            session: edit.id,
            rect: screen,
            text: edit.text.clone(),
            font_px: 13.0 * scale,
            select_all: edit.select_all,
            surface: colorref(host.rt.palette.surface),
            text_color: colorref(host.rt.palette.text),
            topmost: is_widget,
        });
    });
    match sync {
        Sync::Show(ctx) => platform::edit::show(&ctx),
        Sync::End => platform::edit::end(),
        Sync::Skip => {}
    }
}

/// 输入控件里的文本变了：同步回 rt.edit（过期的会话直接丢掉）
fn on_edit_changed(hwnd: HWND, wp: WPARAM) {
    let session = wp.0 as u64;
    let text = platform::edit::text().unwrap_or_default();
    with_host(|host| {
        let Some(edit) = host.rt.edit.as_mut() else { return };
        if edit.id != session {
            return;
        }
        edit.text = text;
        edit.select_all = false;
    });
    invalidate(hwnd);
}

/// 回车 / 失焦：提交这次编辑
fn on_edit_commit(hwnd: HWND, wp: WPARAM) {
    let session = wp.0 as u64;
    with_host(|host| {
        if host.rt.edit.as_ref().map(|edit| edit.id) != Some(session) {
            return;
        }
        let now = now_ms();
        let mut effects = Effects::default();
        app_actions::commit_edit(host, &mut effects, now);
        apply_effects(host, effects, now);
    });
    invalidate(hwnd);
}

/// 转义：取消这次编辑
fn on_edit_cancel(hwnd: HWND, wp: WPARAM) {
    let session = wp.0 as u64;
    with_host(|host| {
        if host.rt.edit.as_ref().map(|edit| edit.id) != Some(session) {
            return;
        }
        app_actions::cancel_edit(host);
    });
    invalidate(hwnd);
}

/// 模态对话框（取色 / 选提示音文件）必须出了 with_host 借用再打开，
/// 否则对话框自己的消息循环会重入 WndProc，触发 RefCell 双借用直接崩溃。
fn run_pending_dialog(hwnd: HWND) {
    let mut pending = None;
    let mut custom = [0u32; 16];
    with_host(|host| {
        pending = host.rt.pending.take();
        custom = std::mem::take(&mut host.rt.custom_colors);
        host.dragging = false;
        host.rt.drag = None;
    });
    let Some(pending) = pending else { return };
    match pending {
        PendingDialog::Color(target, initial) => {
            let picked = platform::color_picker::pick_color(hwnd, initial, &mut custom);
            with_host(|host| {
                host.rt.custom_colors = custom;
                if let Some(color) = picked {
                    app_actions::apply_color(host, target, color);
                }
            });
        }
        PendingDialog::SoundFile => {
            let picked = platform::file::pick_sound_file(hwnd);
            with_host(|host| {
                if let Some(path) = picked {
                    app_actions::apply_sound_path(host, path);
                }
            });
        }
        // 建 / 拆悬浮窗必须在借用之外：CreateWindowExW 会同步回调 WndProc
        PendingDialog::Widget(false) => platform::widget::hide(),
        PendingDialog::Widget(true) => platform::widget::show(),
        PendingDialog::WidgetRefresh => platform::widget::refresh(),
        PendingDialog::WidgetStyle => {
            platform::widget::apply_click_through();
            platform::widget::apply_topmost();
        }
    }
    invalidate(hwnd);
}


fn on_tray(hwnd: HWND, lp: LPARAM) {
    let event = (lp.0 & 0xFFFF) as u32;
    // 循环提示音：点系统气泡即结束
    if event == NIN_BALLOONUSERCLICK {
        with_host(|host| {
            platform::sound::stop();
            host.rt.sound_stopped();
        });
        invalidate(hwnd);
        return;
    }
    match event {
        WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
        }
        WM_RBUTTONUP => {
            let command = show_tray_menu(hwnd);
            if command == 0 {
                return;
            }
            if command == platform::tray::CMD_WIDGET {
                platform::widget::toggle();
                invalidate(hwnd);
                return;
            }
            let now = now_ms();
            let mut restyle = false;
            with_host(|host| {
                let mut effects = Effects::default();
                match command {
                    platform::tray::CMD_SHOW => unsafe {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                        let _ = SetForegroundWindow(hwnd);
                    },
                    platform::tray::CMD_TOGGLE => {
                        if host.app.timer.status == crate::model::RunStatus::Running
                            || host.app.pomodoro.status == crate::model::RunStatus::Running
                        {
                            host.app.timer.toggle(now);
                            host.app.pomodoro.toggle(now);
                        } else {
                            host.app.pomodoro.toggle(now);
                        }
                    }
                    platform::tray::CMD_SKIP => {
                        host.app.skip_pomodoro(now, &mut effects);
                    }
                    platform::tray::CMD_CLICK_THROUGH => {
                        // 穿透开着时悬浮窗自己点不到，只能从这里关回来
                        host.app.visual.widget_click_through = !host.app.visual.widget_click_through;
                        host.dirty = true;
                        restyle = true;
                    }
                    platform::tray::CMD_QUIT => unsafe {
                        let _ = DestroyWindow(hwnd);
                    },
                    _ => {}
                }
                apply_effects(host, effects, now);
            });
            if restyle {
                platform::widget::apply_click_through();
            }
        }
        _ => {}
    }
    invalidate(hwnd);
}

fn show_tray_menu(hwnd: HWND) -> usize {
    unsafe {
        let Ok(menu) = CreatePopupMenu() else { return 0 };
        let toggle_label = with_host_toggle_label();
        let widget_label = if query_host(|host| host.app.visual.widget_visible).unwrap_or(false) {
            "删除桌面小图标"
        } else {
            "显示桌面小图标"
        };
        let click_label = if query_host(|host| host.app.visual.widget_click_through).unwrap_or(false) {
            "小图标点击穿透：开（从托盘关）"
        } else {
            "小图标点击穿透：关"
        };
        let _ = AppendMenuW(menu, MF_STRING, platform::tray::CMD_SHOW, PCWSTR(wide_ptr("显示窗口").as_ptr()));
        let _ = AppendMenuW(menu, MF_STRING, platform::tray::CMD_WIDGET, PCWSTR(wide_ptr(widget_label).as_ptr()));
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            platform::tray::CMD_CLICK_THROUGH,
            PCWSTR(wide_ptr(click_label).as_ptr()),
        );
        let _ = AppendMenuW(menu, MF_STRING, platform::tray::CMD_TOGGLE, PCWSTR(wide_ptr(&toggle_label).as_ptr()));
        let _ = AppendMenuW(menu, MF_STRING, platform::tray::CMD_SKIP, PCWSTR(wide_ptr("跳过当前阶段").as_ptr()));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(menu, MF_STRING, platform::tray::CMD_QUIT, PCWSTR(wide_ptr("退出").as_ptr()));
        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
            point.x,
            point.y,
            None,
            hwnd,
            None,
        );
        let _ = DestroyMenu(menu);
        command.0 as usize
    }
}

fn with_host_toggle_label() -> String {
    let mut label = "开始番茄钟".to_string();
    with_host(|host| {
        if host.app.pomodoro.status == crate::model::RunStatus::Running {
            label = "暂停番茄钟".to_string();
        }
    });
    label
}

fn wide_ptr(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_GETMINMAXINFO => {
            let scale = {
                let mut value = 1.0_f32;
                with_host(|host| value = host.ui.scale.max(1.0));
                value
            };
            let info = unsafe { &mut *(lp.0 as *mut MINMAXINFO) };
            info.ptMinTrackSize.x = (430.0 * scale).round() as i32;
            info.ptMinTrackSize.y = (500.0 * scale).round() as i32;
            LRESULT(0)
        }
        WM_SIZE => {
            resize(hwnd);
            LRESULT(0)
        }
        WM_DPICHANGED => {
            let rect = unsafe { *(lp.0 as *const RECT) };
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            resize(hwnd);
            LRESULT(0)
        }
        WM_PAINT => {
            paint(hwnd);
            LRESULT(0)
        }
        WM_TIMER => {
            on_timer(hwnd);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            on_mouse_move(hwnd, lp);
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            with_host(|host| {
                host.tracking_leave = false;
                host.ui.mouse = (-1.0, -1.0);
            });
            invalidate(hwnd);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            on_click(hwnd);
            run_pending_dialog(hwnd);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            on_release(hwnd);
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            on_tray(hwnd, lp);
            LRESULT(0)
        }
        WM_PENDING => {
            run_pending_dialog(hwnd);
            LRESULT(0)
        }
        platform::tray::WM_TRAY => {
            on_tray(hwnd, lp);
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            on_wheel(hwnd, wp);
            LRESULT(0)
        }
        // 文字输入全部由内嵌的 EDIT 控件负责，它的通知回到这里
        platform::edit::WM_EDIT_CHANGED => {
            on_edit_changed(hwnd, wp);
            LRESULT(0)
        }
        platform::edit::WM_EDIT_COMMIT => {
            on_edit_commit(hwnd, wp);
            LRESULT(0)
        }
        platform::edit::WM_EDIT_CANCEL => {
            on_edit_cancel(hwnd, wp);
            LRESULT(0)
        }
        WM_CLOSE => {
            // 关主窗口只收进托盘：桌面小图标继续用，托盘右键可以删掉它或退出
            with_host(|host| {
                store::save(&host.app);
                unsafe {
                    let _ = ShowWindow(host.hwnd, SW_HIDE);
                }
            });
            LRESULT(0)
        }
        WM_DESTROY => {
            with_host(|host| platform::tray::remove(host.hwnd, host.icon));
            platform::sound::stop();
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wp, lp) },
    }
}

fn main() {
    platform::window::enable_dpi_awareness();
    let app = store::load();
    let rt = Runtime::new(&app);
    let Ok(ui) = Ui::new() else { return };
    unsafe {
        let Ok(instance) = GetModuleHandleW(None) else { return };
        let hinstance = HINSTANCE(instance.0);
        let icon = platform::icon::tomato_icon(32);
        let cursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let class = wine(platform::window::CLASS_NAME);
        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance,
            hIcon: icon,
            hIconSm: icon,
            hCursor: cursor,
            lpszClassName: PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&window_class);

        // 以 DIP 为单位计算初始窗口尺寸，并按屏幕工作区收敛
        let style = WS_OVERLAPPEDWINDOW;
        let system_dpi = GetDpiForSystem().max(96) as u32;
        let scale = system_dpi as f32 / 96.0;
        let (dip_w, dip_h) = WINDOW_SIZES
            .get(app.visual.window_size)
            .map(|entry| (entry.0, entry.1))
            .unwrap_or((WINDOW_SIZES[DEFAULT_WINDOW_SIZE].0, WINDOW_SIZES[DEFAULT_WINDOW_SIZE].1));
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: (dip_w * scale).round() as i32,
            bottom: (dip_h * scale).round() as i32,
        };
        let _ = AdjustWindowRectExForDpi(&mut rect, style, false, WINDOW_EX_STYLE(0), system_dpi);
        let mut work = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work as *mut RECT as *mut core::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        let work_w = (work.right - work.left).max(600);
        let work_h = (work.bottom - work.top).max(500);
        let width = (rect.right - rect.left).min(work_w - 40);
        let height = (rect.bottom - rect.top).min(work_h - 60);
        let x = work.left + (work_w - width) / 2;
        let y = work.top + ((work_h - height) / 2 - 10).max(0);
        let title = wine(platform::window::APP_TITLE);
        let Ok(hwnd) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            style,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(hinstance),
            None,
        ) else {
            return;
        };

        HOST.with(|cell| {
            *cell.borrow_mut() = Some(Host {
                ui,
                app,
                rt,
                hwnd,
                icon,
                dirty: false,
                fast: false,
                dragging: false,
                tracking_leave: false,
                interval: FAST_INTERVAL,
            });
        });
        with_host(|host| host.ui.hide_all_text = host.app.visual.hide_all_text);

        let dark = {
            let mut value = false;
            with_host(|host| value = host.rt.dark);
            value
        };
        platform::window::set_dark_caption(hwnd, dark);
        platform::window::set_rounded_corners(hwnd);
        platform::tray::add(hwnd, icon, platform::window::APP_TITLE);
        SetTimer(Some(hwnd), TIMER_ID, FAST_INTERVAL, None);
        with_host(|host| host.fast = true);
        let _ = ShowWindow(hwnd, SW_SHOW);
        resize(hwnd);

        if query_host(|host| host.app.visual.widget_visible).unwrap_or(false) {
            platform::widget::show();
        }

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

fn wine(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}







