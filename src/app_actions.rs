use crate::action::{Action, ColorTarget, EditTarget, NumberTarget, Section, ViewId};
use crate::clock;
use crate::color::Color;
use crate::drops;
use crate::model::{App, Effects, Preset, PresetField, RunStatus, SoundKind, ThemeId, WINDOW_SIZES};
use crate::color::ThemeColors;
use crate::platform;
use crate::runtime::{EditState, PendingDialog, PresetDraft, WidgetMode};
use crate::views::widget_view;
use crate::ui::Rect;
use crate::Host;

pub fn apply(host: &mut Host, action: Action, rect: Rect, fx: &mut Effects, now: u64) {
    host.dirty = true;
    match action {
        Action::Nav(view) => {
            commit_edit(host, fx, now);
            host.rt.view = view;
            // 只有切到这两个「有状态」的视图才让小图标跟着走；
            // 历史和设置没有自己的状态，小图标保持上一个模块（不然会莫名其妙跳到计时）
            if matches!(view, ViewId::Timer | ViewId::Pomodoro) {
                host.rt.widget_module = view;
            }
            host.rt.open_drop = None;
            host.rt.calendar_open = false;
            host.rt.draft = None;
            host.rt.scroll.entry(view).or_insert(0.0);
        }
        Action::TimerMode(kind) => {
            if host.app.timer.status != RunStatus::Idle {
                fx.toast("请先结束或重置当前计时");
            } else {
                host.app.timer.mode = kind;
            }
        }
        Action::TimerToggle => {
            commit_edit(host, fx, now);
            host.app.timer.toggle(now);
        }
        Action::TimerSkip => {
            commit_edit(host, fx, now);
            host.app.timer_skip(now, fx);
        }
        Action::TimerReset => {
            host.app.timer.reset();
        }
        Action::ActivityEdit(pomodoro) => {
            let text = if pomodoro {
                host.app.pomodoro.activity.clone()
            } else {
                host.app.timer.activity.clone()
            };
            begin_edit(host, EditState::new(EditTarget::ActivityName(pomodoro), text, true));
            host.rt.open_drop = None;
        }
        Action::PomoStage(stage) => {
            if host.app.pomodoro.status != RunStatus::Idle {
                fx.toast("请先结束或重置当前计时");
            } else {
                host.app.pomodoro.stage = stage;
                host.app.apply_preset();
            }
        }
        Action::PomoToggle => {
            commit_edit(host, fx, now);
            host.app.pomodoro.auto_start_at = None;
            host.app.pomodoro.toggle(now);
        }
        Action::PomoSkip => {
            commit_edit(host, fx, now);
            host.app.skip_pomodoro(now, fx);
        }
        Action::PomoReset => {
            host.app.pomodoro_reset();
        }
        Action::ToggleSection(section) => {
            host.rt.toggle(section);
            if section == Section::PomoPreset {
                // 番茄钟视图的计时方案走草稿：不改动上方正在跑的番茄钟，直到点「应用」
                if host.rt.is_open(section) && host.rt.view == ViewId::Pomodoro {
                    host.rt.draft = Some(PresetDraft::from_app(&host.app));
                } else {
                    host.rt.draft = None;
                }
            }
        }
        Action::DropToggle(id) => {
            commit_edit(host, fx, now);
            if host.rt.open_drop == Some(id) {
                host.rt.open_drop = None;
            } else {
                host.rt.open_drop = Some(id);
                host.rt.drop_anchor = rect;
                host.rt.drop_scroll = 0.0;
            }
        }
        Action::DropPick(id, index) => {
            drops::pick(&mut host.app, &mut host.rt, id, index);
            host.rt.open_drop = None;
        }
        Action::NumberStep(target, delta) => {
            if draft_scope(host) {
                if let Some(draft) = host.rt.draft_preset_mut() {
                    step_preset(draft, target, delta);
                }
            } else {
                step_number(&mut host.app, target, delta);
                host.app.refresh_preset_if_idle(now);
            }
        }
        Action::NumberEdit(target) => {
            let value = match host.rt.draft_preset() {
                Some(draft) => preset_number(draft, target),
                None => number_value(&host.app, target),
            };
            let text = value.to_string();
            begin_edit(host, EditState::new(EditTarget::Number(target), text, true));
        }
        Action::PresetNew => {
            host.app.add_preset();
            let text = host.app.preset().name.clone();
            begin_edit(host, EditState::new(EditTarget::PresetName, text, true));
            host.rt.open_drop = None;
            host.app.apply_preset();
            fx.toast("已新建方案");
        }
        Action::PresetNameEdit => {
            let text = host.app.preset().name.clone();
            begin_edit(host, EditState::new(EditTarget::PresetName, text, true));
            host.rt.open_drop = None;
        }
        Action::PresetSave => {
            let name = host.app.preset().name.clone();
            store_save(host);
            fx.toast(format!("已保存「{}」", name));
        }
        Action::PresetDelete => {
            host.app.delete_preset(fx);
            host.app.apply_preset();
        }
        Action::UploadSound => {
            // 模态对话框必须出了 with_host 借用再开，否则会重入 WndProc 触发 RefCell 双借用崩溃
            host.rt.pending = Some(PendingDialog::SoundFile);
        }
        Action::PresetSwitch(field) => {
            let preset = host.app.preset_mut();
            match field {
                PresetField::AutoFocus => preset.auto_focus = !preset.auto_focus,
                PresetField::AutoRest => preset.auto_rest = !preset.auto_rest,
                PresetField::SoundLoop => preset.sound_loop = !preset.sound_loop,
                _ => {}
            }
        }
        Action::ActivityDelete(index) => {
            if host.app.activities.len() <= 1 {
                fx.toast("至少保留一个事件");
            } else if index < host.app.activities.len() {
                let removed = host.app.activities.remove(index);
                let fallback = host.app.activities.first().cloned().unwrap_or_default();
                for preset in host.app.presets.iter_mut() {
                    if preset.focus_event == removed {
                        preset.focus_event = fallback.clone();
                    }
                    if preset.short_rest_event == removed {
                        preset.short_rest_event = fallback.clone();
                    }
                    if preset.long_rest_event == removed {
                        preset.long_rest_event = fallback.clone();
                    }
                }
                host.app.normalize();
                host.app.apply_preset();
            }
        }
        Action::ActivityAddStart => {
            begin_edit(host, EditState::new(EditTarget::ActivityAdd, String::new(), false));
        }
        Action::ActivityAddConfirm => {
            commit_edit(host, fx, now);
        }
        Action::ActivityAddCancel => {
            host.rt.edit = None;
        }
        Action::VisualMode(mode) => {
            host.app.visual.mode = mode;
        }
        Action::ToggleHideNumber => {
            host.app.visual.hide_number = !host.app.visual.hide_number;
        }
        Action::ToggleHideAllText => {
            host.app.visual.hide_all_text = !host.app.visual.hide_all_text;
            host.ui.hide_all_text = host.app.visual.hide_all_text;
        }
        Action::ToggleDarkMode => {
            host.app.visual.dark_mode = !host.app.visual.dark_mode;
            host.app.visual.follow_system = false;
            host.rt.refresh_theme(&host.app);
            platform::window::set_dark_caption(host.hwnd, host.rt.dark);
        }
        Action::ToggleFollowSystem => {
            host.app.visual.follow_system = !host.app.visual.follow_system;
            host.rt.refresh_theme(&host.app);
            platform::window::set_dark_caption(host.hwnd, host.rt.dark);
        }
        Action::ThemeBuiltin(id) => {
            host.app.visual.theme = id;
            host.rt.refresh_theme(&host.app);
        }
        Action::ThemeCustomUse(index) => {
            if index < host.app.visual.custom_themes.len() {
                host.app.visual.custom_index = index;
                host.app.visual.theme = ThemeId::Custom;
                host.rt.refresh_theme(&host.app);
                fx.toast("已应用自定义颜色");
            }
        }
        Action::ThemeCustomDelete(index) => {
            if index < host.app.visual.custom_themes.len() {
                host.app.visual.custom_themes.remove(index);
                if host.app.visual.custom_themes.is_empty() {
                    host.app.visual.custom_index = 0;
                    host.app.visual.theme = ThemeId::A;
                } else {
                    host.app.visual.custom_index = host.app.visual.custom_index.min(host.app.visual.custom_themes.len() - 1);
                }
                host.rt.refresh_theme(&host.app);
                fx.toast("已删除自定义颜色");
            }
        }
        Action::ThemeCustomNew => {
            let theme = ThemeColors::custom_default(host.app.visual.custom_themes.len());
            host.app.visual.custom_themes.push(theme);
            host.app.visual.custom_index = host.app.visual.custom_themes.len() - 1;
            host.app.visual.theme = ThemeId::Custom;
            fx.toast("已新建自定义颜色");
        }
        Action::ThemeSavedUse(index) => {
            if index < host.app.visual.saved_presets.len() {
                host.app.visual.saved_index = index;
                host.app.visual.theme = ThemeId::Saved;
                host.rt.refresh_theme(&host.app);
            }
        }
        Action::ThemeSavedDelete(index) => {
            if index < host.app.visual.saved_presets.len() {
                host.app.visual.saved_presets.remove(index);
                if host.app.visual.saved_presets.is_empty() {
                    host.app.visual.saved_index = 0;
                    host.app.visual.theme = ThemeId::A;
                } else {
                    host.app.visual.saved_index = host.app.visual.saved_index.min(host.app.visual.saved_presets.len() - 1);
                }
                host.rt.refresh_theme(&host.app);
                fx.toast("已删除这个预设颜色");
            }
        }
        Action::ThemeSavePreset => {
            // 保存的是「当前生效的配色」，系统预设也能存成自己的预设
            let mut theme = host.rt.theme.clone();
            let base = if theme.name.trim().is_empty() { "预设".to_string() } else { theme.name.clone() };
            theme.name = format!("{} 副本", base);
            host.app.visual.saved_presets.push(theme);
            host.app.visual.saved_index = host.app.visual.saved_presets.len() - 1;
            host.app.visual.theme = ThemeId::Saved;
            host.rt.refresh_theme(&host.app);
            fx.toast("已保存当前颜色为预设");
        }
        Action::ThemeNameEdit(index) => {
            let text = host
                .app
                .visual
                .custom_themes
                .get(index)
                .map(|theme| theme.name.clone())
                .unwrap_or_default();
            begin_edit(host, EditState::new(EditTarget::ThemeName(index), text, true));
            host.rt.open_drop = None;
        }
        Action::ColorPick(target) => {
            let initial = current_color(&host.app, target);
            host.rt.pending = Some(PendingDialog::Color(target, initial));
        }
        Action::FontFamily(_) => {}
        Action::SliderFontSize | Action::SliderFontWeight => {
            let fraction = slider_fraction(host, rect);
            if action == Action::SliderFontSize {
                host.app.visual.pomo_font_size = (36.0 + fraction * 44.0).round().clamp(36.0, 80.0);
            } else {
                host.app.visual.pomo_font_weight = (((200.0 + fraction * 600.0) / 50.0).round() * 50.0).clamp(200.0, 800.0) as u32;
            }
        }
        Action::SliderWidgetScale => {
            let fraction = slider_fraction(host, rect);
            host.app.visual.widget_scale = (0.6 + fraction * 1.2).clamp(0.6, 1.8);
        }
        Action::SliderWidgetFontSize => {
            let fraction = slider_fraction(host, rect);
            host.app.visual.widget_font_size = (14.0 + fraction * 58.0).round().clamp(14.0, 72.0);
        }
        Action::SliderWidgetFontWeight => {
            let fraction = slider_fraction(host, rect);
            host.app.visual.widget_font_weight =
                ((((200.0 + fraction * 600.0) / 50.0).round() * 50.0).clamp(200.0, 800.0)) as u32;
        }
        Action::ToggleSelect(id) => {
            if let Some(pos) = host.rt.selected.iter().position(|value| *value == id) {
                host.rt.selected.remove(pos);
            } else {
                host.rt.selected.push(id);
            }
        }
        Action::SelectAll(checked) => {
            if checked {
                let ids: Vec<u64> = host.app.visible_history().iter().map(|item| item.id).collect();
                for id in ids {
                    if !host.rt.selected.contains(&id) {
                        host.rt.selected.push(id);
                    }
                }
            } else {
                host.rt.selected.clear();
            }
        }
        Action::BatchDelete => {
            if host.rt.selected.is_empty() {
                fx.toast("请先选择记录");
            } else {
                let ids = host.rt.selected.clone();
                host.app.delete_selected(&ids);
                host.rt.selected.clear();
                fx.toast("已删除选中记录");
            }
        }
        Action::FilterReset => {
            host.app.filter.group = None;
            host.app.filter.event = None;
            host.app.filter.date_start = None;
            host.app.filter.date_end = None;
            host.rt.selected.clear();
        }
        Action::CalendarToggle => {
            host.rt.calendar_open = !host.rt.calendar_open;
            if host.rt.calendar_open {
                host.rt.drop_anchor = rect;
                let (year, month) = clock::year_month();
                host.rt.calendar.year = year;
                host.rt.calendar.month = month;
                host.rt.calendar.start = host.app.filter.date_start.clone();
                host.rt.calendar.end = host.app.filter.date_end.clone();
            }
        }
        Action::CalendarPrev => {
            let calendar = &mut host.rt.calendar;
            if calendar.month == 1 {
                calendar.month = 12;
                calendar.year -= 1;
            } else {
                calendar.month -= 1;
            }
        }
        Action::CalendarNext => {
            let calendar = &mut host.rt.calendar;
            if calendar.month == 12 {
                calendar.month = 1;
                calendar.year += 1;
            } else {
                calendar.month += 1;
            }
        }
        Action::CalendarPickDay(day) => {
            let calendar = &mut host.rt.calendar;
            let date = clock::date_string(calendar.year, calendar.month, day);
            let start = calendar.start.clone();
            let end = calendar.end.clone();
            match (start, end) {
                (None, _) => {
                    calendar.start = Some(date);
                    calendar.end = None;
                }
                (Some(start), None) => {
                    if date < start {
                        calendar.start = Some(date);
                        calendar.end = Some(start);
                    } else {
                        calendar.end = Some(date);
                    }
                }
                (Some(_), Some(_)) => {
                    calendar.start = Some(date);
                    calendar.end = None;
                }
            }
        }
        Action::CalendarClear => {
            let calendar = &mut host.rt.calendar;
            calendar.start = None;
            calendar.end = None;
            host.app.filter.date_start = None;
            host.app.filter.date_end = None;
            host.rt.selected.clear();
            host.rt.calendar_open = false;
        }
        Action::CalendarApply => {
            let start = host.rt.calendar.start.clone();
            let end = host.rt.calendar.end.clone();
            host.app.filter.date_start = start;
            host.app.filter.date_end = if end.is_none() { host.rt.calendar.start.clone() } else { end };
            host.rt.selected.clear();
            host.rt.calendar_open = false;
        }
        Action::StopSound => {
            platform::sound::stop();
            host.rt.sound_stopped();
        }
        Action::PresetApply => {
            if let Some(draft) = host.rt.draft.take() {
                if draft.index < host.app.presets.len() {
                    host.app.active_preset = draft.index;
                    host.app.presets[draft.index] = draft.preset;
                    host.app.apply_preset();
                    fx.toast("已应用到番茄钟");
                }
            }
        }
        Action::ToggleBareUi => {
            host.app.visual.bare_ui = !host.app.visual.bare_ui;
        }
        Action::WindowSize(index) => {
            if index < WINDOW_SIZES.len() {
                host.app.visual.window_size = index;
                host.rt.window_dirty = true;
            }
        }
        Action::WidgetClose => {
            // 关闭桌面小图标要拆窗口，出借用再做
            host.rt.pending = Some(PendingDialog::Widget(false));
        }
        Action::WidgetModeToggle => {
            commit_edit(host, fx, now);
            // −：直接收起成「待机·未点击」；＋：回到运行模式
            host.rt.widget_mode = match host.rt.widget_mode {
                WidgetMode::Running => WidgetMode::Idle,
                _ => WidgetMode::Running,
            };
            host.rt.open_drop = None;
        }
        Action::WidgetRefresh => {
            // 重挂窗口要动窗口样式 / 定时器，出了借用再交给消息循环做
            commit_edit(host, fx, now);
            host.rt.widget_editing = false;
            host.rt.open_drop = None;
            host.rt.pending = Some(PendingDialog::WidgetRefresh);
        }
        Action::WidgetTopmost => {
            host.app.visual.widget_topmost = !host.app.visual.widget_topmost;
            // 窗口样式只能在借用之外改，交给消息循环
            host.rt.pending = Some(PendingDialog::WidgetStyle);
        }
        Action::WidgetEdit => {
            crate::ime_log("WidgetEdit：进入改名（内嵌 EDIT 控件接管输入）");
            let pomodoro = widget_view::module(&host.app, &host.rt) == ViewId::Pomodoro;
            let text = if pomodoro {
                host.app.pomodoro.activity.clone()
            } else {
                host.app.timer.activity.clone()
            };
            begin_edit(host, EditState::new(EditTarget::ActivityName(pomodoro), text, true));
            host.rt.widget_editing = true;
            host.rt.open_drop = None;
        }
        Action::WidgetOpen => {
            commit_edit(host, fx, now);
            host.rt.widget_editing = false;
            host.rt.view = widget_view::module(&host.app, &host.rt);
            host.rt.open_drop = None;
            host.rt.calendar_open = false;
            host.rt.draft = None;
            platform::window::show_and_focus(host.hwnd);
        }
        Action::WidgetVisible(on) => {
            // 建 / 拆悬浮窗要出借用再做，交给消息循环
            host.rt.pending = Some(PendingDialog::Widget(on));
        }
        Action::WidgetClickThrough => {
            host.app.visual.widget_click_through = !host.app.visual.widget_click_through;
            host.rt.pending = Some(PendingDialog::WidgetStyle);
        }
        Action::WidgetLock => {
            host.app.visual.widget_locked = !host.app.visual.widget_locked;
        }
        Action::WidgetModule(view) => {
            commit_edit(host, fx, now);
            host.rt.widget_module = view;
            host.rt.open_drop = None;
        }

    }
}

