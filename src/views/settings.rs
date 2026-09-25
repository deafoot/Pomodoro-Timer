use crate::action::{Action, ColorTarget, DropId, EditTarget, Section};
use crate::color::{Color, Palette, ThemeColors};
use crate::drops;
use crate::model::{ThemeId, VisualMode, WINDOW_SIZES};
use crate::ui::icons;
use crate::ui::widgets;
use crate::ui::{grid, Font, Rect, Ui};

use super::{preset_card, Ctx};

const FIELD_NAMES: [&str; 7] = ["专注", "短休", "长休", "暂停", "背景", "卡片", "窗口"];

pub fn draw(ui: &mut Ui, ctx: &mut Ctx) -> f32 {
    let area = ctx.area;
    let mut y = area.y + 12.0;

    // 计时方案
    y = section_header(ui, ctx, y, icons::PLAN, "计时方案", Section::SettingsPreset);
    if ctx.open(Section::SettingsPreset) {
        let used = preset_card::draw(ui, ctx, Rect::new(area.x, y, area.w, 480.0), false);
        y += used + 14.0;
    }

    // 事件管理
    y = section_header(ui, ctx, y, icons::TAG, "事件管理", Section::SettingsEvent);
    if ctx.open(Section::SettingsEvent) {
        y = activity_tags(ui, ctx, y);
        y += 14.0;
    }

    // 视觉设置
    y = section_header(ui, ctx, y, icons::BRUSH, "视觉设置", Section::SettingsVisual);
    if ctx.open(Section::SettingsVisual) {
        y = sub_header(ui, ctx, y, "番茄钟设置", Section::VisualPomodoro);
        if ctx.open(Section::VisualPomodoro) {
            y = visual_pomodoro(ui, ctx, y);
        }
        y = sub_header(ui, ctx, y, "颜色设置", Section::VisualColor);
        if ctx.open(Section::VisualColor) {
            y = visual_color(ui, ctx, y);
        }
        y = sub_header(ui, ctx, y, "文字设置", Section::VisualText);
        if ctx.open(Section::VisualText) {
            y = visual_text(ui, ctx, y);
        }
        y = sub_header(ui, ctx, y, "界面设置", Section::VisualUi);
        if ctx.open(Section::VisualUi) {
            y = visual_ui(ui, ctx, y);
        }
        y += 6.0;
    }

    // 桌面小图标（独立设置项）
    y = section_header(ui, ctx, y, icons::WIDGET, "桌面小图标", Section::SettingsWidget);
    if ctx.open(Section::SettingsWidget) {
        y = widget_section(ui, ctx, y);
        y += 6.0;
    }

    y - area.y
}

fn section_header(ui: &mut Ui, ctx: &Ctx, y: f32, icon: &str, title: &str, section: Section) -> f32 {
    let rect = Rect::new(ctx.area.x, y, ctx.area.w, 34.0);
    widgets::section_header(ui, &ctx.palette, rect, icon, title, ctx.open(section), Action::ToggleSection(section));
    rect.bottom() + 10.0
}

fn sub_header(ui: &mut Ui, ctx: &Ctx, y: f32, title: &str, section: Section) -> f32 {
    let palette = ctx.palette;
    let rect = Rect::new(ctx.area.x, y, ctx.area.w, 30.0);
    let action = Action::ToggleSection(section);
    ui.hit(rect, action.clone());
    let hover = ui.is_hover(action);
    ui.fill(rect, if hover { palette.soft } else { palette.inset }, 8.0);
    ui.text(title, Rect::new(rect.x + 12.0, rect.y, rect.w - 60.0, rect.h), palette.text, &Font::new(14.5, 700));
    let arrow = Font::new(10.0, 500).right().essential();
    ui.text(if ctx.open(section) { "▲" } else { "▼" }, Rect::new(rect.x, rect.y, rect.w - 12.0, rect.h), palette.muted, &arrow);
    rect.bottom() + 8.0
}

