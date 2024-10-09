use crate::clash_api;
use crate::clash_api::ProxyData;
use crate::my_event::AppEvent;
use crate::my_event::AppEvent::{ProxyLoaded, ShowGroupItemPage, Status};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;
use crate::page::widget::TableWidget;

pub struct GroupPage {
    table_widget: TableWidget,
    selected: String,
    app_tx: UnboundedSender<AppEvent>,
}


impl GroupPage {
    pub fn new(app_tx: UnboundedSender<AppEvent>) -> Self {
        let mut table_widget = TableWidget::new(vec!["名称", "详情", "延迟"]);
        table_widget.set_data(vec![]);
        Self {
            table_widget,
            app_tx,

            selected: String::new(),
        }
    }

    pub fn on_proxy_loaded(&mut self, proxy: ProxyData) {
        self.table_widget.set_data(proxy.to_groups());
        self.table_widget.select(|x| x[0].eq(&self.selected));
    }

    pub fn get_menu() -> Vec<(&'static str, &'static str)> {
        vec![
            ("L", "日志"),
            ("C", "链接"),
            ("ENTER", "查看"),
            ("Q", "退出"),
        ]
    }
    pub async fn on_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => {
                self.select_up();
                self.app_tx.send(AppEvent::Draw).unwrap();
            }
            KeyCode::Down => {
                self.select_down();
                self.app_tx.send(AppEvent::Draw).unwrap();
            }
            KeyCode::Char('l') => {
                self.app_tx.send(AppEvent::ShowLogPage).unwrap();
            }
            KeyCode::Char('c') => {
                self.app_tx.send(AppEvent::ShowConnection).unwrap();
            }
            KeyCode::Esc => {
                self.app_tx.send(AppEvent::Quit).unwrap();
            }
            KeyCode::Enter => {
                if let Some(row) = self.table_widget.current_row() {
                    self.app_tx.send(ShowGroupItemPage(row[0].clone())).unwrap();
                }
            }
            _ => {},
        }
    }
    pub fn select_up(&mut self) {
        let row = self.table_widget.select_up();
        if !row.is_empty() {
            self.selected = row[0].clone();
        }
    }
    pub fn select_down(&mut self) {
        let row = self.table_widget.select_down();
        if !row.is_empty() {
            self.selected = row[0].clone();
        }
    }

    pub fn show(&mut self, area: Rect, buffer: &mut Buffer) {
        self.table_widget.render(area, buffer)
    }

    pub async fn active(&mut self) {
        let app_tx = self.app_tx.clone();
        tokio::spawn(async move {
            let proxy = clash_api::load_proxy().await;
            match proxy {
                Ok(proxy) => {
                    app_tx.send(ProxyLoaded(proxy)).unwrap();
                },
                Err(e) => {
                    app_tx.send(Status(format!("加载数据出错: {e}"))).unwrap();
                }
            }
        });
    }
}
