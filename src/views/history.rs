use crate::action::{Action, DropId};
use crate::clock;
use crate::color::Palette;
use crate::drops;
use crate::model::{fmt_duration, App, Group, HistoryItem};
use crate::ui::icons;
use crate::runtime::Runtime;
use crate::ui::widgets;
use crate::ui::{grid, Font, Rect, Ui};

use super::Ctx;

pub fn draw(ui: &mut Ui, ctx: &mut Ctx) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let app = ctx.app;
    let mut y = area.y + 12.0;

    // 过滤：类型 / 事件 / 日期范围
    let filters = Rect::new(area.x, y, area.w, 38.0);
    let cells = grid(filters, 2, 8.0);
    let group_label = drops::current_label(app, DropId::FilterGroup);
    widgets::select_box(ui, &palette, cells[0], &group_label, true, Action::DropToggle(DropId::FilterGroup));
    let event_label = drops::current_label(app, DropId::FilterEvent);
    widgets::select_box(ui, &palette, cells[1], &event_label, true, Action::DropToggle(DropId::FilterEvent));
    y = filters.bottom() + 8.0;

    let date_text = match (&app.filter.date_start, &app.filter.date_end) {
        (None, _) => format!("{} 选择日期范围", icons::CALENDAR),
        (Some(start), Some(end)) if start != end => format!("{} {} ~ {}", icons::CALENDAR, start, end),
        (Some(start), _) => format!("{} {}", icons::CALENDAR, start),
    };
    let date_btn = Rect::new(area.x, y, area.w, 36.0);
    widgets::border_button(ui, &palette, date_btn, &date_text, Action::CalendarToggle, palette.text);
    y = date_btn.bottom() + 10.0;

    // 批量操作
    let visible = app.visible_history();
    let all_selected = !visible.is_empty() && visible.iter().all(|item| ctx.rt.selected.contains(&item.id));
    let batch = Rect::new(area.x, y, area.w, 30.0);
    widgets::checkbox(
        ui,
        &palette,
        Rect::new(batch.x, batch.mid_y() - 9.0, 18.0, 18.0),
        all_selected,
        Action::SelectAll(!all_selected),
    );
    ui.text("全选", Rect::new(batch.x + 26.0, batch.y, 60.0, batch.h), palette.text, &Font::new(13.0, 400));
    ui.text(
        &format!("已选 {} 项", ctx.rt.selected.len()),
        Rect::new(batch.x + 90.0, batch.y, batch.w - 180.0, batch.h),
        palette.faint,
        &Font::new(12.5, 400),
    );
    let delete_rect = Rect::new(batch.right() - 66.0, batch.y + 2.0, 66.0, batch.h - 4.0);
    let delete_hover = ui.is_hover(Action::BatchDelete);
    ui.hit(delete_rect, Action::BatchDelete);
    ui.fill(delete_rect, if delete_hover { palette.work.with_alpha(0.18) } else { palette.work.with_alpha(0.10) }, 8.0);
    ui.text("删除", delete_rect, palette.work, &Font::new(12.5, 600).center());
    y = batch.bottom() + 10.0;

    // 统计
    let (focus_min, rest_min) = app.minutes_of(&visible);
    let stats = Rect::new(area.x, y, area.w, 66.0);
    let cells = grid(stats, 3, 8.0);
    widgets::stat_card(ui, &palette, cells[0], "专注分钟", &focus_min.to_string(), palette.work);
    widgets::stat_card(ui, &palette, cells[1], "休息分钟", &rest_min.to_string(), palette.short);
    widgets::stat_card(ui, &palette, cells[2], "记录数", &visible.len().to_string(), palette.text);
    y = stats.bottom() + 10.0;

    // 记录列表
    if visible.is_empty() {
        let empty = Rect::new(area.x, y, area.w, 96.0);
        ui.fill(empty, palette.inset, 12.0);
        ui.text("没有符合条件的记录", empty, palette.faint, &Font::new(13.0, 400).center());
        y = empty.bottom() + 8.0;
        return y - area.y;
    }

    for item in visible.iter() {
        let row = Rect::new(area.x, y, area.w, 54.0);
        ui.fill(row, palette.inset, 10.0);
        let checked = ctx.rt.selected.contains(&item.id);
        widgets::checkbox(
            ui,
            &palette,
            Rect::new(row.x + 10.0, row.mid_y() - 9.0, 18.0, 18.0),
            checked,
            Action::ToggleSelect(item.id),
        );
        let focus = item.kind.group() == Group::Focus;
        let dot_color = if focus { palette.work } else { palette.short };
        ui.circle(row.x + 44.0, row.mid_y(), 13.0, dot_color.with_alpha(0.16));
        let icon = if focus { icons::TARGET } else { icons::MOON };
        ui.text(icon, Rect::new(row.x + 31.0, row.mid_y() - 11.0, 26.0, 22.0), dot_color, &Font::new(13.0, 500).center().essential());
        ui.text(
            &item.name,
            Rect::new(row.x + 66.0, row.y + 7.0, row.w - 160.0, 20.0),
            palette.text,
            &Font::new(13.5, 600),
        );
        ui.text(
            &meta(item),
            Rect::new(row.x + 66.0, row.y + 27.0, row.w - 160.0, 18.0),
            palette.faint,
            &Font::new(11.5, 400),
        );
        ui.text(
            &fmt_duration(item.seconds),
            Rect::new(row.right() - 88.0, row.y + 10.0, 78.0, 24.0),
            dot_color,
            &Font::new(13.0, 600).right(),
        );
        y = row.bottom() + 8.0;
    }

    y - area.y
}

