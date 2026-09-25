use crate::action::{Action, DropId, ViewId};
use crate::color::{Color, Palette};
use crate::drops;
use crate::model::{fmt_time, App, Kind, RunStatus, Stage, VisualMode};
use crate::runtime::{Runtime, WidgetMode};
use crate::ui::icons;
use crate::ui::widgets;
use crate::ui::{Font, Rect, Ui};

/// 无边框形态：倒计时块。待机时只显示这块 + 活动，展开后才出现按钮
pub const BARE_W: f32 = 186.0;
/// 展开后的内容宽度（要放下图标行三组共 8 个按钮）
pub const WIDE_W: f32 = 250.0;

/// 倒计时块高度跟着字号走：数字调大了也不会挤到活动行 / 按钮上
pub fn block_h(app: &App) -> f32 {
    (app.visual.widget_font_size.clamp(14.0, 96.0) * 1.4).clamp(48.0, 150.0)
}

const ACTIVITY_H: f32 = 26.0;
const BTN_H: f32 = 30.0;
const GAP: f32 = 6.0;
const LIST_ROW: f32 = 26.0;
const LIST_MAX: usize = 6;
/// 图标行按钮（刷新 / 关闭 / 回主界面 / 待机切换）的边长
const CORNER: f32 = 26.0;

/// 悬浮窗跟随的模块：主界面切到计时 / 番茄钟时跟着走，
/// 也可以在小图标上直接点 计时 / 番茄钟 指定；切到历史 / 设置时保持不变
pub fn module(_app: &App, rt: &Runtime) -> ViewId {
    rt.widget_module
}

pub fn activity_drop(module: ViewId) -> DropId {
    if module == ViewId::Pomodoro {
        DropId::PomoActivity
    } else {
        DropId::TimerActivity
    }
}

/// 活动列表是否正由悬浮窗展开
pub fn list_open(app: &App, rt: &Runtime) -> bool {
    rt.widget_drop && rt.open_drop == Some(activity_drop(module(app, rt)))
}

fn list_rows(app: &App, module: ViewId) -> usize {
    drops::items(app, activity_drop(module)).len().min(LIST_MAX)
}

/// 悬浮窗尺寸（DIP，由 platform::widget 按缩放换算成像素）。
/// 自上而下：活动行 →〔活动下拉列表〕→ 数字 / 图形块 →〔图标行〕→〔控制行〕
pub fn size(app: &App, rt: &Runtime, now: u64) -> (f32, f32) {
    let module = module(app, rt);
    let mode = rt.widget_mode;
    let list = list_open(app, rt);
    let width = if mode.buttons() || list { WIDE_W } else { BARE_W };
    let mut height = ACTIVITY_H + GAP + block_h(app);
    if list {
        height += GAP + list_rows(app, module) as f32 * LIST_ROW + 8.0;
    } else if mode.buttons() {
        height += GAP + CORNER;
        if mode.running() {
            if rt.sound_active(now) {
                height += GAP + BTN_H;
            }
            height += GAP + BTN_H;
        }
    }
    (width, height)
}

struct Info {
    text: String,
    color: Color,
    fraction: f32,
    activity: String,
}

fn info(app: &App, rt: &Runtime, now: u64) -> Info {
    let palette = rt.palette;
    match module(app, rt) {
        ViewId::Pomodoro => {
            let pomo = &app.pomodoro;
            let stage_color = match pomo.stage {
                Stage::Focus => palette.work,
                Stage::ShortRest => palette.short,
                Stage::LongRest => palette.long,
            };
            Info {
                text: fmt_time(pomo.remaining_sec(now)),
                color: if pomo.status == RunStatus::Paused { palette.pause } else { stage_color },
                // 整秒跳变（一顿一顿），不要每帧连续推进
                fraction: pomo.remaining_sec(now) as f32 / pomo.total_sec().max(1) as f32,
                activity: pomo.activity.clone(),
            }
        }
        _ => {
            let timer = &app.timer;
            let elapsed = timer.elapsed_sec(now);
            let color = if timer.mode == Kind::Focus { palette.work } else { palette.short };
            Info {
                text: fmt_time(elapsed),
                color: if timer.status == RunStatus::Paused { palette.pause } else { color },
                fraction: (elapsed % 3600) as f32 / 3600.0,
                activity: timer.activity.clone(),
            }
        }
    }
}

