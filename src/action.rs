use crate::model::{Kind, PresetField, Stage, ThemeId, VisualMode};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ViewId {
    Timer,
    Pomodoro,
    History,
    Settings,
}

impl ViewId {
    pub const ALL: [ViewId; 4] = [ViewId::Timer, ViewId::Pomodoro, ViewId::History, ViewId::Settings];

    pub fn index(self) -> usize {
        match self {
            ViewId::Timer => 0,
            ViewId::Pomodoro => 1,
            ViewId::History => 2,
            ViewId::Settings => 3,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ViewId::Timer => "计时",
            ViewId::Pomodoro => "番茄钟",
            ViewId::History => "历史",
            ViewId::Settings => "设置",
        }
    }

    pub fn icon(self) -> &'static str {
        use crate::ui::icons;
        match self {
            ViewId::Timer => icons::WATCH,
            ViewId::Pomodoro => icons::TARGET,
            ViewId::History => icons::LIST,
            ViewId::Settings => icons::GEAR,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Section {
    PomoPreset,
    SettingsPreset,
    SettingsEvent,
    SettingsVisual,
    VisualPomodoro,
    VisualColor,
    VisualText,
    VisualUi,
    SettingsWidget,
}

impl Section {
    pub const ALL: [Section; 9] = [
        Section::PomoPreset,
        Section::SettingsPreset,
        Section::SettingsEvent,
        Section::SettingsVisual,
        Section::VisualPomodoro,
        Section::VisualColor,
        Section::VisualText,
        Section::VisualUi,
        Section::SettingsWidget,
    ];

    pub fn index(self) -> usize {
        match self {
            Section::PomoPreset => 0,
            Section::SettingsPreset => 1,
            Section::SettingsEvent => 2,
            Section::SettingsVisual => 3,
            Section::VisualPomodoro => 4,
            Section::VisualColor => 5,
            Section::VisualText => 6,
            Section::VisualUi => 7,
            Section::SettingsWidget => 8,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DropId {
    Preset,
    FocusEvent,
    ShortRestEvent,
    LongRestEvent,
    SoundKind,
    FontFamily,
    TimerActivity,
    PomoActivity,
    FilterGroup,
    FilterEvent,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NumberTarget {
    Focus,
    ShortRest,
    LongRest,
    Cycle,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditTarget {
    ActivityAdd,
    ActivityName(bool),
    PresetName,
    ThemeName(usize),
    Number(NumberTarget),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorTarget {
    Custom(usize, usize),
    Saved(usize, usize),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    Nav(ViewId),
    TimerMode(Kind),
    TimerToggle,
    TimerSkip,
    TimerReset,
    ActivityEdit(bool),
    PomoStage(Stage),
    PomoToggle,
    PomoSkip,
    PomoReset,
    ToggleSection(Section),
    DropToggle(DropId),
    DropPick(DropId, usize),
    NumberStep(NumberTarget, i32),
    NumberEdit(NumberTarget),
    PresetNameEdit,
    PresetNew,
    PresetSave,
    PresetDelete,
    UploadSound,
    PresetSwitch(PresetField),
    ActivityDelete(usize),
    ActivityAddStart,
    ActivityAddConfirm,
    ActivityAddCancel,
    VisualMode(VisualMode),
    ToggleHideNumber,
    ToggleHideAllText,
    ToggleDarkMode,
    ToggleFollowSystem,
    ThemeBuiltin(ThemeId),
    ThemeCustomUse(usize),
    ThemeCustomDelete(usize),
    ThemeCustomNew,
    ThemeSavedUse(usize),
    ThemeSavedDelete(usize),
    ThemeSavePreset,
    ThemeNameEdit(usize),
    ColorPick(ColorTarget),
    FontFamily(usize),
    SliderFontSize,
    SliderFontWeight,
    ToggleSelect(u64),
    SelectAll(bool),
    BatchDelete,
    FilterReset,
    CalendarToggle,
    CalendarPrev,
    CalendarNext,
    CalendarPickDay(u32),
    CalendarClear,
    CalendarApply,
    StopSound,
    PresetApply,
    ToggleBareUi,
    WindowSize(usize),
    /// 桌面小图标（悬浮窗）
    WidgetEdit,
    WidgetOpen,
    WidgetClose,
    /// 桌面小图标在「运行模式（全部控件）」与「待机模式（只留数字与活动）」之间切换
    WidgetModeToggle,
    /// 重挂悬浮窗：重判当前跟谁跑、重算尺寸、重置心跳并立刻重绘
    WidgetRefresh,
    /// 固定桌面小图标位置（开启后拖不动）
    WidgetLock,
    /// 直接指定悬浮窗跟随哪个模块（小图标上的 计时 / 番茄钟 按钮）
    WidgetModule(ViewId),
    WidgetVisible(bool),
    WidgetClickThrough,
    WidgetTopmost,
    SliderWidgetScale,
    SliderWidgetFontSize,
    SliderWidgetFontWeight,
}

impl Action {
    /// 按住拖动条连续调整的动作（主窗口按下后要进入拖动状态）
    pub fn is_slider(&self) -> bool {
        matches!(
            self,
            Action::SliderFontSize
                | Action::SliderFontWeight
                | Action::SliderWidgetScale
                | Action::SliderWidgetFontSize
                | Action::SliderWidgetFontWeight
        )
    }
}

