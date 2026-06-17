use crate::{
    AppData,
    AppWindow,
    Config,
    DataCache,
    reload_ui::{
        reload_all, reload_dialogue, reload_state, string_to_id
    },
};
use slint::{
    SharedString,
    Weak,
};
use std::{
    cell::RefCell,
    rc::Rc,
};

pub fn add_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |state_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let mut cache = cache.borrow_mut();
        let state_id = string_to_id(state_name.as_str(), &mut cache);
        // Notify if state already exists
        config.selected_state = state_id;
        data.state_name_map.entry(state_id).or_insert(state_name.to_string());
        if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
            class.entry(state_id).or_insert(Vec::new());
            reload_state(&data, &ui, &config, "");
        }
    }
}

pub fn select_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |state_name| {
        let ui = ui_handle.unwrap();
        let data = data.borrow();
        let mut config = config.borrow_mut();
        let mut cache = cache.borrow_mut();
        let state_id = string_to_id(state_name.as_str(), &mut cache);
        config.selected_state = state_id;
        reload_dialogue(&data, &ui, &config);
    }
}

pub fn remove_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |state_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let mut cache = cache.borrow_mut();
        let state_id = string_to_id(state_name.as_str(), &mut cache);
        if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
            class.remove(&state_id);
            // TODO: Remove state name map if no class contain it
            reload_state(&data, &ui, &config, "");
        }
    }
}

pub fn rename_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |old_name, new_name| {
        let mut cache = cache.borrow_mut();
        let old_id = string_to_id(old_name.as_str(), &mut cache);
        let new_id = string_to_id(new_name.as_str(), &mut cache);

        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();

        if !data.dialogues.contains_key(&new_id)
            && let Some(dialog) = data.dialogues.remove(&old_id)
        {
            data.dialogues.insert(new_id, dialog);
            data.state_name_map.entry(new_id).or_insert(new_name.to_string());
            config.selected_state = new_id;
            reload_all(&data, &ui, &config, "", "");
        } else {
            // TODO: Noti new name exists
        }
    }
}