/// 拖动条位置换算成 0-1
fn slider_fraction(host: &Host, rect: Rect) -> f32 {
    ((host.ui.mouse.0 - rect.x) / rect.w.max(1.0)).clamp(0.0, 1.0)
}

/// 是否处于「计时方案草稿」作用域（番茄钟视图 + 草稿已建立）
fn draft_scope(host: &Host) -> bool {
    host.rt.view == ViewId::Pomodoro && host.rt.draft.is_some()
}

pub fn apply_color(host: &mut Host, target: ColorTarget, color: crate::color::Color) {
    match target {
        ColorTarget::Custom(index, field) => {
            if let Some(theme) = host.app.visual.custom_themes.get_mut(index) {
                theme.set_field(field, color.to_hex());
            }
        }
        ColorTarget::Saved(index, field) => {
            if let Some(theme) = host.app.visual.saved_presets.get_mut(index) {
                theme.set_field(field, color.to_hex());
            }
        }
    }
    host.rt.refresh_theme(&host.app);
    host.dirty = true;
}

pub fn apply_sound_path(host: &mut Host, path: String) {
    if draft_scope(host) {
        if let Some(draft) = host.rt.draft_preset_mut() {
            draft.end_sound = SoundKind::Custom;
        }
    } else {
        host.app.preset_mut().end_sound = SoundKind::Custom;
    }
    host.app.visual.sound_path = path;
    host.dirty = true;
}