fn activity_tags(ui: &mut Ui, ctx: &mut Ctx, y: f32) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let activities = ctx.app.activities.clone();
    let editing = matches!(&ctx.rt.edit, Some(edit) if edit.target == EditTarget::ActivityAdd);
    let font = Font::new(12.5, 400);
    let tag_h = 28.0;
    let mut x = area.x;
    let mut row_y = y;
    for (index, name) in activities.iter().enumerate() {
        let width = ui.text_width(name, &font) + 34.0;
        if x > area.x && x + width > area.right() {
            x = area.x;
            row_y += tag_h + 6.0;
        }
        widgets::tag(ui, &palette, Rect::new(x, row_y, width, tag_h), name, Some(Action::ActivityDelete(index)));
        x += width + 6.0;
    }

    if editing {
        let field_w = (area.w - 130.0).max(120.0);
        let field = Rect::new(area.x, row_y + tag_h + 6.0, field_w, tag_h);
        let edit = ctx.rt.edit.as_ref().unwrap();
        let text = edit.text.clone();
        widgets::text_field(ui, &palette, field, &text, "输入事件名称");
        ctx.rt.edit_rect = field;
        widgets::mini_button(ui, &palette,
            Rect::new(field.right() + 6.0, field.y + 2.0, 50.0, tag_h - 4.0),
            "添加",
            Action::ActivityAddConfirm,
            palette.work,
            true,
        );
        widgets::mini_button(ui, &palette,
            Rect::new(field.right() + 62.0, field.y + 2.0, 50.0, tag_h - 4.0),
            "取消",
            Action::ActivityAddCancel,
            palette.muted,
            false,
        );
        row_y += tag_h + 6.0;
    } else {
        let label = "＋ 添加事件";
        let width = ui.text_width(label, &font) + 24.0;
        if x > area.x && x + width > area.right() {
            x = area.x;
            row_y += tag_h + 6.0;
        }
        widgets::border_button(ui, &palette, Rect::new(x, row_y, width, tag_h), label, Action::ActivityAddStart, palette.work);
    }

    row_y + tag_h
}

fn visual_pomodoro(ui: &mut Ui, ctx: &mut Ctx, y: f32) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let mode = ctx.app.visual.mode;
    let hide_number = ctx.app.visual.hide_number;

    let cards = grid(Rect::new(area.x, y, area.w, 40.0), 3, 8.0);
    for (index, candidate) in VisualMode::ALL.iter().enumerate() {
        let active = mode == *candidate;
        let rect = cards[index];
        let action = Action::VisualMode(*candidate);
        let hover = ui.is_hover(action.clone());
        ui.hit(rect, action);
        ui.fill(
            rect,
            if active {
                palette.work.with_alpha(0.12)
            } else if hover {
                palette.soft
            } else {
                palette.surface
            },
            8.0,
        );
        ui.stroke(rect, if active { palette.work } else { palette.border }, 8.0, if active { 2.0 } else { 1.0 });
        ui.text(
            candidate.label(),
            rect,
            if active { palette.work } else { palette.text },
            &Font::new(13.0, if active { 600 } else { 400 }).center(),
        );
    }
    let mut y = cards[0].bottom() + 8.0;

    let row = Rect::new(area.x, y, area.w, 28.0);
    widgets::switch_row(ui, &palette, row, "隐藏文字（仅图形模式生效）", hide_number, Action::ToggleHideNumber);
    y = row.bottom() + 10.0;
    y
}

