// 全部使用「文本呈现」字形（Unicode Emoji_Presentation = No），
// 避免 DirectWrite 回退到 Segoe UI Emoji 时画出彩色方框。

pub const PLAY: &str = "\u{25B6}"; // ▶ 开始 / 继续
pub const PAUSE: &str = "\u{275A}\u{275A}"; // ❚❚ 两个竖杠，不带方框
pub const SKIP: &str = "\u{21E5}"; // ⇥ 跳过
pub const STOP: &str = "\u{25A0}"; // ■ 结束
pub const RESET: &str = "\u{21BA}"; // ↺ 重置
pub const REFRESH: &str = "\u{21BB}"; // ↻ 刷新
pub const SOUND: &str = "\u{266A}"; // ♪ 提示音
pub const EDIT: &str = "\u{270E}"; // ✎ 改写
pub const CLOSE: &str = "\u{2715}"; // ✕ 删除 / 取消
pub const CHECK: &str = "\u{2713}"; // ✓
pub const PLUS: &str = "\u{FF0B}"; // ＋
pub const PIN: &str = "\u{2691}"; // ⚑ 固定位置（钉住拖不动）
pub const THROUGH: &str = "\u{25CC}"; // ◌ 点击穿透
pub const EXPAND: &str = "\u{2197}"; // ↗ 打开主界面
pub const MINIMIZE: &str = "\u{2013}"; // – 待机 / 收起按钮
pub const CARET_DOWN: &str = "\u{25BC}"; // ▼
pub const CARET_UP: &str = "\u{25B2}"; // ▲

pub const GEAR: &str = "\u{2699}"; // ⚙ 设置
pub const WATCH: &str = "\u{25F7}"; // ◷ 计时
pub const TARGET: &str = "\u{25CE}"; // ◎ 专注
pub const LEAF: &str = "\u{273F}"; // ✿ 短休息
pub const MOON: &str = "\u{263E}"; // ☾ 长休息
pub const LIST: &str = "\u{2630}"; // ☰ 历史
pub const PLAN: &str = "\u{25A4}"; // ▤ 计时方案
pub const TAG: &str = "\u{25C8}"; // ◈ 事件管理
pub const BRUSH: &str = "\u{25D1}"; // ◑ 视觉设置
pub const CALENDAR: &str = "\u{25A6}"; // ▦ 日期
pub const WIDGET: &str = "\u{25A2}"; // ▢ 桌面小图标
pub const SAVE: &str = "\u{25A3}"; // ▣ 保存方案
