use serde::{Deserialize, Serialize};

pub use crate::color::ThemeColors;
use crate::clock;

pub const STATE_VERSION: u32 = 1;

/// 主窗口 4 档默认尺寸（DIP）：紧凑 / 小 / 标准 / 大
pub const WINDOW_SIZES: [(f32, f32, &str); 4] = [
    (460.0, 540.0, "紧凑"),
    (490.0, 600.0, "小"),
    (520.0, 650.0, "标准"),
    (600.0, 760.0, "大"),
];

pub const DEFAULT_WINDOW_SIZE: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Focus,
    ShortRest,
    LongRest,
}

impl Default for Stage {
    fn default() -> Self {
        Stage::Focus
    }
}

impl Stage {
    pub const ALL: [Stage; 3] = [Stage::Focus, Stage::ShortRest, Stage::LongRest];

    pub fn is_focus(self) -> bool {
        self == Stage::Focus
    }

    pub fn kind(self) -> Kind {
        match self {
            Stage::Focus => Kind::Focus,
            Stage::ShortRest => Kind::ShortRest,
            Stage::LongRest => Kind::LongRest,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Stage::Focus => "专注",
            Stage::ShortRest => "短休息",
            Stage::LongRest => "长休息",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Stage::Focus => 0,
            Stage::ShortRest => 1,
            Stage::LongRest => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Group {
    Focus,
    Rest,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Focus,
    Rest,
    ShortRest,
    LongRest,
}

impl Default for Kind {
    fn default() -> Self {
        Kind::Focus
    }
}

impl Kind {
    pub fn group(self) -> Group {
        match self {
            Kind::Focus => Group::Focus,
            _ => Group::Rest,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Focus => "专注",
            Kind::Rest => "休息",
            Kind::ShortRest => "短休息",
            Kind::LongRest => "长休息",
        }
    }

    pub fn from_stage(stage: Stage) -> Kind {
        stage.kind()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RunStatus {
    #[default]
    Idle,
    Running,
    Paused,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SoundKind {
    Bell,
    Ding,
    Chime,
    Custom,
}

impl Default for SoundKind {
    fn default() -> Self {
        SoundKind::Bell
    }
}

impl SoundKind {
    pub const ALL: [SoundKind; 4] = [SoundKind::Bell, SoundKind::Ding, SoundKind::Chime, SoundKind::Custom];

    pub fn label(self) -> &'static str {
        match self {
            SoundKind::Bell => "铃声",
            SoundKind::Ding => "叮咚",
            SoundKind::Chime => "风铃",
            SoundKind::Custom => "自定义",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Timer,
    Pomodoro,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VisualMode {
    Simple,
    Bar,
    Ring,
}

impl Default for VisualMode {
    fn default() -> Self {
        VisualMode::Simple
    }
}

impl VisualMode {
    pub const ALL: [VisualMode; 3] = [VisualMode::Simple, VisualMode::Bar, VisualMode::Ring];

    pub fn label(self) -> &'static str {
        match self {
            VisualMode::Simple => "简洁",
            VisualMode::Bar => "条形",
            VisualMode::Ring => "环形",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ThemeId {
    #[serde(rename = "A")]
    A,
    #[serde(rename = "B")]
    B,
    #[serde(rename = "E")]
    E,
    #[serde(rename = "b3")]
    B3,
    #[serde(rename = "b4")]
    B4,
    #[serde(rename = "b5")]
    B5,
    #[serde(rename = "b6")]
    B6,
    #[serde(rename = "b7")]
    B7,
    #[serde(rename = "b8")]
    B8,
    #[serde(rename = "custom")]
    Custom,
    #[serde(rename = "saved")]
    Saved,
}

impl Default for ThemeId {
    fn default() -> Self {
        ThemeId::A
    }
}

impl ThemeId {
    /// 颜色设置里「系统预设」的展示顺序
    pub const BUILTIN: [ThemeId; 9] = [
        ThemeId::A,
        ThemeId::B,
        ThemeId::E,
        ThemeId::B3,
        ThemeId::B4,
        ThemeId::B5,
        ThemeId::B6,
        ThemeId::B7,
        ThemeId::B8,
    ];

    pub fn colors(self, dark: bool) -> ThemeColors {
        match self {
            ThemeId::A => ThemeColors::builtin("A", dark),
            ThemeId::B => ThemeColors::builtin("B", dark),
            ThemeId::E => ThemeColors::builtin("E", dark),
            ThemeId::B3 => ThemeColors::scheme(3, dark),
            ThemeId::B4 => ThemeColors::scheme(4, dark),
            ThemeId::B5 => ThemeColors::scheme(5, dark),
            ThemeId::B6 => ThemeColors::scheme(6, dark),
            ThemeId::B7 => ThemeColors::scheme(7, dark),
            ThemeId::B8 => ThemeColors::scheme(8, dark),
            ThemeId::Custom | ThemeId::Saved => ThemeColors::builtin("A", dark),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PresetField {
    Focus,
    ShortRest,
    LongRest,
    Cycle,
    FocusEvent,
    ShortRestEvent,
    LongRestEvent,
    EndSound,
    AutoFocus,
    AutoRest,
    SoundLoop,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preset {
    pub name: String,
    pub focus: u32,
    pub short_rest: u32,
    pub long_rest: u32,
    pub cycle: u32,
    pub focus_event: String,
    pub short_rest_event: String,
    pub long_rest_event: String,
    pub end_sound: SoundKind,
    pub auto_focus: bool,
    pub auto_rest: bool,
    pub sound_loop: bool,
}

impl Default for Preset {
    fn default() -> Self {
        Preset {
            name: "新方案".into(),
            focus: 25,
            short_rest: 5,
            long_rest: 15,
            cycle: 4,
            focus_event: "写代码".into(),
            short_rest_event: "喝水".into(),
            long_rest_event: "午休".into(),
            end_sound: SoundKind::Bell,
            auto_focus: false,
            auto_rest: false,
            sound_loop: false,
        }
    }
}

impl Preset {
    pub fn classic() -> Self {
        Preset { name: "经典番茄".into(), ..Default::default() }
    }

    pub fn deep_work() -> Self {
        Preset {
            name: "深度工作".into(),
            focus: 50,
            short_rest: 10,
            long_rest: 20,
            cycle: 3,
            short_rest_event: "散步".into(),
            end_sound: SoundKind::Chime,
            ..Default::default()
        }
    }

    pub fn minutes_for(&self, stage: Stage) -> u32 {
        match stage {
            Stage::Focus => self.focus,
            Stage::ShortRest => self.short_rest,
            Stage::LongRest => self.long_rest,
        }
        .clamp(0, 999)
    }

    pub fn event_for(&self, stage: Stage) -> &str {
        match stage {
            Stage::Focus => &self.focus_event,
            Stage::ShortRest => &self.short_rest_event,
            Stage::LongRest => &self.long_rest_event,
        }
    }

    pub fn sound(&self) -> SoundKind {
        self.end_sound
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Visual {
    pub mode: VisualMode,
    pub hide_number: bool,
    pub hide_all_text: bool,
    pub dark_mode: bool,
    pub follow_system: bool,
    pub theme: ThemeId,
    pub custom_index: usize,
    pub saved_index: usize,
    pub custom_themes: Vec<ThemeColors>,
    pub saved_presets: Vec<ThemeColors>,
    pub pomo_font_family: String,
    pub pomo_font_size: f32,
    pub pomo_font_weight: u32,
    pub sound_path: String,
    /// 无背景界面：不绘制外层卡片，只保留必要的卡片
    pub bare_ui: bool,
    /// 主窗口尺寸档位（0..3，见 WINDOW_SIZES）
    pub window_size: usize,
    /// 桌面小图标（悬浮窗）是否显示
    pub widget_visible: bool,
    /// 桌面小图标位置（屏幕坐标，None = 未放置）
    pub widget_pos: Option<(i32, i32)>,
    /// 点击穿透（不挡桌面操作）
    pub widget_click_through: bool,
    /// 固定位置（开启后拖不动）
    pub widget_locked: bool,
    /// 始终置顶
    pub widget_topmost: bool,
    /// 桌面小图标整体缩放（0.6 - 1.8）
    pub widget_scale: f32,
    /// 桌面小图标数字字号（DIP）
    pub widget_font_size: f32,
    /// 桌面小图标数字粗细
    pub widget_font_weight: u32,
}

impl Default for Visual {
    fn default() -> Self {
        Visual {
            mode: VisualMode::Simple,
            hide_number: false,
            hide_all_text: false,
            dark_mode: false,
            follow_system: false,
            theme: ThemeId::A,
            custom_index: 0,
            saved_index: 0,
            custom_themes: Vec::new(),
            saved_presets: Vec::new(),
            pomo_font_family: "Microsoft YaHei UI".into(),
            pomo_font_size: 52.0,
            pomo_font_weight: 400,
            sound_path: String::new(),
            bare_ui: true,
            window_size: DEFAULT_WINDOW_SIZE,
            widget_visible: false,
            widget_pos: None,
            widget_click_through: false,
            widget_locked: false,
            widget_topmost: true,
            widget_scale: 1.0,
            widget_font_size: 40.0,
            widget_font_weight: 600,
        }
    }
}

impl Visual {
    pub fn effective_dark(&self, system_dark: bool) -> bool {
        if self.follow_system {
            system_dark
        } else {
            self.dark_mode
        }
    }

    pub fn theme_colors(&self, dark: bool) -> ThemeColors {
        match self.theme {
            ThemeId::Custom => self
                .custom_themes
                .get(self.custom_index)
                .cloned()
                .unwrap_or_else(|| ThemeColors::builtin("A", dark)),
            ThemeId::Saved => self
                .saved_presets
                .get(self.saved_index)
                .cloned()
                .unwrap_or_else(|| ThemeColors::builtin("A", dark)),
            other => other.colors(dark),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Filter {
    pub group: Option<Group>,
    pub event: Option<String>,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
}

impl Filter {
    pub fn matches(&self, item: &HistoryItem) -> bool {
        if let Some(group) = self.group {
            if item.kind.group() != group {
                return false;
            }
        }
        if let Some(event) = &self.event {
            if &item.name != event {
                return false;
            }
        }
        if let Some(start) = &self.date_start {
            if &item.date < start {
                return false;
            }
        }
        if let Some(end) = &self.date_end {
            if &item.date > end {
                return false;
            }
        }
        true
    }

    pub fn is_empty(&self) -> bool {
        self.group.is_none() && self.event.is_none() && self.date_start.is_none() && self.date_end.is_none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: u64,
    pub name: String,
    pub kind: Kind,
    pub seconds: u64,
    pub date: String,
    pub time: String,
    #[serde(default)]
    pub completed: bool,
    pub source: Source,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TimerState {
    pub status: RunStatus,
    pub mode: Kind,
    pub activity: String,
    pub accumulated_ms: u64,
    #[serde(skip)]
    pub started_at: Option<u64>,
}

impl TimerState {
    pub fn fresh() -> Self {
        TimerState {
            status: RunStatus::Idle,
            mode: Kind::Focus,
            activity: "写代码".into(),
            accumulated_ms: 0,
            started_at: None,
        }
    }

    pub fn elapsed_ms(&self, now: u64) -> u64 {
        self.accumulated_ms + self.started_at.map_or(0, |t| now.saturating_sub(t))
    }

    pub fn elapsed_sec(&self, now: u64) -> u64 {
        self.elapsed_ms(now) / 1000
    }

    pub fn toggle(&mut self, now: u64) {
        match self.status {
            RunStatus::Running => {
                self.accumulated_ms = self.elapsed_ms(now);
                self.started_at = None;
                self.status = RunStatus::Paused;
            }
            _ => {
                self.started_at = Some(now);
                self.status = RunStatus::Running;
            }
        }
    }

    pub fn reset(&mut self) {
        self.accumulated_ms = 0;
        self.started_at = None;
        self.status = RunStatus::Idle;
    }

    pub fn normalize(&mut self) {
        if self.status == RunStatus::Running {
            self.started_at = None;
            self.status = RunStatus::Paused;
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PomodoroState {
    pub status: RunStatus,
    pub stage: Stage,
    pub activity: String,
    pub cycle: u32,
    pub total_ms: u64,
    pub remaining_ms: u64,
    #[serde(skip)]
    pub deadline: Option<u64>,
    #[serde(skip)]
    pub auto_start_at: Option<u64>,
}

impl PomodoroState {
    pub fn fresh(preset: &Preset) -> Self {
        let total = preset.minutes_for(Stage::Focus) as u64 * 60_000;
        PomodoroState {
            status: RunStatus::Idle,
            stage: Stage::Focus,
            activity: preset.focus_event.clone(),
            cycle: 1,
            total_ms: total,
            remaining_ms: total,
            deadline: None,
            auto_start_at: None,
        }
    }

    pub fn remaining_ms(&self, now: u64) -> u64 {
        match (self.status, self.deadline) {
            (RunStatus::Running, Some(deadline)) => deadline.saturating_sub(now),
            _ => self.remaining_ms,
        }
    }

    pub fn remaining_sec(&self, now: u64) -> u64 {
        self.remaining_ms(now).div_ceil(1000)
    }

    pub fn total_sec(&self) -> u64 {
        self.total_ms / 1000
    }

    pub fn elapsed_sec(&self, now: u64) -> u64 {
        self.total_sec().saturating_sub(self.remaining_sec(now))
    }

    /// 进度：按整秒跳变（图形倒计时一顿一顿，不跟着每帧连续推进）
    pub fn progress(&self, now: u64) -> f32 {
        let total = self.total_sec().max(1);
        (self.remaining_sec(now) as f32 / total as f32).clamp(0.0, 1.0)
    }

    pub fn toggle(&mut self, now: u64) {
        match self.status {
            RunStatus::Running => {
                self.remaining_ms = self.remaining_ms(now);
                self.deadline = None;
                self.status = RunStatus::Paused;
            }
            _ => {
                if self.remaining_ms == 0 {
                    self.remaining_ms = self.total_ms;
                }
                self.deadline = Some(now + self.remaining_ms);
                self.status = RunStatus::Running;
            }
        }
    }

    pub fn stop(&mut self) {
        self.deadline = None;
        self.status = RunStatus::Idle;
        self.auto_start_at = None;
    }

    pub fn normalize(&mut self) {
        self.deadline = None;
        self.auto_start_at = None;
        if self.status == RunStatus::Running {
            self.status = RunStatus::Paused;
        }
        if self.total_ms == 0 {
            self.total_ms = self.remaining_ms.max(60_000);
        }
        self.remaining_ms = self.remaining_ms.min(self.total_ms);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct App {
    pub version: u32,
    pub presets: Vec<Preset>,
    pub active_preset: usize,
    pub activities: Vec<String>,
    pub history: Vec<HistoryItem>,
    pub filter: Filter,
    pub visual: Visual,
    pub timer: TimerState,
    pub pomodoro: PomodoroState,
}

impl Default for App {
    fn default() -> Self {
        let presets = vec![Preset::classic(), Preset::deep_work()];
        let pomodoro = PomodoroState::fresh(&presets[0]);
        App {
            version: STATE_VERSION,
            presets,
            active_preset: 0,
            activities: vec![
                "写代码".into(),
                "阅读".into(),
                "写作".into(),
                "学习".into(),
                "游戏".into(),
                "散步".into(),
                "喝水".into(),
                "听音乐".into(),
                "午休".into(),
            ],
            history: Vec::new(),
            filter: Filter::default(),
            visual: Visual::default(),
            timer: TimerState::fresh(),
            pomodoro,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Effect {
    Toast(String),
    Sound { kind: SoundKind, path: String, looping: bool },
    Balloon { title: String, body: String },
}

#[derive(Default)]
pub struct Effects {
    pub items: Vec<Effect>,
}

impl Effects {
    pub fn toast(&mut self, text: impl Into<String>) {
        self.items.push(Effect::Toast(text.into()));
    }

    pub fn sound(&mut self, kind: SoundKind, path: &str, looping: bool) {
        self.items.push(Effect::Sound { kind, path: path.to_string(), looping });
    }

    pub fn balloon(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.items.push(Effect::Balloon { title: title.into(), body: body.into() });
    }
}

pub fn fmt_time(seconds: u64) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

pub fn fmt_duration(seconds: u64) -> String {
    let m = seconds / 60;
    let s = seconds % 60;
    if s > 0 {
        format!("{}分{}秒", m, s)
    } else {
        format!("{}分", m)
    }
}

impl App {
    pub fn preset(&self) -> &Preset {
        let idx = self.active_preset.min(self.presets.len().saturating_sub(1));
        &self.presets[idx]
    }

    pub fn preset_mut(&mut self) -> &mut Preset {
        let idx = self.active_preset.min(self.presets.len().saturating_sub(1));
        &mut self.presets[idx]
    }

    pub fn select_preset(&mut self, index: usize) {
        if index < self.presets.len() {
            self.active_preset = index;
        }
    }

    pub fn add_preset(&mut self) {
        self.presets.push(Preset::default());
        self.active_preset = self.presets.len() - 1;
    }

    pub fn delete_preset(&mut self, fx: &mut Effects) {
        if self.presets.len() <= 1 {
            fx.toast("至少保留一个方案");
            return;
        }
        let idx = self.active_preset.min(self.presets.len() - 1);
        self.presets.remove(idx);
        if self.active_preset >= self.presets.len() {
            self.active_preset = 0;
        }
        fx.toast("已删除方案");
    }

    pub fn next_history_id(&self) -> u64 {
        self.history.iter().map(|h| h.id).max().unwrap_or(0) + 1
    }

    /// 重新套用当前预设：重置阶段剩余时间并回到空闲。
    /// 时长为 0 的阶段表示「跳过」，会自动推进到下一个非 0 阶段。
    pub fn apply_preset(&mut self) {
        for _ in 0..3 {
            if self.preset().minutes_for(self.pomodoro.stage) > 0 {
                break;
            }
            self.advance_stage();
        }
        let stage = self.pomodoro.stage;
        let minutes = self.preset().minutes_for(stage).max(1);
        self.pomodoro.total_ms = minutes as u64 * 60_000;
        self.pomodoro.remaining_ms = self.pomodoro.total_ms;
        self.pomodoro.status = RunStatus::Idle;
        self.pomodoro.deadline = None;
        self.pomodoro.auto_start_at = None;
        self.map_activity();
    }

    /// 编辑方案参数后：仅在空闲且未开始时重新套用
    pub fn refresh_preset_if_idle(&mut self, now: u64) {
        if self.pomodoro.status == RunStatus::Idle && self.pomodoro.remaining_ms(now) == self.pomodoro.total_ms {
            self.apply_preset();
        }
    }

    pub fn map_activity(&mut self) {
        let stage = self.pomodoro.stage;
        let mapped = self.preset().event_for(stage).to_string();
        if !mapped.is_empty() {
            self.pomodoro.activity = mapped;
        }
    }

    pub fn advance_stage(&mut self) {
        let cycle_target = self.preset().cycle.max(1);
        match self.pomodoro.stage {
            Stage::Focus => self.pomodoro.stage = Stage::ShortRest,
            Stage::ShortRest => {
                self.pomodoro.stage = Stage::Focus;
                self.pomodoro.cycle += 1;
                if self.pomodoro.cycle > cycle_target {
                    self.pomodoro.stage = Stage::LongRest;
                    self.pomodoro.cycle = 1;
                }
            }
            Stage::LongRest => {
                self.pomodoro.stage = Stage::Focus;
                self.pomodoro.cycle = 1;
            }
        }
    }

    pub fn add_history(
        &mut self,
        name: String,
        kind: Kind,
        seconds: u64,
        completed: bool,
        source: Source,
        _now: u64,
    ) -> HistoryItem {
        let item = HistoryItem {
            id: self.next_history_id(),
            name,
            kind,
            seconds,
            date: clock::today(),
            time: clock::stamp(),
            completed,
            source,
        };
        self.history.insert(0, item.clone());
        item
    }

    pub fn finish_pomodoro(&mut self, completed: bool, now: u64, fx: &mut Effects) {
        let elapsed_sec = self.pomodoro.elapsed_sec(now);
        let stage = self.pomodoro.stage;
        let activity = self.pomodoro.activity.clone();
        self.pomodoro.stop();

        if elapsed_sec > 0 {
            self.add_history(activity.clone(), stage.kind(), elapsed_sec, completed, Source::Pomodoro, now);
            let text = if completed {
                format!("完成！{} · {}", activity, fmt_duration(elapsed_sec))
            } else {
                format!("已结束并保存：{} · {}", activity, fmt_duration(elapsed_sec))
            };
            fx.toast(text);
        }

        if completed {
            self.advance_stage();
            let preset = self.preset().clone();
            let auto = if self.pomodoro.stage.is_focus() { preset.auto_focus } else { preset.auto_rest };
            self.apply_preset();
            if auto {
                self.pomodoro.auto_start_at = Some(now + 500);
            }
        } else {
            self.apply_preset();
        }
    }

    /// 跳过：跑着的话先把这一段记进历史，然后不管跑没跑都进下一阶段
    /// （专注 → 短休息 → 专注 …，跑满一个循环进长休息）
    pub fn skip_pomodoro(&mut self, now: u64, fx: &mut Effects) {
        if self.pomodoro.status != RunStatus::Idle {
            self.finish_pomodoro(false, now, fx);
        }
        self.advance_stage();
        // 和「阶段自然结束」一致：方案里开了自动开始就顺手起下一阶段
        let preset = self.preset().clone();
        let auto = if self.pomodoro.stage.is_focus() { preset.auto_focus } else { preset.auto_rest };
        self.apply_preset();
        self.pomodoro.auto_start_at = if auto { Some(now + 500) } else { None };
        fx.toast(format!("已跳到「{}」", self.pomodoro.stage.name()));
    }

    pub fn pomodoro_reset(&mut self) {
        self.pomodoro.stop();
        self.apply_preset();
    }

    pub fn pomodoro_sound(&self) -> (SoundKind, String, bool) {
        let preset = self.preset();
        (preset.sound(), self.visual.sound_path.clone(), preset.sound_loop)
    }

    /// 每秒（或更短间隔）调用，推进所有计时状态
    pub fn tick(&mut self, now: u64, fx: &mut Effects) {
        if self.pomodoro.status == RunStatus::Running {
            if let Some(deadline) = self.pomodoro.deadline {
                if now >= deadline {
                    let (kind, path, looping) = self.pomodoro_sound();
                    fx.sound(kind, &path, looping);
                    let stage = self.pomodoro.stage;
                    let activity = self.pomodoro.activity.clone();
                    fx.balloon(
                        format!("{}结束", stage.name()),
                        format!("{} · {} 已结束", activity, stage.name()),
                    );
                    self.finish_pomodoro(true, now, fx);
                }
            }
        }
        if let Some(at) = self.pomodoro.auto_start_at {
            if now >= at {
                self.pomodoro.auto_start_at = None;
                if self.pomodoro.status == RunStatus::Idle {
                    self.pomodoro.toggle(now);
                    fx.toast("自动开始下一个阶段");
                }
            }
        }
    }

    pub fn timer_end(&mut self, now: u64, fx: &mut Effects) {
        let seconds = self.timer.elapsed_sec(now);
        let activity = self.timer.activity.clone();
        let mode = self.timer.mode;
        self.timer.reset();
        if seconds > 0 {
            self.add_history(activity.clone(), mode, seconds, false, Source::Timer, now);
            fx.toast(format!("已保存：{} · {}", activity, fmt_duration(seconds)));
        } else {
            fx.toast("已跳过");
        }
    }

    /// 跳过：切到另一个状态（专注 ↔ 休息）。
    /// 跑着的时候这个按钮是「结束」——只存历史 + 停止，不碰模式。
    pub fn timer_skip(&mut self, now: u64, fx: &mut Effects) {
        if self.timer.status != RunStatus::Idle {
            self.timer_end(now, fx);
            return;
        }
        self.timer.reset();
        self.timer.mode = match self.timer.mode {
            Kind::Rest => Kind::Focus,
            _ => Kind::Rest,
        };
        fx.toast(format!("已切到「{}」", self.timer.mode.name()));
    }

    pub fn minutes_of(&self, items: &[&HistoryItem]) -> (u32, u32) {
        let mut focus = 0u64;
        let mut rest = 0u64;
        for item in items {
            match item.kind.group() {
                Group::Focus => focus += item.seconds,
                Group::Rest => rest += item.seconds,
            }
        }
        (((focus + 30) / 60) as u32, ((rest + 30) / 60) as u32)
    }

    pub fn today_minutes(&self) -> (u32, u32) {
        let today = clock::today();
        let items: Vec<&HistoryItem> = self.history.iter().filter(|h| h.date == today).collect();
        self.minutes_of(&items)
    }

    pub fn visible_history(&self) -> Vec<&HistoryItem> {
        self.history.iter().filter(|h| self.filter.matches(h)).collect()
    }

    pub fn delete_selected(&mut self, ids: &[u64]) {
        self.history.retain(|h| !ids.contains(&h.id));
    }

    pub fn has_running_timer(&self) -> bool {
        self.timer.status == RunStatus::Running
            || self.pomodoro.status == RunStatus::Running
            || self.pomodoro.auto_start_at.is_some()
    }

    pub fn normalize(&mut self) {
        if self.presets.is_empty() {
            self.presets.push(Preset::classic());
        }
        if self.active_preset >= self.presets.len() {
            self.active_preset = 0;
        }
        if self.activities.is_empty() {
            self.activities.push("写代码".into());
        }
        if !self.activities.contains(&self.timer.activity) {
            self.timer.activity = self.activities[0].clone();
        }
        if !self.activities.contains(&self.pomodoro.activity) {
            self.pomodoro.activity = self.activities[0].clone();
        }
        self.timer.normalize();
        self.pomodoro.normalize();
        self.visual.window_size = self.visual.window_size.min(WINDOW_SIZES.len() - 1);
        self.visual.pomo_font_size = self.visual.pomo_font_size.clamp(36.0, 80.0);
        self.visual.pomo_font_weight = self.visual.pomo_font_weight.clamp(200, 800);
        if self.visual.custom_index >= self.visual.custom_themes.len() {
            self.visual.custom_index = 0;
        }
        if self.visual.saved_index >= self.visual.saved_presets.len() {
            self.visual.saved_index = 0;
        }
    }
}