fn visual_color(ui: &mut Ui, ctx: &mut Ctx, y: f32) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let app = ctx.app;
    let dark = ctx.rt.dark;
    let mut y = y;

    // 系统预设
    subtitle(ui, &palette, area, y, "系统预设");
    y += 22.0;
    let cell_h = 62.0;
    let rows = (ThemeId::BUILTIN.len() + 2) / 3;
    for row in 0..rows {
        let cards = grid(Rect::new(area.x, y + (cell_h + 8.0) * row as f32, area.w, cell_h), 3, 8.0);
        for col in 0..3 {
            let Some(id) = ThemeId::BUILTIN.get(row * 3 + col).copied() else { continue };
            let theme = id.colors(dark);
            widgets::theme_card(ui, cards[col], &theme.name, &theme, app.visual.theme == id, Action::ThemeBuiltin(id));
        }
    }
    y += (cell_h + 8.0) * rows as f32 + 6.0;

    // 我的预设
    subtitle(ui, &palette, area, y, "我的预设");
    y += 22.0;
    if app.visual.saved_presets.is_empty() {
        ui.text("还没有保存的预设颜色", Rect::new(area.x, y, area.w, 20.0), palette.faint, &Font::new(12.0, 400));
        y += 28.0;
    } else {
        for (index, theme) in app.visual.saved_presets.iter().enumerate() {
            let active = app.visual.theme == ThemeId::Saved && app.visual.saved_index == index;
            let title = if theme.name.is_empty() { "已保存预设".to_string() } else { theme.name.clone() };
            // 和系统预设一样的色卡样式，点整张卡即应用
            let head = Rect::new(area.x, y, area.w, 62.0);
            widgets::theme_card(ui, head, &title, theme, active, Action::ThemeSavedUse(index));
            let close = Rect::new(head.right() - 26.0, head.y + 3.0, 20.0, 20.0);
            let close_hover = ui.is_hover(Action::ThemeSavedDelete(index));
            ui.hit(close, Action::ThemeSavedDelete(index));
            ui.text("✕", close, if close_hover { palette.work } else { palette.muted }, &Font::new(11.0, 600).center());
            picker_row(
                ui,
                &palette,
                Rect::new(area.x + 8.0, head.bottom() + 6.0, area.w - 16.0, 36.0),
                theme,
                |field| ColorTarget::Saved(index, field),
            );
            y = head.bottom() + 50.0;
        }
    }

    // 自定义颜色
    let header = Rect::new(area.x, y, area.w, 22.0);
    ui.text("自定义颜色", header, palette.faint, &Font::new(12.0, 500));
    let new_rect = Rect::new(header.right() - 74.0, header.y, 74.0, 22.0);
    widgets::mini_button(ui, &palette, new_rect, "＋ 新建", Action::ThemeCustomNew, palette.work, true);
    y = header.bottom() + 6.0;

    if app.visual.custom_themes.is_empty() {
        ui.text(
            "点击“＋ 新建”创建自己的颜色方案",
            Rect::new(area.x, y, area.w, 20.0),
            palette.faint,
            &Font::new(12.0, 400),
        );
        y += 28.0;
    } else {
        for (index, theme) in app.visual.custom_themes.iter().enumerate() {
            let active = app.visual.theme == ThemeId::Custom && app.visual.custom_index == index;
            let box_rect = Rect::new(area.x, y, area.w, 158.0);
            ui.fill(box_rect, palette.surface, 10.0);
            ui.stroke(
                box_rect,
                if active { Color::from_hex(&theme.work) } else { palette.border },
                10.0,
                if active { 2.0 } else { 1.0 },
            );
            let name_rect = Rect::new(box_rect.x + 10.0, box_rect.y + 8.0, box_rect.w - 20.0, 28.0);
            if matches!(&ctx.rt.edit, Some(edit) if edit.target == EditTarget::ThemeName(index)) {
                let edit = ctx.rt.edit.as_ref().unwrap();
                let text = edit.text.clone();
                widgets::text_field(ui, &palette, name_rect, &text, "颜色方案名称");
                ctx.rt.edit_rect = name_rect;
            } else {
                let hover = ui.hover_rect(name_rect);
                ui.hit(name_rect, Action::ThemeNameEdit(index));
                ui.fill(name_rect, if hover { palette.soft } else { palette.inset }, 6.0);
                ui.text(
                    &theme.name,
                    Rect::new(name_rect.x + 8.0, name_rect.y, name_rect.w - 16.0, name_rect.h),
                    palette.text,
                    &Font::new(12.5, 400),
                );
            }
            swatches(ui, Rect::new(box_rect.x + 10.0, box_rect.y + 42.0, box_rect.w - 20.0, 14.0), theme);
            picker_row(
                ui,
                &palette,
                Rect::new(box_rect.x + 8.0, box_rect.y + 64.0, box_rect.w - 16.0, 36.0),
                theme,
                |field| ColorTarget::Custom(index, field),
            );
            let use_rect = Rect::new(box_rect.x + 10.0, box_rect.bottom() - 40.0, 96.0, 30.0);
            widgets::mini_button(ui, &palette, use_rect, "使用此颜色", Action::ThemeCustomUse(index), palette.work, active);
            let del_rect = Rect::new(use_rect.right() + 8.0, use_rect.y, 60.0, 30.0);
            widgets::mini_button(ui, &palette, del_rect, "删除", Action::ThemeCustomDelete(index), palette.muted, false);
            y = box_rect.bottom() + 10.0;
        }
    }

    let save_rect = Rect::new(area.x, y, area.w, 34.0);
    let hover = ui.is_hover(Action::ThemeSavePreset);
    ui.hit(save_rect, Action::ThemeSavePreset);
    ui.fill(save_rect, if hover { palette.long.lighten(0.08) } else { palette.long }, 8.0);
    ui.text("保存当前颜色为预设", save_rect, Color::WHITE, &Font::new(13.0, 600).center());
    y = save_rect.bottom() + 14.0;

    let divider = Rect::new(area.x, y, area.w, 1.0);
    ui.fill(divider, palette.border, 0.0);
    y = divider.bottom() + 12.0;

    let row = Rect::new(area.x, y, area.w, 28.0);
    widgets::switch_row(ui, &palette, row, "黑暗模式", app.visual.dark_mode, Action::ToggleDarkMode);
    y = row.bottom() + 8.0;
    let row = Rect::new(area.x, y, area.w, 28.0);
    widgets::switch_row(ui, &palette, row, "跟随系统", app.visual.follow_system, Action::ToggleFollowSystem);
    y = row.bottom() + 10.0;
    y
}

