pub mod class_ui;
pub mod dialogue_ui;
pub mod state_ui;

use crate::{
    Affect,
    AppData,
    AppWindow,
    Config,
    ContentLang,
    Noti,
    NotiContent,
    NotiLevel,
    UiDialogue,
};
use regex_lite::Regex;
use slint::{
    ComponentHandle,
    SharedString,
};
use tracing::{
    error,
    info,
    warn,
};
use xxhash_rust::xxh3::xxh3_64;

pub fn reload_all(data: &mut AppData, ui: &AppWindow, config: &Config, search_class: &str, search_state: &str) {
    reload_class(data, ui, search_class);
    reload_state(data, ui, config, search_state);
    reload_dialogue(data, ui, config);
    reload_dialogue_detail(data, ui, config);
}

/// Reload class section and clear state/dialogue section
pub fn reload_class(data: &mut AppData, ui: &AppWindow, search_class: &str) {
    let mut classes: Vec<SharedString> = Vec::new(); // For content tab

    let re = Regex::new(search_class);
    for id in data.dialogues.keys() {
        let name = id_to_class(*id, data);
        let name = match name {
            Some(ret) => ret,
            None => {
                let id_string = id.to_string();
                data.class_name_map.insert(id_string.clone(), *id);
                id_string
            }
        };

        if search_class.is_empty() {
            classes.push(name.into());
        } else if let Ok(re) = re.as_ref()
            && re.is_match(&name)
        {
            classes.push(name.into());
        }
    }

    ui.set_classes(classes.as_slice().into());
    ui.set_states([].into());
    ui.set_dialogues([].into());
    ui.set_dialogue(UiDialogue::default());
}

/// Reload state section and clear dialogue section
pub fn reload_state(data: &mut AppData, ui: &AppWindow, config: &Config, search_state: &str) {
    let re = Regex::new(search_state);

    if let Some(class) = data.dialogues.get(&config.selected_class) {
        let mut states: Vec<SharedString> = Vec::new();
        for state_id in class.keys() {
            let state_name = match id_to_state(*state_id, data) {
                Some(ret) => ret,
                None => {
                    let id_string = state_id.to_string();
                    data.state_name_map.insert(id_string.clone(), *state_id);
                    id_string
                }
            };

            if search_state.is_empty() {
                states.push(state_name.into());
            } else if let Ok(re) = re.as_ref()
                && re.is_match(state_name.as_str())
            {
                states.push(state_name.into());
            }
        }
        ui.set_states(states.as_slice().into());
    }
    ui.set_dialogues([].into());
    ui.set_dialogue(UiDialogue::default());
}

pub fn reload_dialogue(data: &AppData, ui: &AppWindow, config: &Config) {
    if let Some(state) = data.dialogues.get(&config.selected_class)
        && let Some(state_dialogs) = state.get(&config.selected_state)
    {
        let mut dialogues: Vec<SharedString> = Vec::new();
        for dialog in state_dialogs {
            if let Some((_, content)) = dialog.contents.first_key_value() {
                dialogues.push(content.into());
            } else {
                dialogues.push(SharedString::new());
            }
        }

        ui.set_dialogues(dialogues.as_slice().into());
        ui.set_dialogue(UiDialogue::default());
    }
}

pub fn reload_dialogue_detail(data: &AppData, ui: &AppWindow, config: &Config) {
    if let Some(state) = data.dialogues.get(&config.selected_class)
        && let Some(dialog_list) = state.get(&config.selected_state)
        && let Some(dialog) = dialog_list.get(config.selected_dialog)
    {
        let mut contents: Vec<ContentLang> = Vec::new();
        let mut affects: Vec<Affect> = Vec::new();
        let mut events: Vec<SharedString> = Vec::new();
        for (lang, content) in dialog.contents.iter() {
            contents.push(ContentLang {
                language: lang.to_639_3().to_string().into(),
                content: content.into(),
            });
        }
        for (class, state) in dialog.affects.iter() {
            let class_name = id_to_class(*class, data).unwrap_or(class.to_string());
            let state_name = id_to_state(*state, data).unwrap_or(state.to_string());
            affects.push(Affect {
                class: class_name.into(),
                state: state_name.into(),
            });
        }
        for event in dialog.events.iter() {
            let event = id_to_event(*event, data).unwrap_or(event.to_string());
            events.push(event.into());
        }

        let ui_dialogue = UiDialogue {
            contents: contents.as_slice().into(),
            affects: affects.as_slice().into(),
            events: events.as_slice().into(),
        };
        ui.set_dialogue(ui_dialogue);

        let mut lang_list: Vec<SharedString> = Vec::new();
        for lang in config.langs.iter() {
            lang_list.push(lang.to_639_3().to_string().into());
        }
        ui.set_lang_list(lang_list.as_slice().into());

        let mut state_list: Vec<SharedString> = Vec::new();
        for (state, _) in data.state_name_map.iter() {
            state_list.push(state.into());
        }
        ui.set_state_list(state_list.as_slice().into());
    }
}

pub fn show_noti(ui: &AppWindow, level: NotiLevel, message: &str) {
    match level {
        NotiLevel::Error => error!(message),
        NotiLevel::Warn => warn!(message),
        NotiLevel::Info => info!(message),
    }

    ui.global::<NotiContent>().set_noti(Noti {
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

enum NameType {
    Class,
    State,
    Event,
}

fn name_to_id(name: &str, data: &mut AppData, name_type: NameType) -> u64 {
    let data = match name_type {
        NameType::Class => &mut data.class_name_map,
        NameType::State => &mut data.state_name_map,
        NameType::Event => &mut data.event_name_map,
    };

    if let Some(id) = data.get(name) {
        *id
    } else {
        let lower = name.to_lowercase();
        let id = xxh3_64(lower.as_bytes());
        data.insert(name.to_string(), id);
        id
    }
}

fn id_to_name(id: u64, data: &AppData, name_type: NameType) -> Option<String> {
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
