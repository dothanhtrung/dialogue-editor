use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    AppData,
    AppWindow,
    Config,
    GNameMap,
    StringId,
    common::{
        NameType,
        NotiLevel,
        class_to_id,
        event_to_id,
        reload_all,
        reload_class,
        reload_class_map,
        reload_dialogue_detail,
        reload_event_map,
        reload_state,
        reload_state_map,
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

pub fn delete_class_map(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();
        data.class_name_map.remove(name.as_str()); // TODO: Check if content is using this
        reload_all(&mut data, &ui, &config, "", ""); // TODO: Reload tab separate
    }
}

pub fn delete_state_map(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();
        data.state_name_map.remove(name.as_str()); // TODO: Check if content is using this
        reload_all(&mut data, &ui, &config, "", "");
    }
}

pub fn delete_event_map(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();
        data.event_name_map.remove(name.as_str()); // TODO: Check if content is using this
        reload_all(&mut data, &ui, &config, "", "");
    }
}

pub fn update_class_id(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();
        let old_id = class_to_id(name.as_str(), &mut data);

        if let Ok(new_id) = update_id(&mut data, &ui, name.to_string(), old_id, id.as_str(), NameType::Class)
            && let Some(value) = data.dialogues.remove(&old_id)
        {
            data.dialogues.insert(new_id, value);
        }
        reload_all(&mut data, &ui, &config, "", "");
    }
}

pub fn update_state_id(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();
        let old_id = state_to_id(name.as_str(), &mut data);
        if let Ok(new_id) = update_id(&mut data, &ui, name.to_string(), old_id, id.as_str(), NameType::State) {
            for (_, class) in data.dialogues.iter_mut() {
                if let Some(value) = class.remove(&old_id) {
                    class.insert(new_id, value);
                }
            }
        }
        reload_all(&mut data, &ui, &config, "", "");
    }
}

pub fn update_event_id(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
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
        reload_all(&mut data, &ui, &config, "", "");
    }
}

pub fn add_new_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();

        if add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::Class).is_ok() {
            reload_all(&mut data, &ui, &config, "", "");
        }
    }
}

pub fn add_new_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();

        if add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::State).is_ok() {
            reload_all(&mut data, &ui, &config, "", "");
        }
    }
}

pub fn add_new_event(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();

        if add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::Event).is_ok() {
            reload_all(&mut data, &ui, &config, "", "");
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

    for (_, old_id) in data.iter() {
        if *old_id == new_id {
            show_noti(ui, NotiLevel::Error, format!("Id already exists: {}", new_id).as_str());
            return Err(());
        }
    }

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

pub fn reload_all_map(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) -> impl Fn() {
    move || {
        let mut data = data.borrow_mut();
        let ui = ui.unwrap();
        reload_class_map(&data, &ui, "");
        reload_state_map(&data, &ui, "");
        reload_event_map(&data, &ui, "");
    }
}