pub fn draw(ui: &mut Ui, app: &App, rt: &mut Runtime, now: u64) {
    let palette = rt.palette;
    let mode = rt.widget_mode;
    let list = list_open(app, rt);
    let module = module(app, rt);
    let info = info(app, rt, now);
    let (width, height) = size(app, rt, now);

    // 分层窗口是按 alpha 做命中测试的：alpha = 0 的像素收不到鼠标，
    // 所以整块铺一层 alpha = 1/255 的底，才能「点在数字旁边的空白上」也算按住悬浮窗
    ui.fill(
        Rect::new(0.0, 0.0, width, height),
        Color::rgba(255, 255, 255, 1.0 / 255.0),
        0.0,
    );

    let row_w = (width - 12.0).max(120.0);
    let row_x = (width - row_w) / 2.0;

    // 活动行在最上面，活动下拉列表紧贴它下面展开
    let interactive = mode.buttons() && !list;
    draw_activity(
        ui,
        rt,
        &palette,
        module,
        Rect::new(row_x, 0.0, row_w, ACTIVITY_H),
        &info,
        interactive,
    );
    let mut top = ACTIVITY_H + GAP;
    if list {
        let rows = list_rows(app, module) as f32 * LIST_ROW + 8.0;
        draw_list(ui, app, &palette, module, Rect::new(row_x, top, row_w, rows));
        top += rows + GAP;
    }
    // 数字 / 图形块：宽度固定、水平居中，图标不再压在它上面
    let block_h = block_h(app);
    let bx = ((width - BARE_W) / 2.0).max(0.0);
    draw_number(ui, app, &palette, &info, Rect::new(bx, top, BARE_W, block_h), 6.0);
    top += block_h + GAP;
    if list {
        // 下拉展开时收起按钮行，面板不至于拖太长
        return;
    }
    draw_icon_row(ui, app, &palette, &info, mode, module, Rect::new(row_x, top, row_w, CORNER));
    top += CORNER + GAP;
    if !mode.running() {
        return;
    }
    // 有提示音时先给一行「结束提示音」，没有就不占位置
    if rt.sound_active(now) {
        widgets::sound_stop_button(ui, &palette, Rect::new(row_x, top, row_w, BTN_H));
        top += BTN_H + GAP;
    }
    draw_buttons(ui, app, &palette, module, Rect::new(row_x, top, row_w, BTN_H));
}

/// 图标行：左边 [刷新][关闭]，右边 [固定][穿透][回主界面][待机切换]。
/// 中间是 [计时][番茄钟]，直接指定小图标跟哪个模块走。
/// 待机时最右边是 ＋（回运行模式），运行时是 −（收起），位置一致不跳。
fn draw_icon_row(
    ui: &mut Ui,
    app: &App,
    palette: &Palette,
    info: &Info,
    mode: WidgetMode,
    module: ViewId,
    rect: Rect,
) {
    let side = CORNER;
    let right = rect.right() - side;
    widgets::icon_button(ui, palette, Rect::new(rect.x, rect.y, side, side), icons::REFRESH, Action::WidgetRefresh, None);
    widgets::icon_button(
        ui,
        palette,
        Rect::new(rect.x + side + 2.0, rect.y, side, side),
        icons::CLOSE,
        Action::WidgetClose,
        None,
    );
    // 中间组：计时 / 番茄钟，夹在左右两组中间，当前跟的那个用阶段色标出来
    let left_w = 2.0 * side + 2.0;
    let middle_w = 2.0 * side + 2.0;
    let right_w = 4.0 * side + 6.0;
    let middle = rect.x + left_w + (rect.w - left_w - middle_w - right_w) / 2.0;
    widgets::icon_button(
        ui,
        palette,
        Rect::new(middle, rect.y, side, side),
        icons::WATCH,
        Action::WidgetModule(ViewId::Timer),
        if module == ViewId::Timer { Some(info.color) } else { None },
    );
    widgets::icon_button(
        ui,
        palette,
        Rect::new(middle + side + 2.0, rect.y, side, side),
        icons::TARGET,
        Action::WidgetModule(ViewId::Pomodoro),
        if module == ViewId::Pomodoro { Some(info.color) } else { None },
    );
    // 开关类按钮：开着的时候用阶段色标出来
    let locked = app.visual.widget_locked;
    let through = app.visual.widget_click_through;
    widgets::icon_button(
        ui,
        palette,
        Rect::new(right - 3.0 * side - 6.0, rect.y, side, side),
        icons::PIN,
        Action::WidgetLock,
        if locked { Some(info.color) } else { None },
    );
    widgets::icon_button(
        ui,
        palette,
        Rect::new(right - 2.0 * side - 4.0, rect.y, side, side),
        icons::THROUGH,
        Action::WidgetClickThrough,
        if through { Some(info.color) } else { None },
    );
    widgets::icon_button(
        ui,
        palette,
        Rect::new(right - side - 2.0, rect.y, side, side),
        icons::EXPAND,
        Action::WidgetOpen,
        Some(info.color),
    );
    let (icon, tone) = if mode.running() { (icons::MINIMIZE, None) } else { (icons::PLUS, Some(info.color)) };
    widgets::icon_button(ui, palette, Rect::new(right, rect.y, side, side), icon, Action::WidgetModeToggle, tone);
}

