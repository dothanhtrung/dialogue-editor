use crate::{
    AppData,
    AppWindow,
    NameMap,
    StringId,
};
use regex_lite::Regex;
use slint::ComponentHandle;

pub fn reload_all_map(data: &mut AppData, ui: &AppWindow, search_class: &str, search_state: &str, search_event: &str) {
    reload_class_map(data, ui, search_class);
    reload_state_map(data, ui, search_state);
    reload_event_map(data, ui, search_event);
}

pub fn reload_class_map(data: &mut AppData, ui: &AppWindow, search_class: &str) {
    let mut class_map: Vec<StringId> = Vec::new();
    let re = Regex::new(search_class);
    for (name, id) in data.class_name_map.iter() {
        if search_class.is_empty() {
            class_map.push(StringId {
                id: id.to_string().into(),
                name: name.into(),
            });
        } else if let Ok(re) = re.as_ref()
            && re.is_match(&name)
        {
            class_map.push(StringId {
                id: id.to_string().into(),
                name: name.into(),
            });
        }
    }
    ui.global::<NameMap>().set_classes(class_map.as_slice().into());
}

pub fn reload_state_map(data: &mut AppData, ui: &AppWindow, search_state: &str) {
    let re = Regex::new(search_state);
    let mut state_map: Vec<StringId> = Vec::new();

    for (name, id) in data.state_name_map.iter() {
        if search_state.is_empty() {
            state_map.push(StringId {
                id: id.to_string().into(),
                name: name.into(),
            });
        } else if let Ok(re) = re.as_ref()
            && re.is_match(name.as_str())
        {
            state_map.push(StringId {
                id: id.to_string().into(),
                name: name.into(),
            });
        }
    }
    ui.global::<NameMap>().set_states(state_map.as_slice().into());
}

pub fn reload_event_map(data: &AppData, ui: &AppWindow, search_event: &str) {
    let re = Regex::new(search_event);
    let mut event_map: Vec<StringId> = Vec::new();

    for (name, id) in data.event_name_map.iter() {
        if search_event.is_empty() {
            event_map.push(StringId {
                id: id.to_string().into(),
                name: name.into(),
            });
        } else if let Ok(re) = re.as_ref()
            && re.is_match(name.as_str())
        {
            event_map.push(StringId {
                id: id.to_string().into(),
                name: name.into(),
            });
        }
    }
    ui.global::<NameMap>().set_events(event_map.as_slice().into());
}