fn store_save(host: &mut Host) {
    crate::store::save(&host.app);
    host.dirty = false;
}

fn current_color(app: &App, target: ColorTarget) -> Color {
    let text = match target {
        ColorTarget::Custom(index, _field) => app.visual.custom_themes.get(index),
        ColorTarget::Saved(index, field) => {
            let _ = field;
            app.visual.saved_presets.get(index)
        }
    };
    match text {
        Some(theme) => {
            let field = match target {
                ColorTarget::Custom(_, field) | ColorTarget::Saved(_, field) => field,
            };
            Color::from_hex(theme_field(theme, field))
        }
        None => Color::rgb(0xd0, 0x50, 0x40),
    }
}

pub fn theme_field(theme: &ThemeColors, field: usize) -> &str {
    match field {
        0 => &theme.work,
        1 => &theme.short_break,
        2 => &theme.long_break,
        3 => &theme.pause,
        4 => &theme.bg,
        5 => &theme.surface,
        6 => &theme.window,
        _ => &theme.text,
    }
}

fn preset_field(preset: &mut Preset, target: NumberTarget) -> &mut u32 {
    match target {
        NumberTarget::Focus => &mut preset.focus,
        NumberTarget::ShortRest => &mut preset.short_rest,
        NumberTarget::LongRest => &mut preset.long_rest,
        NumberTarget::Cycle => &mut preset.cycle,
    }
}

