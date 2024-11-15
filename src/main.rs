mod clash_api;
mod clash_config;
mod g;
mod my_event;
mod page;
mod app_config;

use crate::clash_api::ProxyData;
use crate::my_event::AppEvent;
use crate::page::{ConnectionPage, GroupItemPage, GroupPage, LogPage};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::palette::tailwind;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::DefaultTerminal;
use std::io;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::app_config::load_config;
use crate::page::widget::filter_widget::FilterWidget;

#[derive(PartialEq)]
enum CurrentPage {
    Group,
    GroupItem,
    Log,
    Connection,
}

pub struct App {
    current_page: CurrentPage,
    app_tx: UnboundedSender<AppEvent>,
    app_rx: UnboundedReceiver<AppEvent>,
    proxy_data: Option<ProxyData>,
    status: String,

    group_page: GroupPage,
    group_item_page: GroupItemPage,
    log_page: FilterWidget<LogPage>,
    connection_page: FilterWidget<ConnectionPage>,

    menu: Vec<(&'static str, &'static str)>,
}

impl App {
    fn new() -> Self {
        let (app_tx, app_rx) = tokio::sync::mpsc::unbounded_channel();
        
        Self {
            current_page: CurrentPage::Group,
            proxy_data: Default::default(),
            status: "就绪".into(),

            group_page: GroupPage::new(app_tx.clone()),
            group_item_page: GroupItemPage::new(app_tx.clone()),
             log_page: FilterWidget::new(app_tx.clone(), LogPage::new(app_tx.clone())),
            connection_page: FilterWidget::new(app_tx.clone(), ConnectionPage::new(app_tx.clone())),

            app_tx,
            app_rx,

            menu: vec![],
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.group_page.active().await;
        self.menu = self.group_page.get_menu();
        let mut terminal = ratatui::init();
        self.draw(&mut terminal)?;
        loop {
            match self.app_rx.recv().await.unwrap() {
                AppEvent::Quit => {
                    return Ok(());
                }
                AppEvent::ProxyLoaded(proxy) => {
                    self.proxy_data = Some(proxy.clone());
                    match self.current_page {
                        CurrentPage::Group => self.group_page.on_proxy_loaded(proxy.clone()),
                        CurrentPage::GroupItem => self.group_item_page.on_proxy_loaded(proxy.clone()),
                        _ => (),
                    }
                    self.draw(&mut terminal)?;
                }
                AppEvent::Key(key_event) => {
                    match self.current_page {
                        CurrentPage::Group => self.group_page.on_key(key_event).await,
                        CurrentPage::GroupItem => self.group_item_page.on_key(key_event).await,
                        CurrentPage::Log => self.log_page.on_key(key_event).await,
                        CurrentPage::Connection => self.connection_page.on_key(key_event).await,
                    };
                },
                AppEvent::Draw => {
                    self.draw(&mut terminal)?;
                }
                AppEvent::Status(msg) => {
                    self.status = msg;
                    self.menu = match self.current_page {
                        CurrentPage::Group => self.group_page.get_menu(),
                        CurrentPage::GroupItem => self.group_page.get_menu(),
                        CurrentPage::Log => self.log_page.get_menu(),
                        CurrentPage::Connection => self.connection_page.get_menu(),
                    };
                    self.draw(&mut terminal)?
                }
                AppEvent::ShowGroupItemPage(name) => {
                    self.current_page = CurrentPage::GroupItem;
                    self.group_item_page.set_group_name(&name);
                    self.group_item_page
                        .on_proxy_loaded(self.proxy_data.clone().unwrap());
                    self.group_item_page.select_selected();
                    self.menu = self.group_item_page.get_menu();
                    self.draw(&mut terminal)?;
                }
                AppEvent::ShowGroupPage => {
                    self.current_page = CurrentPage::Group;
                    self.menu = self.group_page.get_menu();
                    self.draw(&mut terminal)?;
                }
                AppEvent::ShowLogPage => {
                    self.current_page = CurrentPage::Log;
                    self.log_page.active().await;
                    self.menu = self.log_page.get_menu();
                    self.draw(&mut terminal)?
                }
                AppEvent::ShowConnection => {
                    self.current_page = CurrentPage::Connection;
                    self.connection_page.active().await;
                    self.menu = self.connection_page.get_menu();
                    self.draw(&mut terminal)?
                }
                AppEvent::Log(log) => {
                    self.log_page.on_data(Box::new(log));
                    self.draw(&mut terminal)?;
                }
                AppEvent::Connection(connection) => {
                    self.connection_page.on_data(Box::new(connection));
                    self.draw(&mut terminal)?;
                }
                AppEvent::SetMenu(menu) => {
                    self.menu = menu;
                    self.draw(&mut terminal)?;
                }
                AppEvent::ModeChanged(mode) => {
                    self.group_page.set_current_mode(&mode);
                    if self.current_page == CurrentPage::Group {
                        self.menu = self.group_page.get_menu();
                    }
                    self.draw(&mut terminal)?;
                },
            }
        }
    }

    fn draw(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| {
            let area = frame.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
                .split(area);
            match self.current_page {
                CurrentPage::Group => self.group_page.show(layout[0], frame.buffer_mut()),
                CurrentPage::GroupItem => self.group_item_page.show(layout[0], frame.buffer_mut()),
                CurrentPage::Log => self.log_page.show(layout[0], frame.buffer_mut()),
                CurrentPage::Connection => self.connection_page.show(layout[0], frame.buffer_mut()),
            }

            let line = Line::from(vec![Span::raw(self.status.clone())]);

            let p_for_msg = Paragraph::new(line)
                .alignment(Alignment::Left)
                .fg(tailwind::BLACK)
                .block(Block::new().bg(tailwind::SLATE.c200));

            let mut line = vec![];
            let count = g::string_width(&self.status);
            for &(key, text) in &self.menu {
                if !line.is_empty() {
                    line.push("   ".into())
                }
                line.extend(vec![
                    Span::styled(key, Style::default().fg(tailwind::RED.c600)),
                    " ".into(),
                    Span::styled(text, Style::default().fg(tailwind::BLACK)),
                ]);
            }
            let p_for_menu = Paragraph::new(Line::from(line))
                .alignment(Alignment::Right)
                .block(Block::new().bg(tailwind::SLATE.c200));

            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Length(count as u16), Constraint::Fill(1)])
                .split(layout[1]);
            frame.render_widget(p_for_msg, layout[0]);
            frame.render_widget(p_for_menu, layout[1]);
        })?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    {
        let mut config = crate::app_config::CONFIG.write().unwrap();
        *config = load_config().expect("加载配置文件出错");
    }
    clash_api::get_mode().await.expect("连接后端出错，请确认配置是否正确");

    let mut app = App::new();

    let tx = app.app_tx.clone();
    tokio::spawn(async move {
        loop {
            let e = event::read().expect("failed to read crossterm::event::read");
            match e {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        if (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
                            && key_event.modifiers == KeyModifiers::CONTROL
                        {
                            tx.send(AppEvent::Quit).unwrap_or(());
                            break;
                        }
                        if tx.send(AppEvent::Key(key_event)).is_err() {
                            break;
                        }
                    }
                }
                Event::Resize(_, _) => {
                    if tx.send(AppEvent::Draw).is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    let app_result = app.run().await;
    ratatui::restore();
    app_result
}
