use crate::{
    AppData,
    AppWindow,
    Config,
    common::{
        NotiLevel,
        show_noti,
    },
    content_tab::{
        reload_all,
        reload_dialogue,
        reload_state,
        state_to_id,
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
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |state_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let state_id = state_to_id(state_name.as_str(), &mut data);

        config.selected_state = state_id;
        if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
            class.entry(state_id).or_insert_with(|| {
                show_noti(
                    &ui,
                    NotiLevel::Warn,
                    format!("State {} already exists", &state_name).as_str(),
                );
                Vec::new()
            });
            reload_state(&mut data, &ui, &config, "");
        }
    }
}

pub fn select_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |state_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let state_id = state_to_id(state_name.as_str(), &mut data);
        config.selected_state = state_id;
        reload_dialogue(&data, &ui, &config);
    }
}

pub fn remove_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |state_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let state_id = state_to_id(state_name.as_str(), &mut data);
        if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
            class.remove(&state_id);
            reload_state(&mut data, &ui, &config, "");
        }
    }
}

pub fn rename_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |old_name, new_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();

        let old_id = state_to_id(old_name.as_str(), &mut data);
        let new_id = state_to_id(new_name.as_str(), &mut data);

        if !data.dialogues.contains_key(&new_id) {
            if let Some(dialog) = data.dialogues.remove(&old_id) {
                data.dialogues.insert(new_id, dialog);
                config.selected_state = new_id;
                reload_all(&mut data, &ui, &config, "", "");
            }
        } else {
            show_noti(
                &ui,
                NotiLevel::Error,
                format!("Duplicated state {}", new_name).as_str(),
            );
        }
    }
}