/// 时长最小 0 分：表示跳过当前阶段；轮数最小 1
pub fn preset_min(target: NumberTarget) -> u32 {
    if target == NumberTarget::Cycle {
        1
    } else {
        0
    }
}

pub fn step_preset(preset: &mut Preset, target: NumberTarget, delta: i32) {
    let min = preset_min(target) as i32;
    let field = preset_field(preset, target);
    *field = (*field as i32 + delta).clamp(min, 999) as u32;
}

pub fn preset_number(preset: &Preset, target: NumberTarget) -> u32 {
    match target {
        NumberTarget::Focus => preset.focus,
        NumberTarget::ShortRest => preset.short_rest,
        NumberTarget::LongRest => preset.long_rest,
        NumberTarget::Cycle => preset.cycle,
    }
}

fn step_number(app: &mut App, target: NumberTarget, delta: i32) {
    let preset = app.preset_mut();
    step_preset(preset, target, delta);
}

fn number_value(app: &App, target: NumberTarget) -> u32 {
    preset_number(app.preset(), target)
}

/// 开始一次编辑：先清掉上一处（可能来自悬浮窗）的编辑状态，
/// 不然「悬浮窗正在改名时点主窗口的输入框」会把输入框摆到悬浮窗那边去。
pub fn begin_edit(host: &mut Host, state: EditState) {
    host.rt.widget_editing = false;
    host.rt.edit = Some(state);
}

