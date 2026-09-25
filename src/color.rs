use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a }
    }
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn from_hex(text: &str) -> Color {
        let s = text.trim().trim_start_matches('#');
        let parse = |a: usize, b: usize| u8::from_str_radix(&s[a..b], 16).unwrap_or(0);
        if s.len() >= 6 {
            Color::rgb(parse(0, 2), parse(2, 4), parse(4, 6))
        } else {
            Color::rgb(0x88, 0x88, 0x88)
        }
    }

    pub fn to_hex(self) -> String {
        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", c(self.r), c(self.g), c(self.b))
    }

    pub fn with_alpha(self, a: f32) -> Color {
        Color { a, ..self }
    }

    pub fn mix(a: Color, b: Color, t: f32) -> Color {
        Color {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t,
            a: a.a + (b.a - a.a) * t,
        }
    }

    pub fn lighten(self, t: f32) -> Color {
        Color::mix(self, Color::WHITE, t)
    }

    pub fn darken(self, t: f32) -> Color {
        Color::mix(self, Color::rgb(0, 0, 0), t)
    }

    pub fn luminance(self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}

/// 主题配色（与网页版字段一致：work / shortBreak / longBreak / pause / bg / surface / window / text）
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThemeColors {
    pub name: String,
    pub work: String,
    pub short_break: String,
    pub long_break: String,
    pub pause: String,
    pub bg: String,
    pub surface: String,
    pub window: String,
    pub text: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        ThemeColors {
            name: String::new(),
            work: "#e53e3e".into(),
            short_break: "#38a169".into(),
            long_break: "#3182ce".into(),
            pause: "#718096".into(),
            bg: "#f5f5f7".into(),
            surface: "#ffffff".into(),
            window: "#fafafa".into(),
            text: "#1d1d1f".into(),
        }
    }
}

impl ThemeColors {
    pub fn custom_default(index: usize) -> Self {
        ThemeColors { name: format!("自定义{}", index + 1), ..Default::default() }
    }

    /// 内置三套配色，字段取自网页版 colorThemes 表
    pub fn builtin(key: &str, dark: bool) -> ThemeColors {
        let (name, work, short, long, pause, bg, surface, window, text) = match (key, dark) {
            ("A", false) => ("预设颜色1", "#e53e3e", "#38a169", "#3182ce", "#718096", "#f5f5f7", "#ffffff", "#fafafa", "#1d1d1f"),
            ("A", true) => ("预设颜色1", "#c53030", "#38a169", "#3182ce", "#4a5568", "#1a202c", "#232b3b", "#202938", "#f7fafc"),
            ("B", false) => ("预设颜色2", "#c5554e", "#589e72", "#5486b4", "#888e96", "#f5f5f7", "#ffffff", "#fafafa", "#1d1d1f"),
            ("B", true) => ("预设颜色2", "#9b443e", "#487c5c", "#446b8f", "#575c64", "#1a202c", "#232b3b", "#202938", "#f7fafc"),
            ("E", false) => ("预设颜色3", "#b85450", "#61a880", "#5c87b2", "#787878", "#f5f5f7", "#ffffff", "#fafafa", "#1d1d1f"),
            ("E", true) => ("预设颜色3", "#944340", "#4e8867", "#4a6e91", "#5c5c5c", "#1a202c", "#232b3b", "#202938", "#f7fafc"),
            _ => ("预设颜色1", "#e53e3e", "#38a169", "#3182ce", "#718096", "#f5f5f7", "#ffffff", "#fafafa", "#1d1d1f"),
        };
        ThemeColors {
            name: name.into(),
            work: work.into(),
            short_break: short.into(),
            long_break: long.into(),
            pause: pause.into(),
            bg: bg.into(),
            surface: surface.into(),
            window: window.into(),
            text: text.into(),
        }
    }

