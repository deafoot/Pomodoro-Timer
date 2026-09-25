use crate::action::{Action, NumberTarget, ViewId};
use crate::color::{Color, Palette, ThemeColors};
use crate::ui::icons;
use crate::ui::{Align, Font, Rect, Ui};

pub fn nav_bar(ui: &mut Ui, palette: &Palette, rect: Rect, current: ViewId) {
    ui.fill(rect, palette.inset, 0.0);
    let tab_w = rect.w / 4.0;
    for (index, view) in ViewId::ALL.iter().enumerate() {
        let tab = Rect::new(rect.x + tab_w * index as f32, rect.y, tab_w, rect.h);
        let active = *view == current;
        let action = Action::Nav(*view);
        ui.hit(tab, action.clone());
        let hover = ui.is_hover(action);
        if active {
            ui.fill(Rect::new(tab.x, tab.bottom() - 2.0, tab.w, 2.0), palette.work, 0.0);
        }
        let color = if active {
            palette.text
        } else if hover {
            Color::mix(palette.muted, palette.text, 0.4)
        } else {
            palette.muted
        };
        let icon = Font::new(16.0, 500).center().essential();
        ui.text(view.icon(), Rect::new(tab.x, tab.y + 6.0, tab.w, 22.0), color, &icon);
        let label = Font::new(13.0, if active { 600 } else { 400 }).center();
        ui.text(view.label(), Rect::new(tab.x, tab.y + 28.0, tab.w, 20.0), color, &label);
    }
    ui.fill(Rect::new(rect.x, rect.bottom() - 1.0, rect.w, 1.0), palette.border, 0.0);
}

pub struct TabItem {
    pub label: String,
    pub icon: String,
    pub active: bool,
    pub color: Color,
    pub action: Action,
}

pub fn mode_tabs(ui: &mut Ui, palette: &Palette, rect: Rect, items: &[TabItem], tabs: usize) {
    let tab_w = rect.w / tabs.max(1) as f32;
    for (index, item) in items.iter().enumerate() {
        let tab = Rect::new(rect.x + tab_w * index as f32, rect.y, tab_w, rect.h);
        ui.hit(tab.inset(0.0, 4.0), item.action.clone());
        let hover = ui.is_hover(item.action.clone());
        let color = if item.active {
            item.color
        } else if hover {
            Color::mix(palette.muted, item.color, 0.4)
        } else {
            palette.muted
        };
        let show_label = !ui.hide_all_text;
        let text = if show_label {
            format!("{} {}", item.label, item.icon)
        } else {
            item.icon.clone()
        };
        let font = Font::new(if show_label { 14.0 } else { 17.0 }, 600).center().essential();
        ui.text(&text, Rect::new(tab.x, tab.y, tab.w, tab.h), color, &font);
        if item.active {
            ui.fill(Rect::new(tab.x + 8.0, tab.bottom() - 2.0, tab.w - 16.0, 2.0), item.color, 1.0);
        }
    }
}

pub fn primary_button(ui: &mut Ui, rect: Rect, label: &str, icon: &str, color: Color, enabled: bool, action: Action) {
    let hover = enabled && ui.is_hover(action.clone());
    if enabled {
        ui.hit(rect, action);
    }
    let bg = if !enabled {
        Color::mix(color, Color::rgb(140, 140, 145), 0.55)
    } else if hover {
        color.lighten(0.08)
    } else {
        color
    };
    ui.fill(rect, bg, 10.0);
    let show_label = !ui.hide_all_text;
    let text = if show_label { format!("{} {}", icon, label) } else { icon.to_string() };
    let font = Font::new(15.0, 600).center().essential();
    ui.text(&text, rect, Color::WHITE, &font);
}

