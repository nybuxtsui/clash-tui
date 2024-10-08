use crate::clash_api::{get_config, LogItem};
use crate::my_event::AppEvent;
use crate::page::widget::LogWidget;
use crate::page::{start_ws_worker, WsMsg};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tokio::sync::mpsc::{channel, Sender, UnboundedSender};

pub struct LogPage {
    log_widget: LogWidget,
    pub app_tx: UnboundedSender<AppEvent>,
    pub close_tx: Option<Sender<bool>>,
}

impl LogPage {
    pub fn new(app_tx: UnboundedSender<AppEvent>) -> Self {
        Self {
            log_widget: LogWidget::new(5000),
            app_tx,
            close_tx: None,
        }
    }

    pub async fn on_key(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Up => {
                self.log_widget.select_up();
                self.app_tx.send(AppEvent::Draw).unwrap();
                true
            }
            KeyCode::Down => {
                self.log_widget.select_down();
                self.app_tx.send(AppEvent::Draw).unwrap();
                true
            }
            KeyCode::Char('p') => {
                self.app_tx.send(AppEvent::ShowGroupPage).unwrap();
                self.inactive().await;
                true
            }
            KeyCode::Char('c') => {
                self.app_tx.send(AppEvent::ShowConnection).unwrap();
                self.inactive().await;
                true
            }
            KeyCode::Esc => {
                self.app_tx.send(AppEvent::Quit).unwrap();
                true
            }
            _ => false,
        }
    }
    pub fn on_log(&mut self, log: LogItem) {
        self.log_widget.add_line(log.payload);
    }

    pub fn get_menu() -> Vec<(&'static str, &'static str)> {
        vec![
            ("P", "代理"),
            ("C", "链接"),
            ("Q", "退出"),
        ]
    }

    pub async fn active(&mut self) {
        let config = get_config();
        let url = format!("ws://{}/logs", config.host);
        let params = [("level", "info"), ("token", &config.key)];
        let url = reqwest::Url::parse_with_params(&url, &params).unwrap();

        let (tx, rx) = channel::<bool>(1);
        // 重新赋值，则旧的sender会drop，这样在receiver也会关闭，那么async{}就会退出循环，并且结束
        self.close_tx = Some(tx);
        let app_tx = self.app_tx.clone();

        start_ws_worker(url, rx, move |wsmsg| {
            match wsmsg {
                WsMsg::ConnectFail(s) => {
                    app_tx.send(AppEvent::Log(LogItem{r#type: "".into(), payload: format!("连接失败，重试: {s}")})).unwrap();
                },
                WsMsg::Closed => {
                    app_tx.send(AppEvent::Log(LogItem{r#type: "".into(), payload: "连接已断开，重连".into()})).unwrap();
                }
                WsMsg::Message(msg) => {
                    let log = serde_json::from_str::<LogItem>(msg.to_text().unwrap()).unwrap();
                    app_tx.send(AppEvent::Log(log)).unwrap();
                }
            }
        });
    }

    pub async fn inactive(&mut self) {
        self.close_tx = None;
        self.log_widget.clear();
    }

    pub fn show(&mut self, area: Rect, buf: &mut Buffer) {
        self.log_widget.render(area, buf);
    }
}
