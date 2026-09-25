use std::collections::HashMap;

use crate::action::{Action, ColorTarget, DropId, EditTarget, Section, ViewId};
use crate::clock;
use crate::color::{Color, Palette, ThemeColors};
use crate::model::{App, Preset};
use crate::ui::Rect;

pub struct EditState {
    /// 会话号：内嵌输入控件的通知带上它，过期的通知直接丢掉
    pub id: u64,
    pub target: EditTarget,
    pub text: String,
    pub select_all: bool,
}

impl EditState {
    pub fn new(target: EditTarget, text: String, select_all: bool) -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        EditState { id, target, text, select_all }
    }
}

pub struct CalendarState {
    pub year: i32,
    pub month: u32,
    pub start: Option<String>,
    pub end: Option<String>,
}

impl Default for CalendarState {
    fn default() -> Self {
        let (year, month) = clock::year_month();
        CalendarState { year, month, start: None, end: None }
    }
}

/// 需要在消息循环之外打开的模态对话框。
/// 在 with_host 借用期间打开会重入 WndProc，触发 RefCell 双借用而崩溃，所以先记下来、出了借用再开。
pub enum PendingDialog {
    Color(ColorTarget, Color),
    SoundFile,
    /// 显示 / 隐藏桌面小图标：建窗口必须出了借用再做
    Widget(bool),
    /// 重挂桌面小图标（重算尺寸 / 心跳 / 重绘）
    WidgetRefresh,
    /// 重设桌面小图标的窗口样式（点击穿透 / 置顶）
    WidgetStyle,
}

/// 番茄钟视图「计时方案」的草稿：改动只落在这里，点「应用」才写回当前方案。
pub struct PresetDraft {
    pub index: usize,
    pub preset: Preset,
}

impl PresetDraft {
    pub fn from_app(app: &App) -> Self {
        PresetDraft { index: app.active_preset, preset: app.preset().clone() }
    }
}

/// 悬浮窗的三种状态
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WidgetMode {
    /// 待机·未点击：只显示数字 / 图形与活动
    Idle,
    /// 待机·已点击：多出一行图标按钮，但没有 开始 / 结束 / 重置
    Standby,
    /// 运行：图标行 + 控制行都在
    Running,
}

impl WidgetMode {
    /// 是否显示图标行（关闭 / 刷新 / 穿透 / 固定 / 回主界面 / 待机切换）
    pub fn buttons(self) -> bool {
        self != WidgetMode::Idle
    }

    /// 是否显示控制行（开始 / 结束 / 重置）
    pub fn running(self) -> bool {
        self == WidgetMode::Running
    }
}

pub struct Runtime {
    pub view: ViewId,
    pub scroll: HashMap<ViewId, f32>,
    pub scroll_max: HashMap<ViewId, f32>,
    pub sections: [bool; 9],
    pub toast: Option<(String, u64)>,
    pub edit: Option<EditState>,
    pub open_drop: Option<DropId>,
    pub selected: Vec<u64>,
    pub calendar: CalendarState,
    pub calendar_open: bool,
    pub drag: Option<(Action, Rect)>,
    pub custom_colors: [u32; 16],
    pub theme: ThemeColors,
    pub palette: Palette,
    pub dark: bool,
    pub system_dark: bool,
    pub sound_loop_playing: bool,
    pub sound_until: u64,
    pub drop_anchor: Rect,
    pub drop_scroll: f32,
    pub edit_rect: Rect,
    /// 悬浮窗自己的编辑框位置（两个窗口共用 rt.edit，位置得分开记）
    pub widget_edit_rect: Rect,
    pub pending: Option<PendingDialog>,
    pub draft: Option<PresetDraft>,
    pub window_dirty: bool,
    /// 悬浮窗的三态：待机未点击 / 待机已点击 / 运行
    pub widget_mode: WidgetMode,
    /// 悬浮窗当前跟随的模块（主界面切到历史 / 设置时不会跟着乱跳）
    pub widget_module: ViewId,
    /// 上一次「有模块在跑」的观察结果：谁刚开始跑就把小图标切给谁
    pub widget_running_seen: Option<ViewId>,
    /// 当前下拉列表是不是悬浮窗打开的
    pub widget_drop: bool,
    /// 悬浮窗是否正在编辑活动名（决定要不要抢占键盘焦点）
    pub widget_editing: bool,
}

impl Runtime {
    pub fn new(app: &App) -> Self {
        let system_dark = clock::system_dark();
        let dark = app.visual.effective_dark(system_dark);
        let theme = app.visual.theme_colors(dark);
        Runtime {
            view: ViewId::Timer,
            scroll: HashMap::new(),
            scroll_max: HashMap::new(),
            sections: [false; 9],
            toast: None,
            edit: None,
            open_drop: None,
            selected: Vec::new(),
            calendar: CalendarState::default(),
            calendar_open: false,
            drag: None,
            custom_colors: [0x00ff_ffff; 16],
            palette: Palette::new(&theme, dark, app.visual.bare_ui),
            theme,
            dark,
            system_dark,
            sound_loop_playing: false,
            sound_until: 0,
            drop_anchor: Rect::new(0.0, 0.0, 0.0, 0.0),
            drop_scroll: 0.0,
            edit_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            widget_edit_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            pending: None,
            draft: None,
            window_dirty: false,
            widget_mode: WidgetMode::Idle,
            widget_module: ViewId::Timer,
            widget_running_seen: None,
            widget_drop: false,
            widget_editing: false,
        }
    }

    pub fn refresh_theme(&mut self, app: &App) {
        self.system_dark = clock::system_dark();
        self.dark = app.visual.effective_dark(self.system_dark);
        self.theme = app.visual.theme_colors(self.dark);
        self.palette = Palette::new(&self.theme, self.dark, app.visual.bare_ui);
    }

    pub fn scroll_of(&self, view: ViewId) -> f32 {
        *self.scroll.get(&view).unwrap_or(&0.0)
    }

    pub fn set_scroll(&mut self, view: ViewId, value: f32) {
        let max = *self.scroll_max.get(&view).unwrap_or(&0.0);
        self.scroll.insert(view, value.clamp(0.0, max.max(0.0)));
    }

    pub fn is_open(&self, section: Section) -> bool {
        self.sections[section.index()]
    }

    pub fn toggle(&mut self, section: Section) {
        let idx = section.index();
        self.sections[idx] = !self.sections[idx];
    }

    pub fn show_toast(&mut self, text: impl Into<String>, now: u64) {
        self.toast = Some((text.into(), now + 2200));
    }

    pub fn toast_expired(&self, now: u64) -> bool {
        match &self.toast {
            Some((_, until)) => now >= *until,
            None => true,
        }
    }

    /// 提示音是否正在播放（决定是否显示「结束提示音」按钮）
    pub fn sound_active(&self, now: u64) -> bool {
        self.sound_until > now
    }

    pub fn sound_started(&mut self, now: u64, looping: bool) {
        self.sound_loop_playing = looping;
        self.sound_until = if looping { now + 3600_000 } else { now + 2500 };
    }

    pub fn sound_stopped(&mut self) {
        self.sound_loop_playing = false;
        self.sound_until = 0;
    }

    pub fn draft_preset(&self) -> Option<&Preset> {
        self.draft.as_ref().map(|draft| &draft.preset)
    }

    pub fn draft_preset_mut(&mut self) -> Option<&mut Preset> {
        self.draft.as_mut().map(|draft| &mut draft.preset)
    }
}
