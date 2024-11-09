use crate::clash_api;
use crate::clash_api::ProxyData;
use crate::my_event::AppEvent;
use crate::my_event::AppEvent::{ProxyLoaded, ShowGroupItemPage, Status};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;
use crate::page::widget::TableWidget;

const MODE_RULE: &str = "模式:RULE";
const MODE_GLOBAL: &str = "模式:GLOBAL";
const MODE_DIRECT: &str = "模式:DIRECT";

pub struct GroupPage {
    current_mode: &'static str,
    table_widget: TableWidget,
    selected: String,
    app_tx: UnboundedSender<AppEvent>,
}

impl GroupPage {
    pub fn new(app_tx: UnboundedSender<AppEvent>) -> Self {
        let mut table_widget = TableWidget::new(vec!["名称", "详情", "延迟"]);
        table_widget.set_data(vec![]);
        Self {
            current_mode: MODE_RULE,
            table_widget,
            app_tx,

            selected: String::new(),
        }
    }

    pub fn on_proxy_loaded(&mut self, proxy: ProxyData) {
        self.table_widget.set_data(proxy.to_groups());
        self.table_widget.select(|x| x[0].eq(&self.selected));
    }

    pub fn set_current_mode(&mut self, mode: &str) {
        self.current_mode = match mode.to_lowercase().as_str() {
            "direct" => MODE_DIRECT,
            "global" => MODE_GLOBAL,
            "rule" => MODE_RULE,
            _ => MODE_RULE,
        }
    }

    pub fn get_menu(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("CTRL-M", self.current_mode),
            ("L", "日志"),
            ("C", "链接"),
            ("ENTER", "查看"),
            ("ESC", "退出"),
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
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.app_tx.send(AppEvent::ShowLogPage).unwrap();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
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
            KeyCode::Char('m') | KeyCode::Char('M')  => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    let new_mode = match self.current_mode {
                        MODE_DIRECT => "rule",
                        MODE_RULE => "global",
                        MODE_GLOBAL => "direct",
                        _ => "rule",
                    };
                    clash_api::set_mode(new_mode).await;
                    let app_tx = self.app_tx.clone();
                    match clash_api::get_mode().await {
                        Ok(mode) => {
                            app_tx.send(AppEvent::ModeChanged(mode)).unwrap();
                        },
                        Err(e) => {
                            app_tx.send(Status(format!("加载数据出错: {e}"))).unwrap();
                        },
                    };
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
            match clash_api::get_mode().await {
                Ok(mode) => {
                    app_tx.send(AppEvent::ModeChanged(mode)).unwrap();
                }
                Err(e) => {
                    app_tx.send(Status(format!("加载数据出错: {e}"))).unwrap();
                },
            };
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