fn draw_number(ui: &mut Ui, app: &App, palette: &Palette, info: &Info, rect: Rect, reserve: f32) {
    let size = app.visual.widget_font_size.clamp(14.0, 96.0);
    let weight = app.visual.widget_font_weight.clamp(200, 800);
    let family = app.visual.pomo_font_family.clone();
    let font = |size: f32| Font::new(size, weight).center().essential().family(&family);
    // 数字再大也不许压到按钮，也不许超出块宽
    let room = (rect.w - reserve * 2.0).max(40.0);
    let mut size = size;
    let measured = ui.text_width(&info.text, &font(size));
    if measured > room && measured > 0.5 {
        size = (size * room / measured).max(9.0);
    }
    match app.visual.mode {
        VisualMode::Simple => {
            ui.text(&info.text, rect, info.color, &font(size));
        }
        VisualMode::Bar => {
            let digits = Rect::new(rect.x, rect.y - 4.0, rect.w, rect.h - 12.0);
            ui.text(&info.text, digits, info.color, &font(size.min(rect.h - 16.0).max(14.0)));
            let bar = Rect::new(rect.x + 12.0, rect.bottom() - 10.0, rect.w - 24.0, 7.0);
            // 轨道带上阶段色：进度为 0 时也不会剩一条纯白的底
            let track = Palette { inset: Color::mix(palette.inset, info.color, 0.3), ..*palette };
            widgets::progress_bar(ui, &track, bar, info.fraction, info.color);
        }
        VisualMode::Ring => {
            let radius = (rect.h.min(rect.w) / 2.0 - 4.0).max(12.0);
            let thickness = 5.0;
            let cx = rect.mid_x();
            let cy = rect.mid_y();
            // 轨道带上阶段色：进度为 0（比如还没开始）时也不会剩一条纯白的环
            ui.ring(cx, cy, radius, thickness, Color::mix(palette.inset, info.color, 0.3), 1.0);
            ui.ring(cx, cy, radius, thickness, info.color, info.fraction);
            // 数字要塞进环内：先按字号量一下宽度，超了就等比缩
            let room = ((radius - thickness) * 2.0 - 6.0).max(12.0);
            let mut inner = size.min(radius * 1.2).max(9.0);
            let measured = ui.text_width(&info.text, &font(inner));
            if measured > room && measured > 0.5 {
                inner = (inner * room / measured).max(9.0);
            }
            ui.text(&info.text, Rect::new(cx - radius, cy - radius, radius * 2.0, radius * 2.0), info.color, &font(inner));
        }
    }
}