    /// 方案3–方案8（取自网页版 themeList 的浅色 / 深色两组值）。
    /// 按需求：窗口色与背景色相同。
    pub fn scheme(index: usize, dark: bool) -> ThemeColors {
        let (name, work, short, long, pause, bg, surface, text) = match (index, dark) {
            (3, false) => ("方案3｜浅青灰底", "#E16036", "#2EC4B6", "#1D4ED8", "#9AA3AD", "#D8E8EF", "#E2F0F6", "#123039"),
            (3, true) => ("方案3｜浅青灰底", "#FB923C", "#2DD4BF", "#3B82F6", "#87929E", "#1A252B", "#22333B", "#E8F2F6"),
            (4, false) => ("方案4｜奶油暖米色", "#E63946", "#7FB069", "#4A6FA5", "#A89F91", "#EAE3DA", "#F5EFE6", "#3A322B"),
            (4, true) => ("方案4｜奶油暖米色", "#FF6B6B", "#9CD37A", "#7FA6D6", "#8A847C", "#221F1D", "#2B2724", "#F3EDE6"),
            (5, false) => ("方案5｜杏仁暖米色", "#C14953", "#8FAE86", "#5D7A99", "#9E9688", "#E8DFD0", "#F3EBDD", "#3B332A"),
            (5, true) => ("方案5｜杏仁暖米色", "#FF847C", "#B6D2A7", "#8AA7C7", "#7F776D", "#201D1A", "#292521", "#F1EAE1"),
            (6, false) => ("方案6｜草木护眼绿", "#556B2F", "#7BA05B", "#6F8FAF", "#9E9587", "#DDE4D7", "#E8EDE2", "#2C3327"),
            (6, true) => ("方案6｜草木护眼绿", "#B6CD79", "#D0E6A5", "#A7BED3", "#877F74", "#1D231B", "#242B22", "#ECF2E6"),
            (7, false) => ("方案7｜原木暖棕", "#A64B3D", "#8FA478", "#6F9198", "#A19585", "#E4DBCD", "#EFE7DA", "#382F26"),
            (7, true) => ("方案7｜原木暖棕", "#E08A78", "#B8CBA2", "#A5C4CB", "#8A7F72", "#221D1A", "#2B2420", "#F0E8DF"),
            (8, false) => ("方案8｜雾蓝灰护眼", "#4F6358", "#6B9080", "#738CA6", "#9AA3AD", "#D9DEE4", "#E4E8EC", "#2A3138"),
            (8, true) => ("方案8｜雾蓝灰护眼", "#B6CDC0", "#A5D1BE", "#B6C7DB", "#87929E", "#181D23", "#20262D", "#E6ECF2"),
            _ => ("方案3｜浅青灰底", "#E16036", "#2EC4B6", "#1D4ED8", "#9AA3AD", "#D8E8EF", "#E2F0F6", "#123039"),
        };
        ThemeColors {
            name: name.into(),
            work: work.into(),
            short_break: short.into(),
            long_break: long.into(),
            pause: pause.into(),
            bg: bg.into(),
            surface: surface.into(),
            window: bg.into(),
            text: text.into(),
        }
    }

    pub fn set_field(&mut self, index: usize, value: String) {
        match index {
            0 => self.work = value,
            1 => self.short_break = value,
            2 => self.long_break = value,
            3 => self.pause = value,
            4 => self.bg = value,
            5 => self.surface = value,
            6 => self.window = value,
            _ => self.text = value,
        }
    }
}

/// 已解析的调色板
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub work: Color,
    pub short: Color,
    pub long: Color,
    pub pause: Color,
    pub bg: Color,
    pub surface: Color,
    pub window: Color,
    pub text: Color,
    pub muted: Color,
    pub faint: Color,
    pub border: Color,
    pub soft: Color,
    /// 页面底色：有卡片界面时用「背景」，无背景界面时用「窗口」
    pub page: Color,
    /// 卡片内的小块底色（导航条 / 统计卡 / 历史行 / 圆环轨道）：
    /// 有卡片界面时用「窗口」，无背景界面时用「卡片」
    pub inset: Color,
    pub dark: bool,
}

impl Palette {
    pub fn new(theme: &ThemeColors, dark: bool, bare: bool) -> Self {
        let text = Color::from_hex(&theme.text);
        let surface = Color::from_hex(&theme.surface);
        let bg = Color::from_hex(&theme.bg);
        let window = Color::from_hex(&theme.window);
        Palette {
            work: Color::from_hex(&theme.work),
            short: Color::from_hex(&theme.short_break),
            long: Color::from_hex(&theme.long_break),
            pause: Color::from_hex(&theme.pause),
            bg,
            surface,
            window,
            text,
            muted: Color::mix(text, surface, 0.45),
            faint: Color::mix(text, surface, 0.62),
            border: Color::mix(surface, text, 0.14),
            soft: Color::mix(surface, text, 0.05),
            page: if bare { window } else { bg },
            inset: if bare { surface } else { window },
            dark,
        }
    }
}
