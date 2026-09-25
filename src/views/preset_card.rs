// 计时方案卡片（番茄钟视图与设置视图共用）
// 番茄钟视图（inline）走草稿：改动与选择都不影响上方的番茄钟，点「应用」才写回
use crate::action::{Action, DropId, EditTarget, NumberTarget};
use crate::model::{Preset, PresetField};
use crate::ui::icons;
use crate::ui::widgets;
use crate::ui::{Font, Rect, Ui};

use super::Ctx;

const ROW_H: f32 = 34.0;
const GAP: f32 = 8.0;
const LABEL_W: f32 = 52.0;
const NUM_W: f32 = 96.0;

pub fn draw(ui: &mut Ui, ctx: &mut Ctx, area: Rect, inline: bool) -> f32 {
    let palette = ctx.palette;
    let app = ctx.app;
    let source: Preset = match ctx.rt.draft_preset() {
        Some(draft) if inline => draft.clone(),
        _ => app.preset().clone(),
    };
    let name = source.name.clone();
    let (focus, short_rest, long_rest, cycle) = (source.focus, source.short_rest, source.long_rest, source.cycle);
    let (auto_focus, auto_rest, sound_loop) = (source.auto_focus, source.auto_rest, source.sound_loop);
    let mut y = area.y;

    // 头部：方案下拉 + 应用 / 新建 / 保存 / 删除
    let btn_gap = 6.0;
    let btn_h = ROW_H - 8.0;
    let mut specs: Vec<(String, Action, crate::color::Color, bool, f32)> = Vec::new();
    if inline {
        specs.push(("应用".into(), Action::PresetApply, palette.work, true, 52.0));
    }
    specs.push((icons::PLUS.into(), Action::PresetNew, palette.work, false, 32.0));
    if !inline {
        specs.push((icons::SAVE.into(), Action::PresetSave, palette.long, false, 32.0));
        specs.push((icons::CLOSE.into(), Action::PresetDelete, palette.work, false, 32.0));
    }
    let total: f32 = specs.iter().map(|spec| spec.4).sum::<f32>() + btn_gap * (specs.len() as f32 - 1.0);
    let head = Rect::new(area.x, y, area.w, ROW_H);
    let select_w = (head.w - total - btn_gap).max(90.0);
    widgets::select_box(
        ui,
        &palette,
        Rect::new(head.x, head.y, select_w, head.h),
        &name,
        true,
        Action::DropToggle(DropId::Preset),
    );
    let mut bx = head.x + select_w + btn_gap;
    for (label, action, color, filled, w) in specs {
        widgets::mini_button(ui, &palette, Rect::new(bx, head.y + 4.0, w, btn_h), &label, action, color, filled);
        bx += w + btn_gap;
    }
    y = head.bottom() + GAP;

    // 方案名称
    let row = Rect::new(area.x, y, area.w, ROW_H);
    ui.text("名称", Rect::new(row.x, row.y, LABEL_W, row.h), palette.faint, &Font::new(12.5, 500));
    let field = Rect::new(row.x + LABEL_W + 4.0, row.y, row.w - LABEL_W - 4.0, row.h);
    if matches!(&ctx.rt.edit, Some(edit) if edit.target == EditTarget::PresetName) {
        let edit = ctx.rt.edit.as_ref().unwrap();
        let text = edit.text.clone();
        widgets::text_field(ui, &palette, field, &text, "方案名称");
        ctx.rt.edit_rect = field;
    } else {
        let hover = ui.hover_rect(field);
        ui.hit(field, Action::PresetNameEdit);
        ui.fill(field, if hover { palette.soft } else { palette.surface }, 8.0);
        ui.stroke(field, palette.border, 8.0, 1.0);
        let font = Font::new(13.0, 400);
        ui.text(&name, Rect::new(field.x + 10.0, field.y, field.w - 20.0, field.h), palette.text, &font);
    }
    y = row.bottom() + GAP;

    // 阶段时长：事件下拉 + 分钟（可键盘输入，最小 0 分表示跳过该阶段）
    let rows = [
        ("专注", Some(DropId::FocusEvent), NumberTarget::Focus, focus, source.focus_event.clone()),
        ("短休", Some(DropId::ShortRestEvent), NumberTarget::ShortRest, short_rest, source.short_rest_event.clone()),
        ("长休", Some(DropId::LongRestEvent), NumberTarget::LongRest, long_rest, source.long_rest_event.clone()),
        ("长休前轮数", None, NumberTarget::Cycle, cycle, String::new()),
    ];
    for (label, drop, target, value, event) in rows {
        let row = Rect::new(area.x, y, area.w, ROW_H);
        let label_w = if drop.is_some() { LABEL_W } else { 84.0 };
        ui.text(label, Rect::new(row.x, row.y, label_w, row.h), palette.text, &Font::new(13.0, 500));
        let unit = if target == NumberTarget::Cycle { "轮" } else { "分" };
        let unit_w = 18.0;
        let stepper_rect = Rect::new(row.right() - unit_w - NUM_W - 2.0, row.y, NUM_W, row.h);
        if let Some(drop) = drop {
            let select_x = row.x + label_w + 4.0;
            let select_w = (stepper_rect.x - select_x - GAP).max(80.0);
            widgets::select_box(ui, &palette, Rect::new(select_x, row.y, select_w, row.h), &event, true, Action::DropToggle(drop));
        }
        if matches!(&ctx.rt.edit, Some(edit) if edit.target == EditTarget::Number(target)) {
            let edit = ctx.rt.edit.as_ref().unwrap();
            let text = edit.text.clone();
            widgets::text_field(ui, &palette, stepper_rect, &text, "0");
            ctx.rt.edit_rect = stepper_rect;
        } else {
            widgets::number_field(ui, &palette, stepper_rect, value, target, true);
        }
        ui.text(
            unit,
            Rect::new(stepper_rect.right() + 2.0, row.y, unit_w, row.h),
            palette.faint,
            &Font::new(12.0, 400),
        );
        y = row.bottom() + GAP;
    }

    // 提示音
    let row = Rect::new(area.x, y, area.w, ROW_H);
    ui.text("提示音", Rect::new(row.x, row.y, LABEL_W, row.h), palette.text, &Font::new(13.0, 500));
    let upload_w = 54.0;
    let select_x = row.x + LABEL_W;
    let select_w = (row.right() - select_x - upload_w - GAP).max(80.0);
    let sel = Rect::new(select_x, row.y, select_w, row.h);
    widgets::select_box(ui, &palette, sel, source.end_sound.label(), true, Action::DropToggle(DropId::SoundKind));
    widgets::border_button(
        ui,
        &palette,
        Rect::new(sel.right() + GAP, row.y + 4.0, upload_w, row.h - 8.0),
        "上传",
        Action::UploadSound,
        palette.text,
    );
    y = row.bottom() + 10.0;

    // 开关
    let switches = [
        ("自动开始专注", auto_focus, PresetField::AutoFocus),
        ("自动开始休息", auto_rest, PresetField::AutoRest),
        ("提示音循环", sound_loop, PresetField::SoundLoop),
    ];
    for (label, checked, field) in switches {
        let row = Rect::new(area.x, y, area.w, 28.0);
        widgets::switch_row(ui, &palette, row, label, checked, Action::PresetSwitch(field));
        y = row.bottom() + 4.0;
    }

    if inline {
        let hint = Rect::new(area.x, y + 2.0, area.w, 20.0);
        ui.text(
            "这里的改动与上方的番茄钟无关，点「应用」后才生效",
            hint,
            palette.faint,
            &Font::new(11.5, 400),
        );
        y = hint.bottom() + 4.0;
    }

    y + 6.0 - area.y
}