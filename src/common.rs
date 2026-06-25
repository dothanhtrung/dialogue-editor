use crate::{
    AppWindow,
    GNoti,
    Noti,
};
use slint::ComponentHandle;
use tracing::{
    error,
    info,
    warn,
};

#[repr(i32)]
pub enum NotiLevel {
    Error = 0,
    Warn,
    Info,
}

pub enum NameType {
    Class,
    State,
    Event,
}

pub fn show_noti(ui: &AppWindow, level: NotiLevel, message: &str) {
    match level {
        NotiLevel::Error => error!(message),
        NotiLevel::Warn => warn!(message),
        NotiLevel::Info => info!(message),
    }

    ui.global::<GNoti>().set_noti(Noti {
        level: level as i32,
        message: message.into(),
    });
    ui.invoke_show_notification();
}
