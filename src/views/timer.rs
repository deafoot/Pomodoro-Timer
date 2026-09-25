use crate::action::{Action, DropId, EditTarget};
use crate::model::{fmt_time, Kind, RunStatus};
use crate::ui::icons;
use crate::ui::widgets::{self, TabItem};
use crate::ui::{grid, Font, Rect, Ui};

use super::Ctx;

pub fn draw(ui: &mut Ui, ctx: &mut Ctx) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let now = ctx.now;
    let timer = &ctx.app.timer;
    // 视口高度（不含滚动偏移）：紧凑档按它压缩布局
    let viewport = (area.h - ctx.scroll()).max(0.0);
    let compact = viewport < 460.0;
    let gap = if compact { 10.0 } else { 16.0 };
    let mut y = area.y + if compact { 8.0 } else { 12.0 };

    // 模式切换
    let tabs = Rect::new(area.x, y, area.w, if compact { 36.0 } else { 42.0 });
    let items = [
        TabItem {
            label: "专注".into(),
            icon: icons::TARGET.into(),
            active: timer.mode == Kind::Focus,
            color: palette.work,
            action: Action::TimerMode(Kind::Focus),
        },
        TabItem {
            label: "休息".into(),
            icon: icons::LEAF.into(),
            active: timer.mode == Kind::Rest,
            color: palette.short,
            action: Action::TimerMode(Kind::Rest),
        },
    ];
    widgets::mode_tabs(ui, &palette, tabs, &items, 2);
    y = tabs.bottom() + gap;

    // 活动选择（居中显示）
    let row = Rect::new(area.x, y, area.w, if compact { 34.0 } else { 38.0 });
    let editing = matches!(&ctx.rt.edit, Some(edit) if edit.target == EditTarget::ActivityName(false));
    if editing {
        let edit = ctx.rt.edit.as_ref().unwrap();
        let rect = Rect::new(row.x, row.y, row.w - 44.0, row.h);
        let text = edit.text.clone();
        widgets::text_field(ui, &palette, rect, &text, "输入活动名称，回车确认");
        ctx.rt.edit_rect = rect;
    } else {
        let label = ctx.app.timer.activity.clone();
        widgets::select_box_centered(ui, &palette, row, &label, true, Action::DropToggle(DropId::TimerActivity));
    }
    widgets::icon_button(ui, &palette, Rect::new(row.right() - 40.0, row.y + 2.0, 36.0, row.h - 4.0), icons::EDIT, Action::ActivityEdit(false), None);
    y = row.bottom() + gap;

    // 计时显示：吸收剩余高度，紧凑档不溢出
    let stats_h = if compact { 62.0 } else { 74.0 };
    let tail = gap * 2.0 + 46.0 + stats_h + 12.0;
    let height = (viewport - (y - area.y) - tail).clamp(96.0, 176.0);
    let display = Rect::new(area.x, y, area.w, height);
    let color = if timer.mode == Kind::Focus { palette.work } else { palette.short };
    let font = Font::new(ctx.app.visual.pomo_font_size.clamp(36.0, 80.0), ctx.app.visual.pomo_font_weight)
        .center()
        .essential()
        .family(&ctx.app.visual.pomo_font_family);
    let elapsed = timer.elapsed_sec(now);
    ui.text(&fmt_time(elapsed), display, color, &font);
    if ctx.rt.sound_active(now) {
        widgets::sound_stop_button(ui, &palette, display);
    }
    y = display.bottom() + gap;

    // 控制按钮
    let controls = Rect::new(area.x, y, area.w, 46.0);
    let cells = grid(controls, 3, 10.0);
    let (start_icon, start_label, start_color) = match timer.status {
        RunStatus::Running => (icons::PAUSE, "暂停", color),
        RunStatus::Paused => (icons::PLAY, "继续", palette.pause),
        RunStatus::Idle => (icons::PLAY, "开始", color),
    };
    widgets::primary_button(ui, cells[0], start_label, start_icon, start_color, true, Action::TimerToggle);
    // 空闲时是「跳过」：切到另一个状态（专注 ↔ 休息）；跑着时是「结束」：存历史 + 停止，不动模式
    let (skip_icon, skip_label) = if timer.status == RunStatus::Idle { (icons::SKIP, "跳过") } else { (icons::STOP, "结束") };
    widgets::ghost_button(ui, &palette, cells[1], skip_label, skip_icon, true, Action::TimerSkip);
    let reset_enabled = !(timer.status == RunStatus::Idle && timer.elapsed_ms(now) == 0);
    widgets::ghost_button(ui, &palette, cells[2], "重置", icons::RESET, reset_enabled, Action::TimerReset);
    y = controls.bottom() + gap;

    // 今日统计
    let (focus_min, rest_min) = ctx.app.today_minutes();
    let stats = Rect::new(area.x, y, area.w, stats_h);
    let cells = grid(stats, 2, 10.0);
    widgets::stat_card(ui, &palette, cells[0], "今日专注", &format!("{} 分", focus_min), palette.work);
    widgets::stat_card(ui, &palette, cells[1], "今日休息", &format!("{} 分", rest_min), palette.short);
    y = stats.bottom() + 12.0;

    y - area.y
}
