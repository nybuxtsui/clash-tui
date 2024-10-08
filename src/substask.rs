use std::{borrow::Cow, sync::{Arc, Mutex}};

use eframe::egui;

use crate::config::{self, Connection, Subscription};

pub struct SubsTask {
    pub result: Arc<Mutex<Result>>,
    pub status: Arc<Mutex<Status>>,
}

#[derive(Clone, PartialEq)]
pub enum Status {
    Ready,
    Working,
    Finish,
}

#[derive(Clone)]
pub struct Result {
    pub msg: Option<Cow<'static, str>>,
    pub subscription: Subscription,
    pub connections: Vec<Connection>,
}

impl SubsTask {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(Status::Ready)),
            result: Arc::new(Mutex::new(Result{
                msg: None,
                subscription: Subscription::default(),
                connections: Vec::new(),
            })),
        }
    }

    pub fn run(&mut self, name: &str, url: &str, ctx: egui::Context) {
        let name = name.to_string();
        let url = url.to_string();
        {
            let mut status = self.status.lock().unwrap();
            let mut result = self.result.lock().unwrap();
            if *status != Status::Ready {
                *status = Status::Finish;
                result.msg = Some(Cow::from("状态不正确: 不是READY"));
                return;
            }
            *status = Status::Working;
        }
        let result = self.result.clone();
        let status = self.status.clone();
        tokio::spawn(async move {
            let v = config::subs::subscription(&name, &url).await;
            {
                let mut status = status.lock().unwrap();
                let mut result = result.lock().unwrap();
                match v {
                    Ok(v) => {
                        result.msg = None;
                        result.subscription = v.0;
                        result.connections = v.1;
                    },
                    Err(e) => {
                        result.msg = Some(Cow::from(e.to_string()));
                    }
                }
                *status = Status::Finish;
            }
            ctx.request_repaint();
        });
    }
    
    pub fn get_result(&self) -> Option<Result> {
        let mut status = self.status.lock().unwrap();
        let result = self.result.lock().unwrap();
        match *status {
            Status::Finish => {
                *status = Status::Ready;
                Some(result.clone())
            }
            _ => None,
        }
    }
}
