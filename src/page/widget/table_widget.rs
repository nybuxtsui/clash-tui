use crate::g;
use crate::g::COLOR;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{palette::tailwind, Modifier, Style, Stylize as _}
    ,
    text::Text,
    widgets::{
        Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, StatefulWidget, Table, TableState,
    },
};

pub struct TableWidget {
    table_state: TableState,
    scroll_state: ScrollbarState,

    header: Vec<&'static str>,
    data: Vec<Vec<String>>,
}

impl TableWidget {
    pub fn new(header: Vec<&'static str>) -> Self {
        Self {
            table_state: Default::default(),
            scroll_state: Default::default(),
            header,
            data: Vec::new(),
        }
    }

    pub fn set_data(&mut self, data: Vec<Vec<String>>) {
        if data.eq(&self.data) {
            return;
        }

        self.data = data;

        let mut len = self.data.len();
        if len > 0 {
            len -= 1;
        }
        if self.table_state.selected().is_none() {
            self.table_state.select_first();
        }
        self.scroll_state = self.scroll_state.content_length(len);
    }

    fn update_scroll_state(&mut self) {
        if let Some(i) = self.table_state.selected() {
            self.scroll_state = self.scroll_state.position(i);
        };
    }

    pub fn select_up(&mut self) -> Vec<String> {
        if self.data.len() == 0 {
            return vec![];
        }
        self.table_state.select_previous();
        self.update_scroll_state();
        self.data
            .get(self.table_state.selected().unwrap())
            .unwrap()
            .clone()
    }

    pub fn select_down(&mut self) -> Vec<String> {
        if self.data.len() == 0 {
            return vec![];
        }
        self.table_state.select_next();
        self.update_scroll_state();
        let mut i = self.table_state.selected().unwrap();
        if i >= self.data.len() {
            i = self.data.len() - 1;
        }
        self.table_state.select(Some(i));
        self.data.get(i).unwrap().clone()
    }

    pub fn select<F: Fn(&Vec<String>) -> bool>(&mut self, f: F) {
        for (i, row) in self.data.iter().enumerate() {
            if f(row) {
                self.table_state.select(Some(i));
                self.update_scroll_state();
                return;
            }
        }
        self.table_state.select_first();
        self.update_scroll_state();
    }

    pub fn current_row(&self) -> Option<Vec<String>> {
        match self.table_state.selected() {
            None => None,
            Some(i) => self.data.get(i).map(|x| x.clone()),
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let header_style = Style::default()
            .fg(COLOR.header_fg)
            .bg(COLOR.header_bg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(COLOR.selected_style_fg);
        let header = self
            .header
            .iter()
            .map(|s| Cell::from(s as &str))
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let has_selected = if self.header.len() >= 3 {
            self.header[2] == "选中"
        } else { false };

        let rows = self.data.iter().enumerate().map(|(i, data)| {
            let bg_color = match i % 2 {
                0 => COLOR.normal_row_color,
                _ => COLOR.alt_row_color,
            };
            let fg_color = match has_selected && data[2] != "" {
                true => tailwind::GREEN.c500,
                false => COLOR.row_fg,
            };
            data.iter()
                .map(|str| Cell::from(Text::from(str as &str)))
                .collect::<Row>()
                .style(Style::new().fg(fg_color).bg(bg_color))
                .height(1u16)
        });

        // let block = Block::bordered()
        //     .title(Title::from(&self.title as &str).alignment(Alignment::Center))
        //     .border_set(border::DOUBLE);

        let mut widths = Vec::new();
        for (i, header) in self.header.iter().enumerate() {
            let width = self
                .data
                .iter()
                .map(|row| g::string_width(&row[i]))
                .max()
                .unwrap_or(0) as u16;
            let header_width = g::string_width(header) as u16;
            let width = if width > header_width {
                width
            } else {
                header_width
            } + 2;
            if i == header.len() - 1 {
                widths.push(Constraint::Fill(1));
            } else {
                widths.push(Constraint::Length(width));
            }
        }

        let t = Table::new(rows, widths)
            // .block(block)
            .header(header)
            .highlight_style(selected_style)
            .bg(COLOR.buffer_bg)
            .highlight_spacing(HighlightSpacing::Always);

        let mut table_area = area.clone();
        table_area.width -= 1;
        StatefulWidget::render(t, table_area, buf, &mut self.table_state);

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
}