fn visual_ui(ui: &mut Ui, ctx: &mut Ctx, y: f32) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let app = ctx.app;
    let mut y = y;

    let row = Rect::new(area.x, y, area.w, 28.0);
    widgets::switch_row(
        ui,
        &palette,
        row,
        "无背景界面（只保留必要的卡片）",
        app.visual.bare_ui,
        Action::ToggleBareUi,
    );
    y = row.bottom() + 12.0;

    subtitle(ui, &palette, area, y, "窗口大小");
    y += 22.0;
    let cards = grid(Rect::new(area.x, y, area.w, 56.0), 4, 8.0);
    for (index, (w, h, label)) in WINDOW_SIZES.iter().enumerate() {
        let active = app.visual.window_size == index;
        let rect = cards[index];
        let action = Action::WindowSize(index);
        let hover = ui.is_hover(action.clone());
        ui.hit(rect, action);
        ui.fill(
            rect,
            if active {
                palette.work.with_alpha(0.12)
            } else if hover {
                palette.soft
            } else {
                palette.surface
            },
            8.0,
        );
        ui.stroke(rect, if active { palette.work } else { palette.border }, 8.0, if active { 2.0 } else { 1.0 });
        ui.text(
            label,
            Rect::new(rect.x, rect.y + 6.0, rect.w, 20.0),
            if active { palette.work } else { palette.text },
            &Font::new(13.0, if active { 600 } else { 400 }).center(),
        );
        ui.text(
            &format!("{}×{}", *w as u32, *h as u32),
            Rect::new(rect.x, rect.y + 28.0, rect.w, 18.0),
            palette.faint,
            &Font::new(11.0, 400).center(),
        );
    }
    y = cards[0].bottom() + 14.0;

    y
}

