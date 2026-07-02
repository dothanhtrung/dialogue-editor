use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    AppData,
    AppWindow,
    GContent,
    GNameMap,
    StringId,
    common::{
        NameType,
        NotiLevel,
        class_to_id,
        event_to_id,
        show_noti,
        state_to_id,
    },
};
use regex_lite::Regex;
use slint::{
    ComponentHandle,
    SharedString,
    Weak,
};
use xxhash_rust::xxh3::xxh3_64;

// TODO: Update selected class, selected state

pub fn delete_class_map(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();
        data.class_name_map.remove(name.as_str()); // TODO: Check if content is using this
        reload_class_map(&data, &ui, ""); // TODO: Reload tab separate
    }
}

pub fn delete_state_map(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();
        data.state_name_map.remove(name.as_str()); // TODO: Check if content is using this
        reload_state_map(&data, &ui, "");
    }
}

pub fn delete_event_map(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();
        data.event_name_map.remove(name.as_str()); // TODO: Check if content is using this

        reload_event_map(&data, &ui, "");
    }
}

pub fn update_class_id(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();
        let old_id = class_to_id(name.as_str(), &mut data);

        if let Ok(new_id) = update_id(&mut data, &ui, name.to_string(), old_id, id.as_str(), NameType::Class)
            && let Some(value) = data.dialogues.remove(&old_id)
        {
            data.dialogues.insert(new_id, value);
        }
        reload_class_map(&data, &ui, "");
    }
}

pub fn update_state_id(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();
        let old_id = state_to_id(name.as_str(), &mut data);
        if let Ok(new_id) = update_id(&mut data, &ui, name.to_string(), old_id, id.as_str(), NameType::State) {
            for (_, class) in data.dialogues.iter_mut() {
                if let Some(value) = class.remove(&old_id) {
                    class.insert(new_id, value);
                }
            }
        }
        reload_state_map(&data, &ui, "");
    }
}

pub fn update_event_id(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();
        let old_id = event_to_id(name.as_str(), &mut data);
        if let Ok(new_id) = update_id(&mut data, &ui, name.to_string(), old_id, id.as_str(), NameType::Event) {
            for (_, class) in data.dialogues.iter_mut() {
                for (_, state) in class.iter_mut() {
                    for dialogue in state.iter_mut() {
                        let Some(old_event) = dialogue.events.iter().position(|id| *id == old_id) else {
                            continue;
                        };
                        dialogue.events.remove(old_event);
                        dialogue.events.push(new_id);
                    }
                }
            }
        }

        reload_event_map(&data, &ui, "");
    }
}

pub fn add_new_class(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();

        if add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::Class).is_ok() {
            reload_class_map(&mut data, &ui, "");
        }
    }
}

pub fn add_new_state(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();

        if add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::State).is_ok() {
            reload_state_map(&mut data, &ui, "");
        }
    }
}

pub fn add_new_event(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();

        if add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::Event).is_ok() {
            reload_event_map(&data, &ui, "");
        }
    }
}

fn add_new(data: &mut AppData, ui: &AppWindow, new_name: String, new_id: &str, name_type: NameType) -> Result<u64, ()> {
    let data = match name_type {
        NameType::Class => &mut data.class_name_map,
        NameType::State => &mut data.state_name_map,
        NameType::Event => &mut data.event_name_map,
    };

    if data.contains_key(&new_name) {
        show_noti(
            ui,
            NotiLevel::Error,
            format!("Name already exists: {}", &new_name).as_str(),
        );
        return Err(());
    }

    let new_id = if new_id.is_empty() {
        let lower = new_name.to_lowercase();
        xxh3_64(lower.as_bytes())
    } else {
        new_id.parse::<u64>().unwrap_or_else(|_| {
            show_noti(ui, NotiLevel::Warn, format!("Invalid id: {}", new_id).as_str());
            xxh3_64(new_id.as_bytes())
        })
    };

    data.insert(new_name, new_id);
    Ok(new_id)
}

fn update_id(
    data: &mut AppData,
    ui: &AppWindow,
    name: String,
    old_id: u64,
    new_id: &str,
    name_type: NameType,
) -> Result<u64, ()> {
    let data = match name_type {
        NameType::Class => &mut data.class_name_map,
        NameType::State => &mut data.state_name_map,
        NameType::Event => &mut data.event_name_map,
    };

    if let Ok(new_id) = new_id.parse::<u64>() {
        if old_id == new_id {
            return Err(());
        }

        for (_, id) in data.iter() {
            if *id == new_id {
                show_noti(ui, NotiLevel::Error, format!("Duplicated id: {}", new_id).as_str());
                return Err(());
            }
        }
        data.entry(name).and_modify(|e| *e = new_id).or_insert(new_id);

        Ok(new_id)
    } else {
        show_noti(ui, NotiLevel::Error, format!("Invalid id: {}", new_id).as_str());
        Err(())
    }
}

pub fn reload_namemap_tab(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn() {
    move || {
        let data = data.borrow();
        let ui = ui.unwrap();
        reload_all_map(&data, &ui);
    }
}

pub fn reload_all_map(data: &AppData, ui: &AppWindow) {
    reload_class_map(data, ui, "");
    reload_state_map(data, ui, "");
    reload_event_map(data, ui, "");
}

fn reload_class_map(data: &AppData, ui: &AppWindow, search: &str) {
    let class_map = reload_map(data, NameType::Class, search);
    ui.global::<GNameMap>().set_classes(class_map.as_slice().into());
}

fn reload_state_map(data: &AppData, ui: &AppWindow, search: &str) {
    let class_map = reload_map(data, NameType::State, search);
    ui.global::<GNameMap>().set_states(class_map.as_slice().into());

    let mut state_list: Vec<SharedString> = Vec::new();
    for state in data.state_name_map.keys() {
        state_list.push(state.into());
    }
    ui.global::<GContent>().set_state_list(state_list.as_slice().into());
}

// TODO: Why is this delay compare to reload_class_map and reload_state_map?
fn reload_event_map(data: &AppData, ui: &AppWindow, search: &str) {
    let event_map = reload_map(data, NameType::Event, search);

    ui.global::<GNameMap>().set_events(event_map.as_slice().into());

    let mut event_list: Vec<SharedString> = Vec::new();
    for event in data.event_name_map.keys() {
        event_list.push(event.into());
    }
    ui.global::<GContent>().set_event_list(event_list.as_slice().into());
}

fn reload_map(data: &AppData, name_type: NameType, search: &str) -> Vec<StringId> {
    let data = match name_type {
        NameType::Class => &data.class_name_map,
        NameType::State => &data.state_name_map,
        NameType::Event => &data.event_name_map,
    };

    let mut ret = Vec::new();
    let re = Regex::new(search);

    for (name, id) in data.iter() {
        if search.is_empty() || (re.is_ok() && re.as_ref().unwrap().is_match(name.as_str())) {
            ret.push(StringId {
                id: id.to_string().into(),
                name: name.clone().into(),
            })
        }
    }
    ret
}