pub fn ghost_button(ui: &mut Ui, palette: &Palette, rect: Rect, label: &str, icon: &str, enabled: bool, action: Action) {
    let hover = enabled && ui.is_hover(action.clone());
    if enabled {
        ui.hit(rect, action);
    }
    let bg = if !enabled {
        palette.soft.with_alpha(0.5)
    } else if hover {
        palette.soft.lighten(0.03)
    } else {
        palette.soft
    };
    ui.fill(rect, bg, 10.0);
    let color = if enabled { palette.text } else { palette.faint };
    let show_label = !ui.hide_all_text;
    let text = if show_label { format!("{} {}", icon, label) } else { icon.to_string() };
    let font = Font::new(14.0, 500).center().essential();
    ui.text(&text, rect, color, &font);
}

pub fn icon_button(ui: &mut Ui, palette: &Palette, rect: Rect, glyph: &str, action: Action, accent: Option<Color>) {
    let hover = ui.is_hover(action.clone());
    ui.hit(rect, action);
    let base = accent.unwrap_or(palette.text);
    let color = if hover { base.lighten(0.15) } else { base };
    ui.fill(rect, if hover { palette.soft } else { palette.surface }, 8.0);
    let font = Font::new(14.0, 500).center().essential();
    ui.text(glyph, rect, color, &font);
}

pub fn mini_button(ui: &mut Ui, palette: &Palette, rect: Rect, label: &str, action: Action, color: Color, filled: bool) {
    let hover = ui.is_hover(action.clone());
    ui.hit(rect, action);
    if filled {
        ui.fill(rect, if hover { color.lighten(0.1) } else { color }, 6.0);
        let font = Font::new(12.5, 600).center().essential();
        ui.text(label, rect, Color::WHITE, &font);
    } else {
        // 未选中的按钮用卡片底色 + 主题文字色，浅色 / 深色主题下都看得清
        ui.fill(rect, if hover { palette.soft } else { palette.surface }, 6.0);
        ui.stroke(rect, palette.border, 6.0, 1.0);
        let font = Font::new(12.5, 600).center().essential();
        ui.text(label, rect, color, &font);
    }
}

pub fn border_button(ui: &mut Ui, palette: &Palette, rect: Rect, label: &str, action: Action, color: Color) {
    let hover = ui.is_hover(action.clone());
    ui.hit(rect, action);
    ui.fill(rect, if hover { palette.soft } else { palette.surface }, 6.0);
    ui.stroke(rect, if hover { color } else { palette.border }, 6.0, 1.0);
    let font = Font::new(12.5, 500).center();
    ui.text(label, rect, color, &font);
}

pub fn checkbox(ui: &mut Ui, palette: &Palette, rect: Rect, checked: bool, action: Action) {
    ui.hit(rect, action);
    ui.fill(rect, if checked { palette.work } else { palette.surface }, 5.0);
    ui.stroke(rect, if checked { palette.work } else { palette.border }, 5.0, 1.0);
    if checked {
        let font = Font::new(11.0, 700).center().essential();
        ui.text(icons::CHECK, rect, Color::WHITE, &font);
    }
}

pub fn switch(ui: &mut Ui, palette: &Palette, rect: Rect, checked: bool, action: Action) {
    ui.hit(rect, action);
    let radius = rect.h / 2.0;
    let track = if checked { palette.work } else { Color::mix(palette.border, palette.surface, 0.3) };
    ui.fill(rect, track, radius);
    let knob_r = radius - 2.5;
    let knob_x = if checked { rect.right() - radius } else { rect.x + radius };
    ui.circle(knob_x, rect.mid_y(), knob_r, Color::WHITE);
}

pub fn switch_row(ui: &mut Ui, palette: &Palette, rect: Rect, label: &str, checked: bool, action: Action) {
    let switch_rect = Rect::new(rect.right() - 40.0, rect.mid_y() - 11.0, 40.0, 22.0);
    switch(ui, palette, switch_rect, checked, action);
    let font = Font::new(13.0, 400);
    ui.text(label, Rect::new(rect.x, rect.y, rect.w - 50.0, rect.h), palette.text, &font);
}

