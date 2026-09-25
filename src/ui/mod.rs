use std::collections::HashMap;

use windows::core::{Interface, PCWSTR};
use windows_numerics::Vector2;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

pub mod icons;
pub mod widgets;

use crate::action::Action;
use crate::color::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn mid_y(&self) -> f32 {
        self.y + self.h / 2.0
    }

    pub fn mid_x(&self) -> f32 {
        self.x + self.w / 2.0
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.right() && y >= self.y && y <= self.bottom()
    }

    pub fn inset(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, (self.w - dx * 2.0).max(0.0), (self.h - dy * 2.0).max(0.0))
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }

    pub fn intersect(&self, other: &Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let r = self.right().min(other.right());
        let b = self.bottom().min(other.bottom());
        Rect::new(x, y, (r - x).max(0.0), (b - y).max(0.0))
    }

    pub fn valid(&self) -> bool {
        self.w > 0.5 && self.h > 0.5
    }

    pub fn d2d(&self) -> D2D_RECT_F {
        D2D_RECT_F { left: self.x, top: self.y, right: self.right(), bottom: self.bottom() }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VAlign {
    Top,
    #[default]
    Center,
    Bottom,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    pub size: f32,
    pub weight: u32,
    pub align: Align,
    pub valign: VAlign,
    pub family: Option<String>,
    pub essential: bool,
}

impl Font {
    pub fn new(size: f32, weight: u32) -> Self {
        Font { size, weight, align: Align::Left, valign: VAlign::Center, family: None, essential: false }
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn center(mut self) -> Self {
        self.align = Align::Center;
        self
    }

    pub fn right(mut self) -> Self {
        self.align = Align::Right;
        self
    }

    pub fn vtop(mut self) -> Self {
        self.valign = VAlign::Top;
        self
    }

    pub fn family(mut self, family: &str) -> Self {
        self.family = Some(family.to_string());
        self
    }

    pub fn essential(mut self) -> Self {
        self.essential = true;
        self
    }
}

pub struct Hit {
    pub rect: Rect,
    pub action: Action,
}

pub struct Ui {
    pub factory: ID2D1Factory,
    pub dwrite: IDWriteFactory,
    rt: Option<ID2D1HwndRenderTarget>,
    dc_rt: Option<ID2D1DCRenderTarget>,
    dc_size: (u32, u32),
    dc_dpi: f32,
    brushes: HashMap<u32, ID2D1SolidColorBrush>,
    formats: HashMap<String, IDWriteTextFormat>,
    pub hits: Vec<Hit>,
    pub prev: Vec<Hit>,
    pub hover: Option<Action>,
    pub active: Option<Action>,
    pub mouse: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    clips: Vec<Rect>,
    pub ui_family: String,
    pub hide_all_text: bool,
    pub ui_scale: f32,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

impl Ui {
    pub fn new() -> windows::core::Result<Self> {
        let factory: ID2D1Factory = unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };
        let dwrite: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        Ok(Ui {
            factory,
            dwrite,
            rt: None,
            dc_rt: None,
            dc_size: (0, 0),
            dc_dpi: 0.0,
            brushes: HashMap::new(),
            formats: HashMap::new(),
            hits: Vec::new(),
            prev: Vec::new(),
            hover: None,
            active: None,
            mouse: (-1.0, -1.0),
            width: 0.0,
            height: 0.0,
            scale: 1.0,
            clips: Vec::new(),
            ui_family: "Microsoft YaHei UI".into(),
            hide_all_text: false,
            ui_scale: 1.0,
        })
    }

    /// 复制一份绘图环境用于第二个窗口（悬浮窗），共用 D2D / DirectWrite 工厂以省内存
    pub fn fork(&self) -> Ui {
        Ui {
            factory: self.factory.clone(),
            dwrite: self.dwrite.clone(),
            rt: None,
            dc_rt: None,
            dc_size: (0, 0),
            dc_dpi: 0.0,
            brushes: HashMap::new(),
            formats: HashMap::new(),
            hits: Vec::new(),
            prev: Vec::new(),
            hover: None,
            active: None,
            mouse: (-1.0, -1.0),
            width: 0.0,
            height: 0.0,
            scale: self.scale,
            clips: Vec::new(),
            ui_family: self.ui_family.clone(),
            hide_all_text: self.hide_all_text,
            ui_scale: self.ui_scale,
        }
    }

    /// 当前渲染目标（窗口优先，其次离屏 DC）。取克隆是为了不和 &mut self 借用打架。
    fn target(&self) -> Option<ID2D1RenderTarget> {
        if let Some(rt) = &self.rt {
            return rt.cast::<ID2D1RenderTarget>().ok();
        }
        if let Some(rt) = &self.dc_rt {
            return rt.cast::<ID2D1RenderTarget>().ok();
        }
        None
    }

    pub fn has_target(&self) -> bool {
        self.rt.is_some() || self.dc_rt.is_some()
    }

    /// 离屏 DC 渲染目标（悬浮窗）：D2D 直接画进 32bpp 预乘 alpha 的位图。
    /// 透明底不能用 ClearType，只能灰度抗锯齿，否则边缘会出彩边。
    pub fn ensure_dc_target(&mut self, dc: HDC, width: u32, height: u32, dpi: f32) -> windows::core::Result<()> {
        let scale = dpi / 96.0;
        self.scale = scale;
        self.width = width as f32 / scale;
        self.height = height as f32 / scale;
        if width == 0 || height == 0 {
            return Ok(());
        }
        if self.dc_rt.is_some() && self.dc_size == (width, height) && self.dc_dpi == dpi {
            return Ok(());
        }
        self.rt = None;
        let props = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: dpi,
            dpiY: dpi,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };
        let dc_rt = unsafe { self.factory.CreateDCRenderTarget(&props)? };
        let rect = RECT { left: 0, top: 0, right: width as i32, bottom: height as i32 };
        unsafe {
            dc_rt.SetDpi(dpi, dpi);
            dc_rt.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            dc_rt.BindDC(dc, &rect)?;
        }
        self.dc_rt = Some(dc_rt);
        self.dc_size = (width, height);
        self.dc_dpi = dpi;
        self.brushes.clear();
        Ok(())
    }

    pub fn ensure_target(&mut self, hwnd: HWND, width: u32, height: u32, dpi: f32) -> windows::core::Result<()> {
        let scale = dpi / 96.0;
        self.scale = scale;
        self.width = width as f32 / scale;
        self.height = height as f32 / scale;
        if width == 0 || height == 0 {
            return Ok(());
        }
        self.dc_rt = None;
        if self.rt.is_none() {
            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_IGNORE,
                },
                dpiX: dpi,
                dpiY: dpi,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D_SIZE_U { width, height },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };
            let rt = unsafe { self.factory.CreateHwndRenderTarget(&props, &hwnd_props)? };
            unsafe {
                rt.SetDpi(dpi, dpi);
            }
            self.rt = Some(rt);
        } else {
            if let Some(rt) = &self.rt {
                unsafe {
                    let size = D2D_SIZE_U { width, height };
                    let _ = rt.Resize(&size);
                    rt.SetDpi(dpi, dpi);
                }
            }
            self.brushes.clear();
        }
        Ok(())
    }

    pub fn clear(&mut self, color: Color) {
        let Some(rt) = self.target() else { return };
        let c = D2D1_COLOR_F { r: color.r, g: color.g, b: color.b, a: color.a };
        unsafe {
            let _ = rt.Clear(Some(&c));
        }
    }

    pub fn begin(&mut self, clear: Color) {
        self.hits.clear();
        if let Some(rt) = self.target() {
            unsafe {
                let _ = rt.BeginDraw();
            }
            self.clear(clear);
        }
    }

    pub fn end(&mut self) {
        if let Some(rt) = self.target() {
            let result = unsafe { rt.EndDraw(None, None) };
            if result.is_err() {
                self.rt = None;
                self.dc_rt = None;
                self.brushes.clear();
                self.formats.clear();
            }
        }
    }

    pub fn commit_hits(&mut self) {
        self.prev = std::mem::take(&mut self.hits);
        let mouse = self.mouse;
        self.hover = self
            .prev
            .iter()
            .rev()
            .find(|hit| hit.rect.contains(mouse.0, mouse.1))
            .map(|hit| hit.action.clone());
    }

    /// 上一帧命中列表中最上层的元素（用于点击）
    pub fn hit_action(&self) -> Option<(Action, Rect)> {
        let mouse = self.mouse;
        self.prev
            .iter()
            .rev()
            .find(|hit| hit.rect.contains(mouse.0, mouse.1))
            .map(|hit| (hit.action.clone(), hit.rect))
    }

    pub fn is_hover(&self, action: Action) -> bool {
        self.hover == Some(action)
    }

    pub fn hit(&mut self, rect: Rect, action: Action) {
        if !rect.valid() {
            return;
        }
        let visible = match self.clips.last() {
            Some(clip) => clip.intersect(&rect).valid(),
            None => true,
        };
        if visible || self.clips.is_empty() {
            self.hits.push(Hit { rect, action });
        }
    }

    /// 命中区域内是否悬停（用于样式的局部 hover）
    pub fn hover_rect(&self, rect: Rect) -> bool {
        rect.contains(self.mouse.0, self.mouse.1)
    }

    fn brush(&mut self, color: Color) -> Option<ID2D1SolidColorBrush> {
        let key = ((color.r.clamp(0.0, 1.0) * 255.0) as u32) << 24
            | ((color.g.clamp(0.0, 1.0) * 255.0) as u32) << 16
            | ((color.b.clamp(0.0, 1.0) * 255.0) as u32) << 8
            | ((color.a.clamp(0.0, 1.0) * 255.0) as u32);
        if let Some(brush) = self.brushes.get(&key) {
            return Some(brush.clone());
        }
        let rt = self.target()?;
        let c = D2D1_COLOR_F { r: color.r, g: color.g, b: color.b, a: color.a };
        let brush = unsafe { rt.CreateSolidColorBrush(&c, None).ok()? };
        self.brushes.insert(key, brush.clone());
        Some(brush)
    }

    pub fn fill(&mut self, rect: Rect, color: Color, radius: f32) {
        if !rect.valid() || color.a <= 0.001 {
            return;
        }
        let Some(brush) = self.brush(color) else { return };
        let Some(rt) = self.target() else { return };
        unsafe {
            if radius > 0.5 {
                let rr = D2D1_ROUNDED_RECT {
                    rect: rect.d2d(),
                    radiusX: radius,
                    radiusY: radius,
                };
                let _ = rt.FillRoundedRectangle(&rr, &brush);
            } else {
                let r = rect.d2d();
                let _ = rt.FillRectangle(&r, &brush);
            }
        }
    }

    pub fn stroke(&mut self, rect: Rect, color: Color, radius: f32, width: f32) {
        if !rect.valid() {
            return;
        }
        let Some(brush) = self.brush(color) else { return };
        let Some(rt) = self.target() else { return };
        unsafe {
            if radius > 0.5 {
                let rr = D2D1_ROUNDED_RECT { rect: rect.d2d(), radiusX: radius, radiusY: radius };
                let _ = rt.DrawRoundedRectangle(&rr, &brush, width, None);
            } else {
                let r = rect.d2d();
                let _ = rt.DrawRectangle(&r, &brush, width, None);
            }
        }
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, width: f32) {
        let Some(brush) = self.brush(color) else { return };
        let Some(rt) = self.target() else { return };
        unsafe {
            let _ = rt.DrawLine(Vector2 { X: x1, Y: y1 }, Vector2 { X: x2, Y: y2 }, &brush, width, None);
        }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, color: Color) {
        let Some(brush) = self.brush(color) else { return };
        let Some(rt) = self.target() else { return };
        let e = D2D1_ELLIPSE { point: Vector2 { X: cx, Y: cy }, radiusX: r, radiusY: r };
        unsafe {
            let _ = rt.FillEllipse(&e, &brush);
        }
    }

    /// 圆环进度：从 12 点钟方向顺时针
    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, thickness: f32, color: Color, fraction: f32) {
        let Some(brush) = self.brush(color) else { return };
        let Some(rt) = self.target() else { return };
        let fraction = fraction.clamp(0.0, 1.0);
        let steps = ((360.0 * fraction).ceil() as i32).max(1);
        let step = std::f32::consts::TAU / 360.0;
        for i in 0..steps {
            let a0 = i as f32 * step - std::f32::consts::FRAC_PI_2;
            let a1 = (i + 1) as f32 * step - std::f32::consts::FRAC_PI_2;
            unsafe {
                let _ = rt.DrawLine(
                    Vector2 { X: cx + r * a0.cos(), Y: cy + r * a0.sin() },
                    Vector2 { X: cx + r * a1.cos(), Y: cy + r * a1.sin() },
                    &brush,
                    thickness,
                    None,
                );
            }
        }
    }

    pub fn push_clip(&mut self, rect: Rect) {
        let clipped = match self.clips.last() {
            Some(outer) => rect.intersect(outer),
            None => rect,
        };
        self.clips.push(clipped);
        if let Some(rt) = self.target() {
            let r = clipped.d2d();
            unsafe {
                let _ = rt.PushAxisAlignedClip(&r, D2D1_ANTIALIAS_MODE_ALIASED);
            }
        }
    }

    pub fn pop_clip(&mut self) {
        self.clips.pop();
        if let Some(rt) = self.target() {
            unsafe {
                rt.PopAxisAlignedClip();
            }
        }
    }

    fn format(&mut self, font: &Font) -> Option<IDWriteTextFormat> {
        let family = font.family.clone().unwrap_or_else(|| self.ui_family.clone());
        let key = format!("{}|{}|{}|{:?}|{:?}", family, (font.size * 4.0) as u32, font.weight, font.align, font.valign);
        if let Some(fmt) = self.formats.get(&key) {
            return Some(fmt.clone());
        }
        let name = wide(&family);
        let locale = wide("zh-CN");
        let fmt = unsafe {
            let f = self
                .dwrite
                .CreateTextFormat(
                    PCWSTR(name.as_ptr()),
                    None,
                    DWRITE_FONT_WEIGHT(font.weight as i32),
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    font.size,
                    PCWSTR(locale.as_ptr()),
                )
                .ok()?;
            let _ = f.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP);
            let _ = f.SetTextAlignment(match font.align {
                Align::Left => DWRITE_TEXT_ALIGNMENT_LEADING,
                Align::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
                Align::Right => DWRITE_TEXT_ALIGNMENT_TRAILING,
            });
            let _ = f.SetParagraphAlignment(match font.valign {
                VAlign::Top => DWRITE_PARAGRAPH_ALIGNMENT_NEAR,
                VAlign::Center => DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
                VAlign::Bottom => DWRITE_PARAGRAPH_ALIGNMENT_FAR,
            });
            f
        };
        self.formats.insert(key, fmt.clone());
        Some(fmt)
    }

    pub fn text(&mut self, text: &str, rect: Rect, color: Color, font: &Font) {
        if text.is_empty() || !rect.valid() {
            return;
        }
        if self.hide_all_text && !font.essential {
            return;
        }
        let Some(fmt) = self.format(font) else { return };
        let Some(brush) = self.brush(color) else { return };
        let Some(rt) = self.target() else { return };
        let data: Vec<u16> = text.encode_utf16().collect();
        let r = rect.d2d();
        unsafe {
            let _ = rt.DrawText(&data, &fmt, &r, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }
    }

    pub fn measure(&mut self, text: &str, font: &Font) -> (f32, f32) {
        let Some(fmt) = self.format(font) else { return (0.0, font.size) };
        let data: Vec<u16> = text.encode_utf16().collect();
        let layout = unsafe { self.dwrite.CreateTextLayout(&data, &fmt, 4000.0, 400.0) };
        match layout {
            Ok(layout) => {
                let mut metrics = DWRITE_TEXT_METRICS::default();
                let ok = unsafe { layout.GetMetrics(&mut metrics) };
                if ok.is_ok() {
                    (metrics.widthIncludingTrailingWhitespace, metrics.height)
                } else {
                    (0.0, font.size)
                }
            }
            Err(_) => (0.0, font.size),
        }
    }

    /// 文本宽度（不含换行），用于布局
    pub fn text_width(&mut self, text: &str, font: &Font) -> f32 {
        self.measure(text, font).0
    }
}

pub fn grid(rect: Rect, columns: usize, gap: f32) -> Vec<Rect> {
    let total_gap = gap * (columns.saturating_sub(1)) as f32;
    let cell = (rect.w - total_gap) / columns as f32;
    (0..columns)
        .map(|i| Rect::new(rect.x + (cell + gap) * i as f32, rect.y, cell, rect.h))
        .collect()
}


