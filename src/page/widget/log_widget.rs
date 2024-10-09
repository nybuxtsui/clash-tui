use crate::g::COLOR;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, StatefulWidget, Style, Text};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget, Wrap};
use std::borrow::Cow;

pub struct LogWidget {
    // 最大缓存行数
    max: usize,
    // 需要显示的内容
    lines: Vec<String>,

    // 滚动条信息
    scroll_state: ScrollbarState,
    scroll_pos: Option<usize>,

    cached_width: usize,
    cached_lines: Vec<String>,
    // 过滤字符串
    filter: String,
}

impl LogWidget {
    pub fn new(max: usize) -> Self {
        Self {
            max,
            scroll_state: Default::default(),
            lines: Vec::new(),
            scroll_pos: None,

            cached_width: usize::MAX,
            cached_lines: Vec::new(),
            filter: String::new(),
        }
    }

    pub fn set_filter(&mut self, filter: &str) {
        if self.filter != filter {
            self.filter = filter.to_string();
            self.cached_width = usize::MAX;
            self.cached_lines.clear();
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.lines.push(line.clone());
        if self.lines.len() > self.max {
            let c = self.lines.len() - self.max;
            self.lines.drain(0..c);

            self.cached_lines.clear();
            self.cached_width = usize::MAX;
        } else if self.cached_width != usize::MAX {
            let width = self.cached_width - 2;

            self.cached_lines.extend(
                textwrap::wrap(&line, width)
                    .into_iter()
                    .map(Cow::into_owned)
            );

        }
        self.scroll_pos = None;
    }

    pub fn clear(&mut self) {
        self.cached_width = usize::MAX;
        self.cached_lines.clear();
        self.lines.clear();
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let width = (area.width - 2) as usize;
        let height = (area.height - 2) as usize;

        if width != self.cached_width {
            self.cached_width = width;
            self.cached_lines = self.lines.iter()
                .filter(|l| {
                    if self.filter.is_empty() {
                        return true;
                    }
                    l.contains(&self.filter)
                })
                .flat_map(|x| {textwrap::wrap(x, width)})
                .map(Cow::into_owned)
                .collect::<Vec<String>>();
        }


        let total_line = self.cached_lines.len();
        let scroll_max = if total_line > height { total_line - height } else { 0 };

        let mut i = self.scroll_pos.unwrap_or(scroll_max);
        if i > scroll_max {
            i = scroll_max;
        }
        self.scroll_state = self.scroll_state.content_length(scroll_max).position(i);
        self.scroll_pos = Some(i);

        let lines: Vec<Line> = self.cached_lines
            .iter()
            .skip(self.scroll_pos.unwrap())
            .map(String::as_str)
            .map(Line::from)
            .collect();

        let p = Paragraph::new(Text::from(lines))
            .style(Style::default().fg(COLOR.row_fg))
            .wrap(Wrap{ trim: false })
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double).bg(COLOR.buffer_bg));

        p.render(area, buf);

        StatefulWidget::render(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            buf,
            &mut self.scroll_state,
        );

    }

    pub fn select_up(&mut self) {
        self.scroll_state.prev();
        self.scroll_pos = match self.scroll_pos {
            Some(i) => {
                if i > 0 {
                    Some(i - 1)
                } else {
                    Some(0)
                }
            },
            None => None,
        };
    }

    pub fn select_down(&mut self) {
        self.scroll_state.next();
        self.scroll_pos = match self.scroll_pos {
            Some(i) => {
                Some(i + 1)
            },
            None => None,
        };
    }

}