fn meta(item: &HistoryItem) -> String {
    let mut text = format!("{} · {} {}", item.kind.name(), item.date, item.time);
    if !item.completed {
        text.push_str(" · 提前结束");
    }
    if item.source == crate::model::Source::Pomodoro {
        text.push_str(" · 番茄钟");
    }
    text
}

pub fn draw_calendar_popup(ui: &mut Ui, _app: &App, rt: &mut Runtime, palette: &Palette) {
    let anchor = rt.drop_anchor;
    let w = 306.0;
    let h = 286.0;
    let x = anchor.x.min((ui.width - w - 8.0).max(8.0)).max(8.0);
    let y = (anchor.bottom() + 6.0).min((ui.height - h - 8.0).max(8.0)).max(8.0);
    let panel = Rect::new(x, y, w, h);
    ui.fill(panel.offset(0.0, 3.0), crate::color::Color::rgba(0, 0, 0, 0.16), 12.0);
    ui.fill(panel, palette.surface, 12.0);
    ui.stroke(panel, palette.border, 12.0, 1.0);

    let year = rt.calendar.year;
    let month = rt.calendar.month;
    let today = clock::today();
    let start = rt.calendar.start.clone();
    let end = rt.calendar.end.clone();

    // 月份切换
    let header = Rect::new(panel.x + 8.0, panel.y + 8.0, panel.w - 16.0, 30.0);
    let prev = Rect::new(header.x, header.y, 30.0, header.h);
    ui.hit(prev, Action::CalendarPrev);
    ui.text("‹", prev, palette.text, &Font::new(16.0, 600).center());
    let next = Rect::new(header.right() - 30.0, header.y, 30.0, header.h);
    ui.hit(next, Action::CalendarNext);
    ui.text("›", next, palette.text, &Font::new(16.0, 600).center());
    ui.text(
        &format!("{} 年 {} 月", year, month),
        header,
        palette.text,
        &Font::new(13.0, 600).center(),
    );

    // 星期表头
    let weekdays = ["日", "一", "二", "三", "四", "五", "六"];
    let grid_x = panel.x + 8.0;
    let grid_w = panel.w - 16.0;
    let cell_w = grid_w / 7.0;
    let head_y = header.bottom() + 2.0;
    for (index, name) in weekdays.iter().enumerate() {
        let cell = Rect::new(grid_x + cell_w * index as f32, head_y, cell_w, 20.0);
        ui.text(name, cell, palette.faint, &Font::new(11.5, 500).center());
    }

    // 日期网格
    let days = clock::days_in_month(year, month);
    let lead = clock::first_weekday(year, month);
    let prev_days = if month == 1 {
        clock::days_in_month(year - 1, 12)
    } else {
        clock::days_in_month(year, month - 1)
    };
    let cell_h = 26.0;
    let grid_y = head_y + 22.0;
    for slot in 0..42u32 {
        let col = slot % 7;
        let row = slot / 7;
        let cell = Rect::new(grid_x + cell_w * col as f32, grid_y + cell_h * row as f32, cell_w, cell_h);
        if slot < lead {
            let day = prev_days - lead + slot + 1;
            ui.text(&day.to_string(), cell, palette.faint.with_alpha(0.55), &Font::new(12.0, 400).center());
            continue;
        }
        let index = slot - lead;
        if index >= days {
            let day = index - days + 1;
            ui.text(&day.to_string(), cell, palette.faint.with_alpha(0.55), &Font::new(12.0, 400).center());
            continue;
        }
        let day = index + 1;
        let date = clock::date_string(year, month, day);
        let selected = match (&start, &end) {
            (Some(s), Some(e)) => &date >= s && &date <= e,
            (Some(s), None) => &date == s,
            _ => false,
        };
        let hover = ui.hover_rect(cell.inset(2.0, 1.0));
        ui.hit(cell.inset(2.0, 1.0), Action::CalendarPickDay(day));
        if selected {
            ui.fill(cell.inset(2.0, 1.0), palette.work.with_alpha(0.16), 6.0);
        } else if hover {
            ui.fill(cell.inset(2.0, 1.0), palette.soft, 6.0);
        }
        if date == today {
            ui.stroke(cell.inset(2.0, 1.0), palette.work.with_alpha(0.6), 6.0, 1.0);
        }
        let color = if selected { palette.work } else { palette.text };
        ui.text(&day.to_string(), cell, color, &Font::new(12.5, if selected { 600 } else { 400 }).center());
    }

    // 操作
    let buttons = Rect::new(panel.x + 10.0, panel.bottom() - 42.0, panel.w - 20.0, 32.0);
    let cells = grid(buttons, 2, 8.0);
    widgets::border_button(ui, palette, cells[0], "清除", Action::CalendarClear, palette.text);
    widgets::mini_button(ui, &palette, cells[1], "应用", Action::CalendarApply, palette.short, true);
}