fn visual_text(ui: &mut Ui, ctx: &mut Ctx, y: f32) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let app = ctx.app;
    let mut y = y;

    let row = Rect::new(area.x, y, area.w, 34.0);
    ui.text("字体", Rect::new(row.x, row.y, 44.0, row.h), palette.text, &Font::new(13.0, 500));
    let label = drops::current_label(app, DropId::FontFamily);
    widgets::select_box(
        ui,
        &palette,
        Rect::new(row.x + 46.0, row.y, row.w - 46.0, row.h),
        &label,
        true,
        Action::DropToggle(DropId::FontFamily),
    );
    y = row.bottom() + 10.0;

    let size = app.visual.pomo_font_size.clamp(36.0, 80.0);
    let weight = app.visual.pomo_font_weight.clamp(200, 800);
    let value_w = 52.0;

    let row = Rect::new(area.x, y, area.w, 28.0);
    ui.text("大小", Rect::new(row.x, row.y, 40.0, row.h), palette.text, &Font::new(12.5, 500));
    let slider = Rect::new(row.x + 44.0, row.y, row.w - 44.0 - value_w - 8.0, row.h);
    widgets::slider(ui, &palette, slider, (size - 36.0) / 44.0, Action::SliderFontSize);
    ui.text(
        &format!("{}px", size as u32),
        Rect::new(slider.right() + 8.0, row.y, value_w, row.h),
        palette.faint,
        &Font::new(12.0, 400).right(),
    );
    y = row.bottom() + 6.0;

    let row = Rect::new(area.x, y, area.w, 28.0);
    ui.text("粗细", Rect::new(row.x, row.y, 40.0, row.h), palette.text, &Font::new(12.5, 500));
    let slider = Rect::new(row.x + 44.0, row.y, row.w - 44.0 - value_w - 8.0, row.h);
    widgets::slider(ui, &palette, slider, (weight as f32 - 200.0) / 600.0, Action::SliderFontWeight);
    ui.text(
        &weight.to_string(),
        Rect::new(slider.right() + 8.0, row.y, value_w, row.h),
        palette.faint,
        &Font::new(12.0, 400).right(),
    );
    y = row.bottom() + 10.0;

    let preview = Rect::new(area.x, y, area.w, 92.0);
    ui.fill(preview, palette.inset, 10.0);
    let font = Font::new(size, weight).center().family(&app.visual.pomo_font_family);
    ui.text("12:34", preview, palette.work, &font);
    y = preview.bottom() + 10.0;

    let row = Rect::new(area.x, y, area.w, 28.0);
    widgets::switch_row(ui, &palette, row, "隐藏界面文字（保留必要图标）", app.visual.hide_all_text, Action::ToggleHideAllText);
    y = row.bottom() + 10.0;
    y
}

fn subtitle(ui: &mut Ui, palette: &Palette, area: Rect, y: f32, text: &str) {
    ui.text(text, Rect::new(area.x, y, area.w, 20.0), palette.faint, &Font::new(12.0, 500));
}

fn swatches(ui: &mut Ui, rect: Rect, theme: &ThemeColors) {
    let values = [
        theme.work.as_str(),
        theme.short_break.as_str(),
        theme.long_break.as_str(),
        theme.pause.as_str(),
        theme.bg.as_str(),
        theme.surface.as_str(),
        theme.window.as_str(),
    ];
    let cell = rect.w / 7.0;
    for (index, value) in values.iter().enumerate() {
        ui.fill(Rect::new(rect.x + cell * index as f32, rect.y, cell - 1.0, rect.h), Color::from_hex(value), 2.0);
    }
}

fn switch_line(ui: &mut Ui, palette: &Palette, area: Rect, y: f32, label: &str, checked: bool, action: Action) -> f32 {
    let row = Rect::new(area.x, y, area.w, 28.0);
    widgets::switch_row(ui, palette, row, label, checked, action);
    row.bottom() + 8.0
}

