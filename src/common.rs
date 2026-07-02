use crate::{
    AppData,
    AppWindow,
    GNoti,
    Noti,
};
use regex::{Regex, RegexBuilder};
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

pub fn class_to_id(name: &str, data: &mut AppData) -> u64 {
    name_to_id(name, data, NameType::Class)
}

pub fn state_to_id(name: &str, data: &mut AppData) -> u64 {
    name_to_id(name, data, NameType::State)
}

pub fn event_to_id(name: &str, data: &mut AppData) -> u64 {
    name_to_id(name, data, NameType::Event)
}

pub fn id_to_class(id: u64, data: &AppData) -> Option<String> {
    id_to_name(id, data, NameType::Class)
}

pub fn id_to_state(id: u64, data: &AppData) -> Option<String> {
    id_to_name(id, data, NameType::State)
}

pub fn id_to_event(id: u64, data: &AppData) -> Option<String> {
    id_to_name(id, data, NameType::Event)
}

pub fn name_to_id(name: &str, data: &mut AppData, name_type: NameType) -> u64 {
    let data = match name_type {
        NameType::Class => &mut data.class_name_map,
        NameType::State => &mut data.state_name_map,
        NameType::Event => &mut data.event_name_map,
    };

    if let Some(id) = data.get(name) {
        *id
    } else {
        // TODO: Check duplicated id
        let lower = name.to_lowercase();
        let id = xxh3_64(lower.as_bytes());
        data.insert(name.to_string(), id);
        id
    }
}

pub fn id_to_name(id: u64, data: &AppData, name_type: NameType) -> Option<String> {
    let data = match name_type {
        NameType::Class => &data.class_name_map,
        NameType::State => &data.state_name_map,
        NameType::Event => &data.event_name_map,
    };

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