use crate::{
    AppData,
    AppWindow,
    Config,
    NotiLevel,
    content_tab::{
        class_to_id,
        reload_all,
        reload_class,
        reload_state,
        show_noti,
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
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let class_id = class_to_id(class_name.as_str(), &mut data);

        if data.dialogues.contains_key(&class_id) {
            show_noti(
                &ui,
                crate::NotiLevel::Warn,
                format!("Dialogue for class {} already exists", class_name).as_str(),
            );
        } else {
            data.dialogues.insert(class_id, BTreeMap::new());
        }
        config.selected_class = class_id;
        config.selected_state = 0;
        reload_class(&mut data, &ui, "");
    }
}

pub fn rename_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |old_name, new_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let old_class_id = class_to_id(old_name.as_str(), &mut data);
        let new_class_id = class_to_id(new_name.as_str(), &mut data);

        if !data.dialogues.contains_key(&new_class_id) {
            if let Some(value) = data.dialogues.remove(&old_class_id) {
                data.dialogues.insert(new_class_id, value);
                config.selected_class = new_class_id;
                reload_all(&mut data, &ui, &config, "", "");
            }
        } else {
            show_noti(&ui, NotiLevel::Error, format!("Duplicated class {}", new_name).as_str());
        }
    }
}

// TODO: class should be removed from name_map tab, not from content tab
pub fn remove_class(data: Rc<RefCell<AppData>>, ui_handle: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let class_id = class_to_id(class_name.as_str(), &mut data);
        // data.class_name_map.remove(&class_name.to_string());
        // TODO: Add clean function to remove orphan classs/states/events
        data.dialogues.remove(&class_id);
        reload_class(&mut data, &ui, "");
    }
}

pub fn select_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let class_id = class_to_id(class_name.as_str(), &mut data);
        config.selected_class = class_id;
        reload_state(&mut data, &ui, &config, "");
    }
}
