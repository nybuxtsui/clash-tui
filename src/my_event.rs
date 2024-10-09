use crate::clash_api::{LogItem, ProxyData};
use crossterm::event::KeyEvent;
use crate::clash_api::Connection;

pub enum AppEvent {
    Quit,
    Draw,
    ProxyLoaded(ProxyData),
    Key(KeyEvent),
    SetMenu(Vec<(&'static str, &'static str)>),

    ShowGroupPage,
    ShowGroupItemPage(String),
    ShowLogPage,
    ShowConnection,

    Status(String),
    Log(LogItem),
    Connection(Connection),
}
