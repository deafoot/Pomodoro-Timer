use crate::action::{Action, DropId, EditTarget, Section};
use crate::model::{fmt_time, RunStatus, Stage, VisualMode};
use crate::runtime::PresetDraft;
use crate::ui::icons;
use crate::ui::widgets::{self, TabItem};
use crate::ui::{grid, Font, Rect, Ui};

use super::{preset_card, Ctx};

pub fn draw(ui: &mut Ui, ctx: &mut Ctx) -> f32 {
    let palette = ctx.palette;
    let area = ctx.area;
    let now = ctx.now;
    // 下方的「计时方案」用草稿：改动不影响上方番茄钟，点「应用」才生效
    if ctx.open(Section::PomoPreset) && ctx.rt.draft.is_none() {
        ctx.rt.draft = Some(PresetDraft::from_app(ctx.app));
    }
    let pomo = &ctx.app.pomodoro;
    // 视口高度（不含滚动偏移）：紧凑档按它压缩布局，否则右边会挂一条滚动条
    let viewport = (area.h - ctx.scroll()).max(0.0);
    let compact = viewport < 480.0;
    let gap = if compact { 10.0 } else { 16.0 };
    let mut y = area.y + if compact { 8.0 } else { 12.0 };

    let stage_color = |stage: Stage| match stage {
        Stage::Focus => palette.work,
        Stage::ShortRest => palette.short,
        Stage::LongRest => palette.long,
    };
    let stage_icon = |stage: Stage| match stage {
        Stage::Focus => icons::TARGET,
        Stage::ShortRest => icons::LEAF,
        Stage::LongRest => icons::MOON,
    };

    // 阶段切换
    let tabs = Rect::new(area.x, y, area.w, if compact { 36.0 } else { 42.0 });
    let items: Vec<TabItem> = Stage::ALL
        .iter()
        .map(|stage| TabItem {
            label: stage.name().into(),
            icon: stage_icon(*stage).into(),
            active: pomo.stage == *stage,
            color: stage_color(*stage),
            action: Action::PomoStage(*stage),
        })
        .collect();
    widgets::mode_tabs(ui, &palette, tabs, &items, 3);
    y = tabs.bottom() + gap;

    // 活动选择（居中显示）
    let row = Rect::new(area.x, y, area.w, if compact { 34.0 } else { 38.0 });
    let editing = matches!(&ctx.rt.edit, Some(edit) if edit.target == EditTarget::ActivityName(true));
    if editing {
        let edit = ctx.rt.edit.as_ref().unwrap();
        let text = edit.text.clone();
        let rect = Rect::new(row.x, row.y, row.w - 44.0, row.h);
        widgets::text_field(ui, &palette, rect, &text, "输入活动名称，回车确认");
        ctx.rt.edit_rect = rect;
    } else {
        let label = pomo.activity.clone();
        widgets::select_box_centered(ui, &palette, row, &label, true, Action::DropToggle(DropId::PomoActivity));
    }
    widgets::icon_button(ui, &palette, Rect::new(row.right() - 40.0, row.y + 2.0, 36.0, row.h - 4.0), icons::EDIT, Action::ActivityEdit(true), None);
    y = row.bottom() + gap;

    // 计时显示：简洁 / 条形 / 环形
    let color = stage_color(pomo.stage);
    let mode = ctx.app.visual.mode;
    let hide_number = ctx.app.visual.hide_number;
    let visual = ctx.app.visual.clone();
    let remaining = pomo.remaining_sec(now);
    let progress = pomo.progress(now);
    // 显示块吸收剩余高度：紧凑档刚好塞满不挂滚动条，大窗口也不会空一大截
    let stats_h = if compact { 62.0 } else { 74.0 };
    let open_preset = ctx.open(Section::PomoPreset);
    let (min_h, max_h) = match mode {
        VisualMode::Simple => (92.0, 150.0),
        VisualMode::Bar => (108.0, 176.0),
        VisualMode::Ring => (120.0, 200.0),
    };
    let tail = gap * 3.0 + 46.0 + stats_h + 32.0 + if open_preset { 10.0 } else { 12.0 };
    let height = if open_preset {
        max_h
    } else {
        (viewport - (y - area.y) - tail).clamp(min_h, max_h)
    };
    let display = Rect::new(area.x, y, area.w, height);
    let font = |size: f32| {
        Font::new(size, visual.pomo_font_weight)
            .center()
            .essential()
            .family(&visual.pomo_font_family)
    };
    match mode {
        VisualMode::Simple => {
            ui.text(&fmt_time(remaining), display, color, &font(visual.pomo_font_size.clamp(36.0, 80.0)));
        }
        VisualMode::Bar => {
            let text_area = Rect::new(display.x, display.y, display.w, display.h - 44.0);
            if !hide_number {
                ui.text(&fmt_time(remaining), text_area, color, &font(visual.pomo_font_size.clamp(36.0, 80.0)));
            }
            let bar = Rect::new(display.x + 16.0, display.bottom() - 26.0, display.w - 32.0, 16.0);
            widgets::progress_bar(ui, &palette, bar, progress, color);
        }
        VisualMode::Ring => {
            let radius = (display.h.min(display.w) / 2.0 - 14.0).max(48.0);
            let cx = display.mid_x();
            let cy = display.mid_y();
            ui.ring(cx, cy, radius, 13.0, palette.inset, 1.0);
            ui.ring(cx, cy, radius, 13.0, color, progress);
            if !hide_number {
                let mut size = visual.pomo_font_size.clamp(36.0, 80.0).min(radius * 1.05);
                let room = ((radius - 13.0) * 2.0 - 10.0).max(20.0);
                let text = fmt_time(remaining);
                let measured = ui.text_width(&text, &font(size));
                if measured > room && measured > 0.5 {
                    size = (size * room / measured).max(16.0);
                }
                ui.text(&text, Rect::new(cx - radius, cy - radius, radius * 2.0, radius * 2.0), color, &font(size));
            }
        }
    }
    // 数字右边的「结束提示音」按钮：没有提示音时自动消失
    if ctx.rt.sound_active(now) {
        widgets::sound_stop_button(ui, &palette, display);
    }
    y = display.bottom() + gap;

    // 控制按钮
    let controls = Rect::new(area.x, y, area.w, 46.0);
    let cells = grid(controls, 3, 10.0);
    let (start_icon, start_label, start_color) = match pomo.status {
        RunStatus::Running => (icons::PAUSE, "暂停", color),
        RunStatus::Paused => (icons::PLAY, "继续", palette.pause),
        RunStatus::Idle => (icons::PLAY, "开始", color),
    };
    widgets::primary_button(ui, cells[0], start_label, start_icon, start_color, true, Action::PomoToggle);
    // 跳过 = 进下一阶段（专注 → 短休息 → 专注 …），跑着的话先把这一段记进历史
    widgets::ghost_button(ui, &palette, cells[1], "跳过", icons::SKIP, true, Action::PomoSkip);
    let reset_enabled = !(pomo.status == RunStatus::Idle && pomo.remaining_sec(now) == pomo.total_sec());
    widgets::ghost_button(ui, &palette, cells[2], "重置", icons::RESET, reset_enabled, Action::PomoReset);
    y = controls.bottom() + gap;

    // 统计
    let (focus_min, rest_min) = ctx.app.today_minutes();
    let cycle = ctx.app.preset().cycle.max(1);
    let stats = Rect::new(area.x, y, area.w, stats_h);
    let cells = grid(stats, 3, 10.0);
    widgets::stat_card(ui, &palette, cells[0], "今日专注", &format!("{} 分", focus_min), palette.work);
    widgets::stat_card(ui, &palette, cells[1], "今日休息", &format!("{} 分", rest_min), palette.short);
    widgets::stat_card(ui, &palette, cells[2], "当前循环", &format!("{} / {}", pomo.cycle, cycle), palette.text);
    y = stats.bottom() + gap;

    // 计时方案（可折叠，草稿 + 应用）：这里的文字不跟着「隐藏界面文字」一起藏
    let keep_text = ui.hide_all_text;
    ui.hide_all_text = false;
    let header = Rect::new(area.x, y, area.w, 32.0);
    widgets::section_header(
        ui,
        &palette,
        header,
        icons::PLAN,
        "计时方案",
        ctx.open(Section::PomoPreset),
        Action::ToggleSection(Section::PomoPreset),
    );
    y = header.bottom() + 10.0;
    if ctx.open(Section::PomoPreset) {
        let used = preset_card::draw(ui, ctx, Rect::new(area.x, y, area.w, 520.0), true);
        y += used + 10.0;
    }
    ui.hide_all_text = keep_text;

    y - area.y
}
