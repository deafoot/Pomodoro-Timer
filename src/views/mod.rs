pub mod history;
pub mod pomodoro;
pub mod widget_view;
pub mod preset_card;
pub mod settings;
pub mod timer;

use crate::action::{Action, ViewId};
use crate::color::Palette;
use crate::drops;
use crate::model::App;
use crate::runtime::Runtime;
use crate::ui::widgets;
use crate::ui::{Font, Rect, Ui};

pub const CARD_MAX_W: f32 = 540.0;
pub const MARGIN: f32 = 14.0;
pub const PAD: f32 = 16.0;
pub const NAV_H: f32 = 58.0;

pub struct Ctx<'a> {
    pub app: &'a App,
    pub rt: &'a mut Runtime,
    pub palette: Palette,
    pub now: u64,
    /// 内容坐标系（已应用滚动偏移）
    pub area: Rect,
}

impl<'a> Ctx<'a> {
    pub fn open(&self, section: crate::action::Section) -> bool {
        self.rt.is_open(section)
    }

    pub fn scroll(&self) -> f32 {
        self.rt.scroll_of(self.rt.view)
    }
}

pub fn draw(ui: &mut Ui, app: &App, rt: &mut Runtime, now: u64) {
    let palette = rt.palette;
    let width = ui.width;
    let height = ui.height;
    let bare = app.visual.bare_ui;
    ui.fill(Rect::new(0.0, 0.0, width, height), palette.page, 0.0);
    // 无背景界面：不画外层卡片，内容直接用整个窗口宽度，少留空
    let (_card_w, card) = if bare {
        let w = (width - MARGIN).max(200.0);
        (w, Rect::new((width - w) / 2.0, 0.0, w, height))
    } else {
        let w = (width - MARGIN * 2.0).min(CARD_MAX_W);
        (w, Rect::new((width - w) / 2.0, MARGIN, w, (height - MARGIN * 2.0).max(140.0)))
    };
    if !bare {
        ui.fill(card, palette.surface, 18.0);
        ui.stroke(card, palette.border.with_alpha(0.7), 18.0, 1.0);
    }

    let nav = Rect::new(card.x, card.y, card.w, NAV_H);
    widgets::nav_bar(ui, &palette, nav, rt.view);

    let content = Rect::new(
        card.x + PAD,
        nav.bottom(),
        card.w - PAD * 2.0,
        (card.bottom() - nav.bottom() - PAD * 0.5).max(0.0),
    );
    let view = rt.view;
    let scroll = rt.scroll_of(view);
    let inner = Rect::new(content.x, content.y - scroll, content.w, content.h + scroll);

    ui.push_clip(content);
    // 设置界面的文字永远不隐藏（否则开关自身都看不见）
    let hide_all_text = ui.hide_all_text;
    if matches!(view, ViewId::Settings | ViewId::History) {
        ui.hide_all_text = false;
    }
    // 悬浮窗正在编辑活动名时，主窗口不重复画同一个编辑框（键盘焦点在悬浮窗那边）
    let held_edit = if rt.widget_editing { rt.edit.take() } else { None };
    let content_h = {
        let mut ctx = Ctx { app, rt: &mut *rt, palette, now, area: inner };
        match view {
            ViewId::Timer => timer::draw(ui, &mut ctx),
            ViewId::Pomodoro => pomodoro::draw(ui, &mut ctx),
            ViewId::History => history::draw(ui, &mut ctx),
            ViewId::Settings => settings::draw(ui, &mut ctx),
        }
    };
    if let Some(edit) = held_edit {
        rt.edit = Some(edit);
    }
    ui.pop_clip();

    rt.scroll_max.insert(view, (content_h - content.h).max(0.0));
    let scroll = rt.scroll_of(view);
    rt.set_scroll(view, scroll);
    let scroll = rt.scroll_of(view);
    widgets::scrollbar(
        ui,
        &palette,
        Rect::new(card.right() - 10.0, content.y + 4.0, 5.0, content.h - 8.0),
        scroll,
        content_h,
        content.h,
    );

    // 弹出层（下拉 / 日历 / 吐司）是要点着用的，永远显示文字
    ui.hide_all_text = false;
    draw_dropdown(ui, app, rt, &palette);
    draw_calendar(ui, app, rt, &palette);
    draw_toast(ui, rt, &palette, now);
    ui.hide_all_text = hide_all_text;
}

fn draw_dropdown(ui: &mut Ui, app: &App, rt: &mut Runtime, palette: &Palette) {
    let Some(id) = rt.open_drop else { return };
    let anchor = rt.drop_anchor;
    if !anchor.valid() {
        return;
    }
    let items = drops::items(app, id);
    if items.is_empty() {
        return;
    }
    let selected = drops::selected_with(app, rt, id);
    let item_h = 30.0;
    let max_items = 9.0;
    let visible = (items.len() as f32).min(max_items);
    let h = item_h * visible + 10.0;
    let w = anchor.w.max(170.0);
    let x = anchor.x.min((ui.width - w - 6.0).max(6.0));
    let y = (anchor.bottom() + 4.0).min((ui.height - h - 6.0).max(6.0));
    let list = Rect::new(x, y, w, h);
    ui.fill(list.offset(0.0, 2.0), crate::color::Color::rgba(0, 0, 0, 0.12), 10.0);
    ui.fill(list, palette.surface, 10.0);
    ui.stroke(list, palette.border, 10.0, 1.0);

    let max_scroll = (items.len() as f32 * item_h - (h - 10.0)).max(0.0);
    let scroll = rt.drop_scroll.clamp(0.0, max_scroll);
    rt.drop_scroll = scroll;

    ui.push_clip(list.inset(1.0, 5.0));
    for (index, item) in items.iter().enumerate() {
        let row_y = list.y + 5.0 + index as f32 * item_h - scroll;
        let row = Rect::new(list.x + 4.0, row_y, list.w - 8.0, item_h - 2.0);
        if row.bottom() < list.y || row.y > list.bottom() {
            continue;
        }
        let action = Action::DropPick(id, index);
        ui.hit(row, action.clone());
        let hover = ui.is_hover(action);
        if index == selected {
            ui.fill(row, palette.soft, 7.0);
        } else if hover {
            ui.fill(row, palette.soft.with_alpha(0.6), 7.0);
        }
        let color = if index == selected { palette.work } else { palette.text };
        let font = Font::new(13.0, if index == selected { 600 } else { 400 });
        ui.text(item, Rect::new(row.x + 10.0, row.y, row.w - 20.0, row.h), color, &font);
    }
    ui.pop_clip();
}

fn draw_calendar(ui: &mut Ui, app: &App, rt: &mut Runtime, palette: &Palette) {
    if !rt.calendar_open {
        return;
    }
    history::draw_calendar_popup(ui, app, rt, palette);
}

fn draw_toast(ui: &mut Ui, rt: &Runtime, palette: &Palette, now: u64) {
    let Some((text, until)) = &rt.toast else { return };
    if now >= *until {
        return;
    }
    let font = Font::new(13.0, 500);
    let width = ui.text_width(text, &font) + 34.0;
    let rect = Rect::new((ui.width - width) / 2.0, ui.height - 62.0, width, 38.0);
    let bg = if palette.dark { crate::color::Color::rgb(240, 240, 242) } else { crate::color::Color::rgb(38, 36, 34) };
    let fg = if palette.dark { crate::color::Color::rgb(30, 30, 32) } else { crate::color::Color::rgb(248, 248, 248) };
    ui.fill(rect, bg.with_alpha(0.96), 10.0);
    ui.text(text, rect, fg, &font.align(crate::ui::Align::Center));
}
