use crate::clash_api::Connection;
use crate::clash_api::ConnectionItem;
use crate::my_event::AppEvent;
use crate::page::widget::{FilterInnerWidget, TableWidget};
use crate::page::{start_ws_worker, WsMsg};
use chrono::{DateTime, Local, TimeDelta, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use humansize::{format_size, BINARY};
use indexmap::IndexMap;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::Any;
use tokio::sync::mpsc::{channel, Sender, UnboundedSender};
use crate::app_config::get_config;

pub struct ConnectionPage {
    table_widget: TableWidget,
    app_tx: UnboundedSender<AppEvent>,
    close_tx: Option<Sender<bool>>,
    // 最后一次的数据，用于计算和本次数据的差值，比如上传下载速度
    last_data: IndexMap<String, ConnectionItem>,
    last_upload: u32,
    last_download: u32,
    pause: bool,
}

impl ConnectionPage {
    pub fn new(app_tx: UnboundedSender<AppEvent>) -> Self {
        Self {
            table_widget: TableWidget::new(vec![
                "源主机",
                "主机",
                "链路",
                "下载速度",
                "上传速度",
                "下载量",
                "上传量",
                "类型",
                "连接时间",
                "规则",
            ]),
            app_tx,
            close_tx: None,
            last_data: IndexMap::new(),
            last_upload: 0,
            last_download: 0,
            pause: false,
        }
    }
    pub async fn inactive(&mut self) {
        self.app_tx.send(AppEvent::Status("就绪".to_owned())).unwrap();
        self.close_tx = None;
        self.table_widget.set_data(vec![]);
    }

}

impl FilterInnerWidget for ConnectionPage {
    fn set_filter(&mut self, filter: &str) {
        self.table_widget.set_filter(filter);
    }

    fn get_menu() -> Vec<(&'static str, &'static str)> {
        vec![
            ("<Space>", "暂停"),
            ("/", "搜索"),
            ("P", "代理"),
            ("L", "日志"),
            ("Q", "退出"),
        ]
    }

    async fn on_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => {
                self.table_widget.select_up();
                self.app_tx.send(AppEvent::Draw).unwrap();
            }
            KeyCode::Down => {
                self.table_widget.select_down();
                self.app_tx.send(AppEvent::Draw).unwrap();
            }
            KeyCode::Char('p') => {
                self.app_tx.send(AppEvent::ShowGroupPage).unwrap();
                self.inactive().await;
            }
            KeyCode::Char('l') => {
                self.app_tx.send(AppEvent::ShowLogPage).unwrap();
                self.inactive().await;
            }
            KeyCode::Char(' ') => {
                self.pause = !self.pause;
                self.app_tx.send(AppEvent::Status(if self.pause {"暂停"} else {"恢复"}.to_owned())).unwrap();
            }
            KeyCode::Esc => {
                self.app_tx.send(AppEvent::Quit).unwrap();
            }
            _ => {},
        }
    }

    fn show(&mut self, area: Rect, buf: &mut Buffer) {
        self.table_widget.render(area, buf);
    }

    async fn active(&mut self) {
        self.pause = false;
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
                    let str = msg.to_text().unwrap();
                    match serde_json::from_str::<Connection>(str) {
                        Ok(connection) => {
                            app_tx.send(AppEvent::Connection(connection)).unwrap();
                        }
                        Err(e) => {
                            app_tx.send(AppEvent::Status(format!("处理连接数据出错: {e}"))).unwrap();
                        }
                    }
                }
            }
        });
    }

    fn on_data(&mut self, data: Box<dyn Any>) {
        if self.pause {
            return;
        }
        if let Ok(connection) = data.downcast::<Connection>() {
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
                        match *n {
                            0 => *i = None,
                            _ => break,
                        };
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
                        match *n {
                            0 => *i = None,
                            _ => break,
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
                    if !i.is_empty() {
                        return i;
                    }
                }
                ""
            }
            let now: DateTime<Local> = Local::now();
            let mut data = Vec::new();
            for conn in &connection.connections {
                let dt_start = match conn.start.parse::<DateTime<Utc>>() {
                    Ok(utc_datetime) => {
                        let local_datetime: DateTime<Local> = utc_datetime.with_timezone(&Local);
                        let duration = now.signed_duration_since(local_datetime);
                        format_duration(duration)
                    },
                    Err(e) => {
                        self.app_tx.send(AppEvent::Status(format!("格式化时间错误: {} {e}", conn.start))).unwrap();
                        String::new()
                    },
                };
                let last_connection = self.last_data.get(&conn.id);
                let dlspeed = match last_connection {
                    Some(last_connection) => { format_size(conn.download - last_connection.download, BINARY) }
                    None => {format_size(0usize, BINARY)}
                };
                let upspeed = match last_connection {
                    Some(last_connection) => { format_size(conn.upload - last_connection.upload, BINARY) }
                    None => {format_size(0usize, BINARY)}
                };

                data.push(vec![
                    conn.metadata.source_ip.clone(),
                    format!("{}:{}", get_not_empty(&vec![conn.metadata.sniff_host.as_str(), conn.metadata.host.as_str(), conn.metadata.destination_ip.as_str()]), conn.metadata.destination_port),
                    conn.chains.last().unwrap_or(&"".to_string()).clone(),
                    dlspeed,
                    upspeed,
                    format_size(conn.download, BINARY),
                    format_size(conn.upload, BINARY),
                    format!("{}({})", conn.metadata.inbound_name, conn.metadata.network),
                    dt_start,
                    if conn.rule_payload == "" { conn.rule.clone() } else { conn.rule_payload.clone() },
                ])
            }
            self.last_data.clear();
            for v in connection.connections {
                self.last_data.insert(v.id.clone(), v);
            }
            self.table_widget.set_data(data);
        }
    }

}
