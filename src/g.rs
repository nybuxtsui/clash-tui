use ratatui::prelude::Color;
use ratatui::style::palette::tailwind;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub buffer_bg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub row_fg: Color,
    pub selected_style_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
}

pub static COLOR: LazyLock<ThemeColors> = LazyLock::new(|| {
    ThemeColors {
        buffer_bg: tailwind::SLATE.c950,
        header_bg: tailwind::BLUE.c900,
        header_fg: tailwind::SLATE.c200,
        row_fg: tailwind::SLATE.c200,
        selected_style_fg: tailwind::BLUE.c400,
        normal_row_color: tailwind::SLATE.c950,
        alt_row_color: tailwind::SLATE.c900,
    }
});


pub fn string_width(s: &str) -> u32 {
    let mut len = 0;
    for c in s.chars() {
        len += match c as u32 {
            0..=0xff => 1,
            _ => 2,
        };
    }
    len
}
