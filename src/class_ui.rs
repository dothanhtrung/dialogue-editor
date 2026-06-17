use crate::{
    AppData,
    AppWindow,
    Config,
    DataCache,
    reload_ui::{
        reload_all,
        reload_class,
        reload_state,
        string_to_id,
    },
};
use slint::{
    SharedString,
    Weak,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
};

pub fn add_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let mut cache = cache.borrow_mut();
        // TODO: Cache the id to avoid hashing too many times.
        let class_id = string_to_id(class_name.as_str(), &mut cache);
        // TODO: Notify if class already exist
        data.class_name_map.entry(class_id).or_insert(class_name.to_string());
        data.dialogues.entry(class_id).or_insert(BTreeMap::new());
        config.selected_class = class_id;
        config.selected_state = 0;
        reload_class(&data, &ui, "");
    }
}

pub fn rename_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |old_name, new_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let mut cache = cache.borrow_mut();
        let old_class_id = string_to_id(old_name.as_str(), &mut cache);
        let new_class_id = string_to_id(new_name.as_str(), &mut cache);

        if !data.dialogues.contains_key(&new_class_id) {
            if let Some(value) = data.dialogues.remove(&old_class_id) {
                data.dialogues.insert(new_class_id, value);
                data.class_name_map.entry(new_class_id).or_insert(new_name.to_string());
                data.class_name_map.remove(&old_class_id);

                config.selected_class = new_class_id;
                reload_all(&data, &ui, &config, "", "");
            }
        } else {
            // TODO: Noti new name already exists
        }
    }
}

pub fn remove_class(
    data: Rc<RefCell<AppData>>,

    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut cache = cache.borrow_mut();
        let class_id = string_to_id(class_name.as_str(), &mut cache);
        data.class_name_map.remove(&class_id);
        // TODO: Remove state name map if no class contain it
        data.dialogues.remove(&class_id);
        reload_class(&data, &ui, "");
    }
}

pub fn select_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let data = data.borrow();
        let mut config = config.borrow_mut();
        let mut cache = cache.borrow_mut();
        let class_id = string_to_id(class_name.as_str(), &mut cache);
        config.selected_class = class_id;
        reload_state(&data, &ui, &config, "");
    }
}
