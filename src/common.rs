use std::collections::BTreeMap;

use crate::{
    AppWindow,
    GNoti,
    Noti,
};
use regex::{
    Regex,
    RegexBuilder,
};
use slint::ComponentHandle;
use tracing::{
    error,
    info,
    warn,
};
use xxhash_rust::xxh3::xxh3_64;

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
    Sequence,
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

pub fn name_to_id(name: &str, data: &mut BTreeMap<String, u64>) -> u64 {
    if let Some(id) = data.get(name) {
        *id
    } else {
        let lower = name.to_lowercase();
        let id = xxh3_64(lower.as_bytes());
        // TODO: History
        data.insert(name.to_string(), id);
        id
    }
}

pub fn id_to_name(id: u64, data: &BTreeMap<String, u64>) -> Option<String> {
    for (name, i) in data.iter() {
        if id == *i {
            return Some(name.clone());
        }
    }

    None
}

pub fn new_regex(search: &str) -> Result<Regex, regex::Error> {
    RegexBuilder::new(search).case_insensitive(true).build()
}
