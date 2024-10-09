use std::any::Any;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::style::palette::tailwind;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Paragraph, Widget};
use tokio::sync::mpsc::UnboundedSender;
use crate::my_event::AppEvent;

#[derive(PartialEq)]
enum Status {
    Normal,
    FilterEdit,
}

pub trait FilterInnerWidget {
    fn set_filter(&mut self, filter: &str);
    fn get_menu() -> Vec<(&'static str, &'static str)>;
    async fn on_key(&mut self, key_event: KeyEvent);
    fn show(&mut self, area: Rect, buf: &mut Buffer);
    async fn active(&mut self);
    fn on_data(&mut self, data: Box<dyn Any>);
}

pub struct FilterWidget<T: FilterInnerWidget> {
    app_tx: UnboundedSender<AppEvent>,
    filter: String,
    status: Status,

    inner_widget: T,
}

impl<T: FilterInnerWidget> FilterWidget<T> {
    pub(crate) fn on_data(&mut self, data: Box<dyn Any>) {
        self.inner_widget.on_data(data)

    }
    pub async fn active(&mut self) {
        self.inner_widget.active().await;
    }

    pub fn get_menu() -> Vec<(&'static str, &'static str)> {
        T::get_menu()
    }

    pub fn new(app_tx: UnboundedSender<AppEvent>, inner: T) -> FilterWidget<T> {
        Self {
            app_tx,
            filter: Default::default(),
            status: Status::Normal,

            inner_widget: inner,
        }
    }

    pub fn get_menu_filter_edit() -> Vec<(&'static str, &'static str)> {
        vec![
            ("ENTER", "确认"),
            ("ESC", "放弃"),
        ]
    }

    pub async fn on_key(&mut self, key_event: KeyEvent) {
        match self.status {
            Status::Normal => {
                match key_event.code {
                    KeyCode::Char('/') => {
                        self.status = Status::FilterEdit;
                        self.inner_widget.set_filter("");
                        self.app_tx.send(AppEvent::SetMenu(Self::get_menu_filter_edit())).unwrap();
                    }
                    _ => self.inner_widget.on_key(key_event).await
                }
            },
            Status::FilterEdit => {
                match key_event.code {
                    KeyCode::Enter => {
                        self.status = Status::Normal;
                        self.inner_widget.set_filter(&self.filter);
                        self.app_tx.send(AppEvent::SetMenu(T::get_menu())).unwrap();
                    }
                    KeyCode::Backspace => {
                        self.filter.pop();
                        self.app_tx.send(AppEvent::Draw).unwrap();
                    }
                    KeyCode::Esc => {
                        self.status = Status::Normal;
                        self.filter.clear();
                        self.inner_widget.set_filter("");
                        self.app_tx.send(AppEvent::SetMenu(T::get_menu())).unwrap();
                    }
                    KeyCode::Char(c) => {
                        self.filter.push(c);
                        self.app_tx.send(AppEvent::Draw).unwrap();
                    }
                    _ => {},
                }
            }
        }
    }

    pub fn show(&mut self, area: Rect, buf: &mut Buffer) {
        if self.filter != "" || self.status == Status::FilterEdit {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
                .split(area);
            self.inner_widget.show(layout[0], buf);

            let fg = if self.status == Status::FilterEdit {
                tailwind::GREEN.c600
            } else {
                tailwind::BLACK
            };

            let line = Line::from(vec![Span::raw(format!("过滤: {}", self.filter.clone()))]);
            let p_for_msg = Paragraph::new(line)
                .alignment(Alignment::Left)
                .fg(fg)
                .block(Block::new().bg(tailwind::SLATE.c200));
            p_for_msg.render(layout[1], buf);
        } else {
            self.inner_widget.show(area, buf);
        }
    }
}
