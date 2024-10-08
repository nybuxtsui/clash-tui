mod group_item_page;
mod group_page;
mod log_page;
mod widget;
mod connection_page;

use std::time::Duration;

use futures_util::StreamExt as _;
pub use group_item_page::GroupItemPage;
pub use group_page::GroupPage;
pub use log_page::LogPage;
pub use connection_page::ConnectionPage;
use tokio::{select, sync::mpsc::Receiver};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;


pub enum WsMsg {
    ConnectFail(String),
    Closed,
    Message(Message),
}

pub fn start_ws_worker<F>(url: Url, mut rx: Receiver<bool>, f: F)
    where
        F: Fn(WsMsg)  + Send + Sync + 'static {
    tokio::spawn(async move {
        'o: loop {
            let mut ws_stream = loop {
                select! {
                    ws = connect_async(url.to_string()) => {
                        match ws {
                            Ok(ws) => {
                                break ws.0;
                            },
                            Err(e) => {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                f(WsMsg::ConnectFail(e.to_string()));
                                continue
                            }
                        }
                    }
                    _ = rx.recv() => {
                        break 'o;
                    }
                }
            };
            loop {
                select! {
                    msg = ws_stream.next() => {
                        match msg {
                            Some(Ok(msg)) => {
                                f(WsMsg::Message(msg));
                            },
                            Some(Err(e)) => {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                f(WsMsg::ConnectFail(e.to_string()));
                                break;
                            },
                            None => {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                f(WsMsg::Closed);
                                break;
                            }
                        }
                    },
                    _ = rx.recv() => {
                        ws_stream.close(None).await.unwrap();
                        break 'o;
                    }
                }
            }


        }
    });

}