pub fn select_box(ui: &mut Ui, palette: &Palette, rect: Rect, text: &str, enabled: bool, action: Action) {
    select_box_align(ui, palette, rect, text, enabled, action, Align::Left);
}

/// 居中的下拉框（番茄钟 / 计时界面的活动显示）
pub fn select_box_centered(ui: &mut Ui, palette: &Palette, rect: Rect, text: &str, enabled: bool, action: Action) {
    select_box_align(ui, palette, rect, text, enabled, action, Align::Center);
}

fn select_box_align(ui: &mut Ui, palette: &Palette, rect: Rect, text: &str, enabled: bool, action: Action, align: Align) {
    let hover = enabled && ui.is_hover(action.clone());
    if enabled {
        ui.hit(rect, action);
    }
    ui.fill(rect, if hover { palette.soft } else { palette.surface }, 8.0);
    ui.stroke(rect, palette.border, 8.0, 1.0);
    let font = Font::new(13.0, 500).align(align);
    let text_rect = match align {
        Align::Center => rect,
        _ => Rect::new(rect.x + 10.0, rect.y, rect.w - 28.0, rect.h),
    };
    ui.text(text, text_rect, palette.text, &font);
    let arrow = Font::new(10.0, 500).center().essential();
    ui.text(icons::CARET_DOWN, Rect::new(rect.right() - 22.0, rect.y, 16.0, rect.h), palette.muted, &arrow);
}


pub fn slider(ui: &mut Ui, palette: &Palette, rect: Rect, fraction: f32, action: Action) {
    let track = Rect::new(rect.x, rect.mid_y() - 3.0, rect.w, 6.0);
    ui.fill(track, palette.border, 3.0);
    ui.fill(Rect::new(track.x, track.y, track.w * fraction.clamp(0.0, 1.0), track.h), palette.work, 3.0);
    let knob_x = track.x + track.w * fraction.clamp(0.0, 1.0);
    ui.circle(knob_x, track.mid_y(), 8.0, Color::WHITE);
    ui.circle(knob_x, track.mid_y(), 8.0, palette.work.with_alpha(0.0));
    ui.stroke(Rect::new(knob_x - 8.0, track.mid_y() - 8.0, 16.0, 16.0), palette.work, 8.0, 1.5);
    ui.hit(rect, action);
}

pub fn tag(ui: &mut Ui, palette: &Palette, rect: Rect, label: &str, close: Option<Action>) {
    ui.fill(rect, palette.soft, 12.0);
    let font = Font::new(12.5, 400);
    let text_w = ui.text_width(label, &font);
    let has_close = close.is_some();
    let close_w = if has_close { 18.0 } else { 0.0 };
    let total = text_w + close_w;
    let start = rect.x + (rect.w - total) / 2.0;
    ui.text(label, Rect::new(start, rect.y, text_w + 2.0, rect.h), palette.text, &font);
    if let Some(action) = close {
        let close_rect = Rect::new(start + text_w + 2.0, rect.y, close_w, rect.h);
        let hover = ui.is_hover(action.clone());
        ui.hit(close_rect, action);
        let glyph = Font::new(11.0, 600).center().essential();
        ui.text("✕", close_rect, if hover { palette.work } else { palette.muted }, &glyph);
    }
}

pub fn section_header(ui: &mut Ui, palette: &Palette, rect: Rect, icon: &str, title: &str, open: bool, action: Action) {
    ui.hit(rect, action.clone());
    let hover = ui.is_hover(action);
    if hover {
        ui.fill(rect.inset(-6.0, 0.0), palette.soft, 8.0);
    }
    let icon_font = Font::new(16.0, 500).essential();
    ui.text(icon, Rect::new(rect.x, rect.y, 24.0, rect.h), palette.text, &icon_font);
    let font = Font::new(16.0, 700);
    ui.text(title, Rect::new(rect.x + 24.0, rect.y, rect.w - 60.0, rect.h), palette.text, &font);
    let arrow = Font::new(11.0, 500).right().essential();
    ui.text(if open { icons::CARET_UP } else { icons::CARET_DOWN }, rect, palette.muted, &arrow);
}

