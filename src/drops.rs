use crate::action::DropId;
use crate::model::{App, Group, SoundKind, Stage};
use crate::runtime::Runtime;

pub const FONT_FAMILIES: [(&str, &str); 5] = [
    ("系统默认", "Microsoft YaHei UI"),
    ("微软雅黑", "Microsoft YaHei"),
    ("苹方", "PingFang SC"),
    ("Arial", "Arial"),
    ("Georgia", "Georgia"),
];

pub fn items(app: &App, id: DropId) -> Vec<String> {
    match id {
        DropId::Preset => app.presets.iter().map(|preset| preset.name.clone()).collect(),
        DropId::FocusEvent | DropId::ShortRestEvent | DropId::LongRestEvent => app.activities.clone(),
        DropId::SoundKind => SoundKind::ALL.iter().map(|kind| kind.label().to_string()).collect(),
        DropId::FontFamily => FONT_FAMILIES.iter().map(|(label, _)| label.to_string()).collect(),
        DropId::TimerActivity | DropId::PomoActivity => app.activities.clone(),
        DropId::FilterGroup => vec!["全部类型".to_string(), "专注".to_string(), "休息".to_string()],
        DropId::FilterEvent => {
            let mut list = vec!["全部事件".to_string()];
            list.extend(app.activities.iter().cloned());
            list
        }
    }
}

pub fn selected(app: &App, id: DropId) -> usize {
    match id {
        DropId::Preset => app.active_preset,
        DropId::FocusEvent => index_of(&app.activities, &app.preset().focus_event),
        DropId::ShortRestEvent => index_of(&app.activities, &app.preset().short_rest_event),
        DropId::LongRestEvent => index_of(&app.activities, &app.preset().long_rest_event),
        DropId::SoundKind => SoundKind::ALL.iter().position(|kind| *kind == app.preset().end_sound).unwrap_or(0),
        DropId::FontFamily => FONT_FAMILIES
            .iter()
            .position(|(_, value)| *value == app.visual.pomo_font_family)
            .unwrap_or(0),
        DropId::TimerActivity => index_of(&app.activities, &app.timer.activity),
        DropId::PomoActivity => index_of(&app.activities, &app.pomodoro.activity),
        DropId::FilterGroup => match app.filter.group {
            None => 0,
            Some(Group::Focus) => 1,
            Some(Group::Rest) => 2,
        },
        DropId::FilterEvent => match &app.filter.event {
            None => 0,
            Some(name) => index_of(&app.activities, name) + 1,
        },
    }
}

fn index_of(list: &[String], value: &str) -> usize {
    list.iter().position(|item| item == value).unwrap_or(0)
}

pub fn current_label(app: &App, id: DropId) -> String {
    let list = items(app, id);
    let index = selected(app, id);
    list.get(index).cloned().unwrap_or_default()
}

/// 下拉框当前高亮项：番茄钟视图的「计时方案」在草稿模式下用草稿的序号
pub fn selected_with(app: &App, rt: &Runtime, id: DropId) -> usize {
    if id == DropId::Preset {
        if let Some(draft) = &rt.draft {
            return draft.index;
        }
    }
    selected(app, id)
}

/// 草稿模式下把「方案 / 阶段事件 / 提示音」的选择写进草稿，不影响真正在跑的番茄钟
fn pick_draft(app: &App, rt: &mut Runtime, id: DropId, index: usize) -> bool {
    match id {
        DropId::Preset => {
            if let Some(preset) = app.presets.get(index).cloned() {
                if let Some(draft) = rt.draft.as_mut() {
                    draft.index = index;
                    draft.preset = preset;
                }
            }
            true
        }
        DropId::FocusEvent | DropId::ShortRestEvent | DropId::LongRestEvent => {
            if let Some(name) = app.activities.get(index).cloned() {
                if let Some(draft) = rt.draft.as_mut() {
                    match id {
                        DropId::FocusEvent => draft.preset.focus_event = name,
                        DropId::ShortRestEvent => draft.preset.short_rest_event = name,
                        _ => draft.preset.long_rest_event = name,
                    }
                }
            }
            true
        }
        DropId::SoundKind => {
            if let Some(kind) = SoundKind::ALL.get(index) {
                if let Some(draft) = rt.draft.as_mut() {
                    draft.preset.end_sound = *kind;
                }
            }
            true
        }
        _ => false,
    }
}

pub fn pick(app: &mut App, rt: &mut Runtime, id: DropId, index: usize) {
    if rt.draft.is_some() && pick_draft(app, rt, id, index) {
        return;
    }
    match id {
        DropId::Preset => {
            app.select_preset(index);
            app.apply_preset();
        }
        DropId::FocusEvent | DropId::ShortRestEvent | DropId::LongRestEvent => {
            let Some(name) = app.activities.get(index).cloned() else { return };
            match id {
                DropId::FocusEvent => app.preset_mut().focus_event = name.clone(),
                DropId::ShortRestEvent => app.preset_mut().short_rest_event = name.clone(),
                _ => app.preset_mut().long_rest_event = name.clone(),
            }
            let stage = app.pomodoro.stage;
            let affects_stage = matches!(
                (id, stage),
                (DropId::FocusEvent, Stage::Focus)
                    | (DropId::ShortRestEvent, Stage::ShortRest)
                    | (DropId::LongRestEvent, Stage::LongRest)
            );
            if affects_stage {
                app.pomodoro.activity = name;
            }
            app.refresh_preset_if_idle(crate::clock::now_ms());
        }
        DropId::SoundKind => {
            if let Some(kind) = SoundKind::ALL.get(index) {
                app.preset_mut().end_sound = *kind;
            }
        }
        DropId::FontFamily => {
            if let Some((_, value)) = FONT_FAMILIES.get(index) {
                app.visual.pomo_font_family = (*value).to_string();
            }
        }
        DropId::TimerActivity | DropId::PomoActivity => {
            if let Some(name) = app.activities.get(index).cloned() {
                if id == DropId::TimerActivity {
                    app.timer.activity = name;
                } else {
                    app.pomodoro.activity = name;
                }
            }
        }
        DropId::FilterGroup => {
            app.filter.group = match index {
                1 => Some(Group::Focus),
                2 => Some(Group::Rest),
                _ => None,
            };
            rt.selected.clear();
        }
        DropId::FilterEvent => {
            app.filter.event = if index == 0 {
                None
            } else {
                app.activities.get(index - 1).cloned()
            };
            rt.selected.clear();
        }
    }
}
