use crate::clash_api::{get_config, ConnectionItem};
use crate::clash_api::Connection;
use crate::my_event::AppEvent;
use crate::page::widget::TableWidget;
use crate::page::{start_ws_worker, WsMsg};
use chrono::{DateTime, Local, TimeDelta, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use humansize::{format_size, BINARY};
use indexmap::IndexMap;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tokio::sync::mpsc::{channel, Sender, UnboundedSender};

pub struct ConnectionPage {
    pub table_widget: TableWidget,
    pub app_tx: UnboundedSender<AppEvent>,
    pub close_tx: Option<Sender<bool>>,
    pub last_data: IndexMap<String, ConnectionItem>,
    pub last_upload: u32,
    pub last_download: u32,
}

impl ConnectionPage {
    pub fn new(app_tx: UnboundedSender<AppEvent>) -> Self {
        Self {
            table_widget: TableWidget::new(vec![
                "源主机",
                "主机",
                "链路",
                "规则",
                "下载速度",
                "上传速度",
                "下载量",
                "上传量",
                "类型",
                "连接时间",
            ]),
            app_tx,
            close_tx: None,
            last_data: IndexMap::new(),
            last_upload: 0,
            last_download: 0,
        }
    }

    pub async fn on_key(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Up => {
                self.table_widget.select_up();
                self.app_tx.send(AppEvent::Draw).unwrap();
                true
            }
            KeyCode::Down => {
                self.table_widget.select_down();
                self.app_tx.send(AppEvent::Draw).unwrap();
                true
            }
            KeyCode::Char('p') => {
                self.app_tx.send(AppEvent::ShowGroupPage).unwrap();
                self.inactive().await;
                true
            }
            KeyCode::Char('l') => {
                self.app_tx.send(AppEvent::ShowLogPage).unwrap();
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

    pub fn on_data(&mut self, connection: Connection) {
        let mut status = String::new();
        let mut dlspeed = 0;
        let mut upspeed = 0;
        if self.last_download > 0 && self.last_upload > 0 {
            dlspeed = connection.download_total - self.last_download;
            upspeed = connection.download_total - self.last_download;
        }
        status.push_str(&format!(
            "下载速度:{}|上传速度:{}|总下载:{}|总上传:{}",
            format_size(dlspeed, BINARY),
            format_size(upspeed, BINARY),
            format_size(connection.download_total, BINARY),
            format_size(connection.upload_total, BINARY)
        ));
        self.last_download = connection.download_total;
        self.last_upload = connection.upload_total;

        self.app_tx.send(AppEvent::Status(status)).unwrap();
        fn format_duration(duration: TimeDelta) -> String {
            let seconds = duration.num_seconds();
            if seconds < 10 {
                return "几秒前".to_string();
            }
            let minutes = seconds / 60;
            let hours = minutes / 60;
            let days = hours / 24;
            let remaining_seconds = seconds % 60;
            let remaining_minutes = minutes % 60;
            let remaining_hours = hours % 24;

            let mut result = String::new();
            let mut v = vec![Some(days), Some(remaining_hours), Some(remaining_minutes), Some(remaining_seconds)];
            for i in &mut v {
                if let Some(n) = i {
                    if *n == 0 {
                        *i = None
                    } else {
                        break;
                    }
                }
            }
            let mut count = 0;
            for i in &mut v {
                if let Some(_) = i {
                    count += 1;
                    if count > 2 {
                        *i = None
                    }
                }
            }
            for i in &mut v.iter_mut().rev() {
                if let Some(n) = i {
                    if *n == 0 {
                        *i = None
                    } else {
                        break;
                    }
                }
            }
            for i in 0..v.len() {
                if let Some(n) = v[i] {
                    result.push_str(&format!("{n}{}", ["天", "小时", "分", "秒"][i]));
                }
            }
            result
        }

        fn get_not_empty<'a>(v: &[&'a str]) -> &'a str {
            for i in v {
                if !i.eq(&"") {
                    return i;
                }
            }
            return "";
        }
        let now: DateTime<Local> = Local::now();
        let mut data = Vec::new();
        for i in &connection.connections {
            let t = match i.start.parse::<DateTime<Utc>>() {
                Ok(utc_datetime) => {
                    let local_datetime: DateTime<Local> = utc_datetime.with_timezone(&Local);
                    let duration = now.signed_duration_since(local_datetime);
                    let f = format_duration(duration);
                    f.to_string()
                },
                Err(_) => {
                    "".into()
                }
            };
            let last_connection = self.last_data.get(&i.id);
            let dlspeed = if let Some(last_connection) = last_connection {
                format_size(i.download - last_connection.download, BINARY)
            } else {
                format_size(0usize, BINARY)
            };
            let upspeed = if let Some(last_connection) = last_connection {
                format_size(i.upload - last_connection.upload, BINARY)
            } else {
                format_size(0usize, BINARY)
            };

            data.push(vec![
                i.metadata.source_ip.clone(),
                format!("{}:{}", get_not_empty(&vec![i.metadata.sniff_host.as_str(), i.metadata.host.as_str(), i.metadata.destination_ip.as_str()]), i.metadata.destination_port),
                i.chains.last().unwrap_or(&"".to_string()).clone(),
                if i.rule_payload==""{i.rule.clone()} else {i.rule_payload.clone()},
                dlspeed,
                upspeed,
                format_size(i.download, BINARY),
                format_size(i.upload, BINARY),
                format!("{}({})", i.metadata.inbound_name, i.metadata.network),
                t,

            ])
        }
        self.last_data.clear();
        for v in connection.connections {
            self.last_data.insert(v.id.clone(), v);
        }
        self.table_widget.set_data(data);
    }

    pub fn get_menu() -> Vec<(&'static str, &'static str)> {
        vec![
            ("P", "代理"),
            ("L", "日志"),
            ("Q", "退出"),
        ]
    }

    pub async fn active(&mut self) {
        let config = get_config();
        let url = format!("ws://{}/connections", config.host);
        let params = [("token", &config.key)];
        let url = reqwest::Url::parse_with_params(&url, &params).unwrap();

        let (tx, rx) = channel::<bool>(1);
        // 重新赋值，则旧的sender会drop，这样在receiver也会关闭，那么async{}就会退出循环，并且结束
        self.close_tx = Some(tx);
        let app_tx = self.app_tx.clone();
        start_ws_worker(url, rx, move |wsmsg| {
            match wsmsg {
                WsMsg::ConnectFail(s) => {
                    app_tx.send(AppEvent::Status(format!("连接失败，重试: {s}"))).unwrap();
                },
                WsMsg::Closed => {
                    app_tx.send(AppEvent::Status("连接已断开，重连".into())).unwrap();
                }
                WsMsg::Message(msg) => {
                    let connection = serde_json::from_str::<Connection>(msg.to_text().unwrap()).unwrap();
                    app_tx.send(AppEvent::Connection(connection)).unwrap();
                }
            }
        });
    }

    pub async fn inactive(&mut self) {
        self.app_tx.send(AppEvent::Status("就绪".to_owned())).unwrap();
        self.close_tx = None;
        self.table_widget.set_data(vec![]);
    }

    pub fn show(&mut self, area: Rect, buf: &mut Buffer) {
        self.table_widget.render(area, buf);
    }
}