/// 输入框外壳。光标、组字串、选区都由盖在上面的系统 EDIT 控件负责，
/// 这里画的文字只是控件还没摆好时的过渡（所以不再画光标和候选串）。
pub fn text_field(ui: &mut Ui, palette: &Palette, rect: Rect, text: &str, placeholder: &str) {
    ui.fill(rect, palette.surface, 8.0);
    ui.stroke(rect, palette.border, 8.0, 1.0);
    let font = Font::new(13.0, 400);
    let inner = Rect::new(rect.x + 10.0, rect.y, rect.w - 20.0, rect.h);
    if text.is_empty() {
        ui.text(placeholder, inner, palette.faint, &font);
    } else {
        ui.text(text, inner, palette.text, &font);
    }
}

pub fn theme_card(ui: &mut Ui, rect: Rect, title: &str, theme: &ThemeColors, active: bool, action: Action) {
    let hover = ui.is_hover(action.clone());
    ui.hit(rect, action);
    ui.fill(rect, Color::rgb(255, 255, 255).with_alpha(if hover { 0.95 } else { 0.75 }), 10.0);
    ui.stroke(rect, if active { Color::from_hex(&theme.work) } else { Color::rgb(224, 224, 224) }, 10.0, if active { 2.0 } else { 1.0 });
    let font = Font::new(12.0, 600);
    ui.text(title, Rect::new(rect.x + 10.0, rect.y + 6.0, rect.w - 20.0, 18.0), Color::rgb(40, 38, 36), &font);
    let swatches = [
        theme.work.as_str(),
        theme.short_break.as_str(),
        theme.long_break.as_str(),
        theme.pause.as_str(),
        theme.bg.as_str(),
        theme.surface.as_str(),
        theme.window.as_str(),
    ];
    let sw = Rect::new(rect.x + 10.0, rect.y + 28.0, rect.w - 20.0, 16.0);
    let cell = sw.w / 7.0;
    for (index, value) in swatches.iter().enumerate() {
        let r = Rect::new(sw.x + cell * index as f32, sw.y, cell - 1.0, sw.h);
        ui.fill(r, Color::from_hex(value), 2.0);
    }
}

pub fn swatch(ui: &mut Ui, rect: Rect, color: Color, active: bool) -> bool {
    let hover = ui.hover_rect(rect);
    ui.fill(rect, color, 6.0);
    ui.stroke(rect, if active { Color::rgb(60, 60, 60) } else { Color::rgb(200, 200, 200) }, 6.0, if active { 2.0 } else { 1.0 });
    hover
}

pub fn scrollbar(ui: &mut Ui, palette: &Palette, track: Rect, offset: f32, content_h: f32, view_h: f32) {
    if content_h <= view_h + 1.0 {
        return;
    }
    let thumb_h = (track.h * (view_h / content_h)).max(28.0);
    let max_scroll = (content_h - view_h).max(1.0);
    let t = (offset / max_scroll).clamp(0.0, 1.0);
    let y = track.y + (track.h - thumb_h) * t;
    ui.fill(track, palette.soft.with_alpha(0.6), track.w / 2.0);
    ui.fill(Rect::new(track.x, y, track.w, thumb_h), palette.border, track.w / 2.0);
}

pub fn stat_card(ui: &mut Ui, palette: &Palette, rect: Rect, label: &str, value: &str, value_color: Color) {
    ui.fill(rect, palette.inset, 10.0);
    let value_font = Font::new(18.0, 700).center().essential();
    if ui.hide_all_text {
        // 隐藏界面文字时只留数值，整块居中
        ui.text(value, rect, value_color, &value_font);
        return;
    }
    let label_h = 18.0;
    let value_h = 30.0;
    let total = label_h + value_h + 4.0;
    let top = rect.y + ((rect.h - total) / 2.0).max(4.0);
    let label_font = Font::new(12.0, 400).center();
    ui.text(label, Rect::new(rect.x + 6.0, top, rect.w - 12.0, label_h), palette.faint, &label_font);
    ui.text(value, Rect::new(rect.x + 6.0, top + label_h + 4.0, rect.w - 12.0, value_h), value_color, &value_font);
}