fn slider_row(
    ui: &mut Ui,
    palette: &Palette,
    area: Rect,
    y: f32,
    label: &str,
    fraction: f32,
    value: String,
    action: Action,
) -> f32 {
    let row = Rect::new(area.x, y, area.w, 28.0);
    ui.text(label, Rect::new(row.x, row.y, 62.0, row.h), palette.text, &Font::new(13.0, 400));
    let value_w = 56.0;
    let slider = Rect::new(row.x + 64.0, row.y, (row.w - 64.0 - value_w - 8.0).max(40.0), row.h);
    widgets::slider(ui, palette, slider, fraction, action);
    ui.text(&value, Rect::new(slider.right() + 8.0, row.y, value_w, row.h), palette.faint, &Font::new(12.0, 400).right());
    row.bottom() + 8.0
}
fn picker_row(ui: &mut Ui, palette: &Palette, rect: Rect, theme: &ThemeColors, target: impl Fn(usize) -> ColorTarget) {
    let cells = grid(rect, 7, 3.0);
    for (field, cell) in cells.iter().enumerate() {
        let swatch = Rect::new(cell.x, cell.y, cell.w, 20.0);
        ui.hit(swatch, Action::ColorPick(target(field)));
        let color = Color::from_hex(crate::app_actions::theme_field(theme, field));
        widgets::swatch(ui, swatch, color, false);
        ui.text(
            FIELD_NAMES[field],
            Rect::new(cell.x, cell.y + 20.0, cell.w, 14.0),
            palette.faint,
            &Font::new(10.5, 400).center(),
        );
    }
}

fn widget_section(ui: &mut Ui, ctx: &mut Ctx, y: f32) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let app = ctx.app;
    let mut y = y;

    y = switch_line(ui, &palette, area, y, "显示桌面小图标", app.visual.widget_visible, Action::WidgetVisible(!app.visual.widget_visible));
    y = switch_line(ui, &palette, area, y, "点击穿透（开后就只能从托盘菜单关）", app.visual.widget_click_through, Action::WidgetClickThrough);
    y = switch_line(ui, &palette, area, y, "固定位置（开着时拖不动）", app.visual.widget_locked, Action::WidgetLock);
    y = switch_line(ui, &palette, area, y, "始终置顶", app.visual.widget_topmost, Action::WidgetTopmost);
    y += 6.0;

    let scale = app.visual.widget_scale.clamp(0.6, 1.8);
    y = slider_row(
        ui,
        &palette,
        area,
        y,
        "图标大小",
        (scale - 0.6) / 1.2,
        format!("{}%", (scale * 100.0).round() as u32),
        Action::SliderWidgetScale,
    );
    let size = app.visual.widget_font_size.clamp(14.0, 72.0);
    y = slider_row(
        ui,
        &palette,
        area,
        y,
        "数字大小",
        (size - 14.0) / 58.0,
        format!("{}px", size.round() as u32),
        Action::SliderWidgetFontSize,
    );
    let weight = app.visual.widget_font_weight.clamp(200, 800);
    y = slider_row(
        ui,
        &palette,
        area,
        y,
        "数字粗细",
        (weight as f32 - 200.0) / 600.0,
        weight.to_string(),
        Action::SliderWidgetFontWeight,
    );
    y += 6.0;
    ui.text(
        "待机只显示活动与数字 / 图形；点一下出按钮行，点 − 收起，15 秒没动静自动收起。",
        Rect::new(area.x, y, area.w, 18.0),
        palette.faint,
        &Font::new(11.5, 400),
    );
    ui.text(
        "按住任意位置都能拖动，位置自动记住；切了模块或改了设置没跟上就点左上角 ↻ 刷新。",
        Rect::new(area.x, y + 18.0, area.w, 18.0),
        palette.faint,
        &Font::new(11.5, 400),
    );
    y += 44.0;
    y
}
