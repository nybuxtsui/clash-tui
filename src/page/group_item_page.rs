use crate::clash_api;
use crate::clash_api::ProxyData;
use crate::my_event::AppEvent;
use crate::my_event::AppEvent::{ProxyLoaded, ShowGroupPage, Status};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;
use crate::page::widget::TableWidget;

pub struct GroupItemPage {
    table_widget: TableWidget,
    group_name: String,
    app_tx: UnboundedSender<AppEvent>,
}

impl GroupItemPage {
    pub fn new(app_tx: UnboundedSender<AppEvent>) -> Self {
        let mut table_widget = TableWidget::new(vec!["名称", "延迟", "选中"]);
        table_widget.set_data(vec![]);
        Self {
            table_widget,
            app_tx,
            group_name: String::default(),
        }
    }

    pub fn get_menu() -> Vec<(&'static str, &'static str)> {
        vec![
            ("CTRL-T", "测速"),
            ("ENTER", "选择"),
            ("ESC", "返回"),
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
            KeyCode::Char('t') | KeyCode::Char('T') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    let group_name = self.group_name.clone();
                    let tx = self.app_tx.clone();
                    tokio::spawn(async move {
                        tx.send(Status("测速中...".into())).unwrap();
                        match clash_api::check_delay(&group_name).await {
                            Ok(_) => {}
                            Err(e) => {tx.send(Status(format!("检查延时出错: {e}"))).unwrap();}
                        }
                        let proxy = clash_api::load_proxy().await;
                        match proxy {
                            Ok(proxy) => {
                                tx.send(ProxyLoaded(proxy)).unwrap();
                                tx.send(Status("测速完成".into())).unwrap();
                            },
                            Err(e) => {
                                tx.send(Status(format!("加载数据出错: {e}"))).unwrap();
                            }
                        }
                    });
                }
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.app_tx.send(AppEvent::ShowLogPage).unwrap();
            }
            KeyCode::Enter => {
                if let Some(row) = self.table_widget.current_row() {
                    if row[2].is_empty() {
                        self.app_tx
                            .send(ShowGroupPage)
                            .unwrap();
                        let new_group = row[0].clone();

                        let group_name = self.group_name.clone();
                        let tx = self.app_tx.clone();
                        tokio::spawn(async move {
                            if let Err(err) = clash_api::select_group_current(&group_name, &new_group).await {
                                tx.send(Status(format!("设置分组出错: {err}"))).unwrap();
                            }
                            let proxy = clash_api::load_proxy().await;
                            match proxy {
                                Ok(proxy) => {
                                    tx.send(ProxyLoaded(proxy)).unwrap();
                                },
                                Err(e) => {
                                    tx.send(Status(format!("加载数据出错: {e}"))).unwrap();
                                }
                            }
                        });
                    } else {
                        self.app_tx.send(ShowGroupPage).unwrap();
                    }
                }
            }
            KeyCode::Esc => {
                if self.table_widget.current_row().is_some() {
                    self.app_tx.send(ShowGroupPage).unwrap();
                }
            }
            _ => {},
        }
    }

    pub fn set_group_name(&mut self, group_name: &str) {
        self.group_name = group_name.into();
    }

    pub fn on_proxy_loaded(&mut self, proxy: ProxyData) {
        self.table_widget
            .set_data(proxy.to_group_items(&self.group_name))
    }

    pub fn select_up(&mut self) {
        self.table_widget.select_up();
    }
    pub fn select_down(&mut self) {
        self.table_widget.select_down();
    }

    pub fn select_selected(&mut self) {
        self.table_widget.select(|x| !x[2].is_empty())
    }

    pub fn show(&mut self, area: Rect, buffer: &mut Buffer) {
        self.table_widget.render(area, buffer)
    }
}