/// 数字输入：左侧数字区可点击后用键盘输入，右侧 +/− 步进

/// 「结束提示音」按钮：出现在数字右侧，没有提示音时调用方不绘制
pub fn sound_stop_button(ui: &mut Ui, palette: &Palette, area: Rect) -> Rect {
    let label = "结束提示音";
    let font = Font::new(12.5, 600).center();
    let show_label = !ui.hide_all_text;
    let w = if show_label { ui.text_width(label, &font) + 50.0 } else { 40.0 };
    let h = 34.0;
    let rect = Rect::new(area.right() - w, area.mid_y() - h / 2.0, w, h);
    let action = Action::StopSound;
    let hover = ui.is_hover(action.clone());
    ui.hit(rect, action);
    ui.fill(rect, if hover { palette.work } else { palette.surface }, h / 2.0);
    ui.stroke(rect, palette.work, h / 2.0, 1.0);
    let color = if hover { Color::WHITE } else { palette.work };
    let icon_font = Font::new(13.0, 600).center().essential();
    ui.text(icons::SOUND, Rect::new(rect.x + 10.0, rect.y, 18.0, rect.h), color, &icon_font);
    if show_label {
        ui.text(label, Rect::new(rect.x + 30.0, rect.y, rect.w - 38.0, rect.h), color, &font);
    }
    rect
}

/// 数字输入：左侧数字区可点击后用键盘输入，右侧 +/− 步进
pub fn number_field(ui: &mut Ui, palette: &Palette, rect: Rect, value: u32, target: NumberTarget, enabled: bool) {
    ui.fill(rect, palette.surface, 8.0);
    ui.stroke(rect, palette.border, 8.0, 1.0);
    let font = Font::new(13.0, 500).center();
    ui.text(&value.to_string(), Rect::new(rect.x, rect.y, rect.w - 44.0, rect.h), palette.text, &font);
    let btn = 22.0;
    let minus = Rect::new(rect.right() - 44.0, rect.mid_y() - btn / 2.0, btn, btn);
    let plus = Rect::new(rect.right() - 44.0 + btn, rect.mid_y() - btn / 2.0, btn, btn);
    let glyph = Font::new(14.0, 600).center().essential();
    let text_color = if enabled { palette.text } else { palette.faint };
    for (r, symbol, action) in [(minus, "−", Action::NumberStep(target, -1)), (plus, "+", Action::NumberStep(target, 1))] {
        let hover = enabled && ui.is_hover(action.clone());
        ui.hit(r, action);
        ui.fill(r, if hover { palette.soft } else { palette.surface }, 6.0);
        ui.text(symbol, r, text_color, &glyph);
    }
    if enabled {
        ui.hit(Rect::new(rect.x, rect.y, rect.w - 44.0, rect.h), Action::NumberEdit(target));
    }
}

pub fn progress_bar(ui: &mut Ui, palette: &Palette, rect: Rect, fraction: f32, color: Color) {
    let radius = rect.h / 2.0;
    ui.fill(rect, palette.inset, radius);
    let w = rect.w * fraction.clamp(0.0, 1.0);
    if w > 1.0 {
        ui.fill(Rect::new(rect.x, rect.y, w, rect.h), color, radius);
    }
}

/// 按可用宽度截断文字并加省略号（DirectWrite 不换行会直接溢出，这里手工收一下）
pub fn ellipsize(ui: &mut Ui, text: &str, width: f32, font: &Font) -> String {
    if width <= 4.0 || ui.text_width(text, font) <= width {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let mut probe = out.clone();
        probe.push(ch);
        probe.push('…');
        if ui.text_width(&probe, font) > width {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}

