use crate::{
    Affect,
    AppData,
    AppWindow,
    Config,
    ContentLang,
    DataCache,
    UiDialogue,
};
use regex_lite::Regex;
use slint::{
    SharedString,
    ToSharedString,
};
use xxhash_rust::xxh3::xxh3_64;

pub fn reload_all(data: &AppData, ui: &AppWindow, config: &Config, search_class: &str, search_state: &str) {
    reload_class(data, ui, search_class);
    reload_state(data, ui, config, search_state);
    reload_dialogue(data, ui, config);
    reload_dialogue_detail(data, ui, config);
}

/// Reload class section and clear state/dialogue section
pub fn reload_class(data: &AppData, ui: &AppWindow, search_class: &str) {
    let mut classes: Vec<SharedString> = Vec::new();
    let re = Regex::new(search_class);
    for id in data.dialogues.keys() {
        let name = if let Some(name) = data.class_name_map.get(id) { name.clone() } else { id.to_string() };
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
pub fn reload_state(data: &AppData, ui: &AppWindow, config: &Config, search_state: &str) {
    let re = Regex::new(search_state);

    if let Some(class) = data.dialogues.get(&config.selected_class) {
        let mut states: Vec<SharedString> = Vec::new();
        for state_id in class.keys() {
            let state_name =
                if let Some(ret) = data.state_name_map.get(state_id) { ret.clone() } else { state_id.to_string() };
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
            let class_name =
                if let Some(name) = data.class_name_map.get(class) { name.into() } else { class.to_shared_string() };
            let state_name =
                if let Some(name) = data.state_name_map.get(state) { name.into() } else { class.to_shared_string() };
            affects.push(Affect {
                class: class_name,
                state: state_name,
            });
        }
        for event in dialog.events.iter() {
            let event =
                if let Some(name) = data.event_name_map.get(event) { name.into() } else { event.to_shared_string() };
            events.push(event);
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
        for (_, state) in data.state_name_map.iter() {
            state_list.push(state.into());
        }
        ui.set_state_list(state_list.as_slice().into());
    }
}

// TODO: Allow to manually set id of string without hashing
pub fn string_to_id(name: &str, cache: &mut DataCache) -> u64 {
    let lower = name.to_lowercase();
    if let Some(id) = cache.name_map.get(lower.as_str()) {
        *id
    } else {
        let id = xxh3_64(lower.as_bytes());
        cache.name_map.insert(lower, id);
        id
    }
}