/// 活动行：整体居中（[圆点] 活动 [✎]），文字与圆点跟数字同色
fn draw_activity(
    ui: &mut Ui,
    rt: &mut Runtime,
    palette: &Palette,
    module: ViewId,
    rect: Rect,
    info: &Info,
    interactive: bool,
) {
    let drop_id = activity_drop(module);
    if rt.widget_editing && interactive {
        let edit = rt.edit.as_ref();
        let text = edit.map(|edit| edit.text.clone()).unwrap_or_else(|| info.activity.clone());
        let field = Rect::new(rect.x, rect.y + 1.0, rect.w, rect.h - 2.0);
        widgets::text_field(ui, palette, field, &text, "输入活动名称，回车确认");
        rt.widget_edit_rect = field;
        return;
    }
    let font = Font::new(12.5, 600);
    let btn = 24.0;
    // ● + 活动名严格居中；✎ 挂在文字右边、不参与居中，但左右留白对称
    let shown = widgets::ellipsize(ui, &info.activity, (rect.w - 2.0 * (btn + 8.0) - 14.0).max(20.0), &font);
    let text_w = ui.text_width(&shown, &font);
    let group = 14.0 + text_w;
    let start = rect.x + ((rect.w - group) / 2.0).max(0.0);
    ui.circle(start + 4.0, rect.mid_y(), 3.5, info.color);
    let name = Rect::new(start + 12.0, rect.y, text_w, rect.h);
    ui.text(&shown, name, info.color, &font);
    if interactive {
        ui.hit(name, Action::DropToggle(drop_id));
        let edit_btn = Rect::new(name.right() + 8.0, rect.y + 1.0, btn, rect.h - 2.0);
        widgets::icon_button(ui, palette, edit_btn, icons::EDIT, Action::WidgetEdit, None);
    }
}

fn draw_buttons(ui: &mut Ui, app: &App, palette: &Palette, module: ViewId, rect: Rect) {
    let cells = crate::ui::grid(rect, 3, 6.0);
    if module == ViewId::Pomodoro {
        let pomo = &app.pomodoro;
        let color = match pomo.stage {
            Stage::Focus => palette.work,
            Stage::ShortRest => palette.short,
            Stage::LongRest => palette.long,
        };
        let (icon, label, tone) = match pomo.status {
            RunStatus::Running => (icons::PAUSE, "暂停", color),
            RunStatus::Paused => (icons::PLAY, "继续", palette.pause),
            RunStatus::Idle => (icons::PLAY, "开始", color),
        };
        widgets::mini_button(ui, palette, cells[0], &format!("{} {}", icon, label), Action::PomoToggle, tone, true);
        // 跳过 = 进下一阶段（跑着的话先把这一段记进历史）
        widgets::mini_button(ui, palette, cells[1], &format!("{} 跳过", icons::SKIP), Action::PomoSkip, palette.text, false);
        widgets::mini_button(ui, palette, cells[2], &format!("{} 重置", icons::RESET), Action::PomoReset, palette.text, false);
    } else {
        let timer = &app.timer;
        let color = if timer.mode == Kind::Focus { palette.work } else { palette.short };
        let (icon, label, tone) = match timer.status {
            RunStatus::Running => (icons::PAUSE, "暂停", color),
            RunStatus::Paused => (icons::PLAY, "继续", palette.pause),
            RunStatus::Idle => (icons::PLAY, "开始", color),
        };
        widgets::mini_button(ui, palette, cells[0], &format!("{} {}", icon, label), Action::TimerToggle, tone, true);
        // 空闲时是「跳过」：切到另一个状态（专注 ↔ 休息）；跑着时是「结束」：存历史 + 停止，不动模式
        let (mid_icon, mid_label) = if timer.status == RunStatus::Idle { (icons::SKIP, "跳过") } else { (icons::STOP, "结束") };
        widgets::mini_button(ui, palette, cells[1], &format!("{} {}", mid_icon, mid_label), Action::TimerSkip, palette.text, false);
        widgets::mini_button(ui, palette, cells[2], &format!("{} 重置", icons::RESET), Action::TimerReset, palette.text, false);
    }
}

fn draw_list(ui: &mut Ui, app: &App, palette: &Palette, module: ViewId, rect: Rect) {
    let id = activity_drop(module);
    let items = drops::items(app, id);
    let selected = drops::selected(app, id);
    ui.fill(rect, palette.surface, 10.0);
    ui.stroke(rect, palette.border, 10.0, 1.0);
    let rows = items.len().min(LIST_MAX);
    for index in 0..rows {
        let row = Rect::new(rect.x + 4.0, rect.y + 4.0 + index as f32 * LIST_ROW, rect.w - 8.0, LIST_ROW);
        let action = Action::DropPick(id, index);
        ui.hit(row, action.clone());
        if index == selected {
            ui.fill(row, palette.soft, 6.0);
        }
        let color = if index == selected { palette.work } else { palette.text };
        let font = Font::new(12.5, 400);
        let shown = widgets::ellipsize(ui, &items[index], row.w - 16.0, &font);
        ui.text(&shown, Rect::new(row.x + 8.0, row.y, row.w - 16.0, row.h), color, &font);
    }
}