pub fn cancel_edit(host: &mut Host) {
    host.rt.edit = None;
    host.rt.widget_editing = false;
}

pub fn commit_edit(host: &mut Host, fx: &mut Effects, now: u64) {
    host.rt.widget_editing = false;
    let Some(edit) = host.rt.edit.take() else { return };
    let text = edit.text.trim().to_string();
    match edit.target {
        EditTarget::ActivityAdd => {
            if text.is_empty() {
                fx.toast("请输入事件名称");
            } else if host.app.activities.contains(&text) {
                fx.toast("该事件已存在");
            } else {
                host.app.activities.push(text.clone());
                fx.toast(format!("已添加「{}」", text));
            }
        }
        EditTarget::ActivityName(pomodoro) => {
            if !text.is_empty() {
                if !host.app.activities.contains(&text) {
                    host.app.activities.push(text.clone());
                    fx.toast(format!("已添加活动「{}」", text));
                }
                if pomodoro {
                    host.app.pomodoro.activity = text;
                } else {
                    host.app.timer.activity = text;
                }
            }
        }
        EditTarget::PresetName => {
            if !text.is_empty() {
                if let Some(draft) = host.rt.draft_preset_mut() {
                    draft.name = text;
                } else {
                    host.app.preset_mut().name = text;
                }
            }
        }
        EditTarget::ThemeName(index) => {
            if let Some(theme) = host.app.visual.custom_themes.get_mut(index) {
                theme.name = text;
            }
        }
        EditTarget::Number(target) => {
            if let Ok(value) = text.parse::<u32>() {
                let value = value.clamp(preset_min(target), 999);
                if let Some(draft) = host.rt.draft_preset_mut() {
                    *preset_field(draft, target) = value;
                } else {
                    *preset_field(host.app.preset_mut(), target) = value;
                    host.app.refresh_preset_if_idle(now);
                }
            }
        }
    }
}

pub fn click_away(host: &mut Host, fx: &mut Effects, now: u64) {
    if host.rt.open_drop.is_some() {
        host.rt.open_drop = None;
    } else if host.rt.edit.is_some() {
        commit_edit(host, fx, now);
    } else if host.rt.calendar_open {
        host.rt.calendar_open = false;
    }
}
