use crate::{
    AppData,
    AppWindow,
    Config,
    GContent,
    UiDialogue,
    common::{
        NotiLevel,
        id_to_state,
        show_noti,
        state_to_id,
    },
    content_tab::dialogue_ui::reload_dialogue,
};
use regex::Regex;
use slint::{
    ComponentHandle,
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

            ui.global::<GContent>().set_dialogues([].into());
            ui.global::<GContent>().set_dialogue(UiDialogue::default());
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
        reload_dialogue(&data, &ui, &config, "");
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
        let mut config = config.borrow_mut();
        let state_id = state_to_id(state_name.as_str(), &mut data);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
            class.remove(&state_id);
            config.selected_state = 0;
            reload_state(&mut data, &ui, &config, "");

            ui.global::<GContent>().set_dialogues([].into());
            ui.global::<GContent>().set_dialogue(UiDialogue::default());
        }
    }
}

pub fn rename_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |old_name, new_name| {
        if old_name == new_name {
            return;
        }

        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();

        let old_id = state_to_id(old_name.as_str(), &mut data);
        let new_id = state_to_id(new_name.as_str(), &mut data);

        if !data.dialogues.contains_key(&new_id) {
            if let Some(dialog) = data.dialogues.remove(&old_id) {
                data.dialogues.insert(new_id, dialog);
                config.selected_state = new_id;
                reload_state(&mut data, &ui, &config, "");
            }
        } else {
            show_noti(&ui, NotiLevel::Error, format!("Duplicated state {}", new_name).as_str());
        }
    }
}

pub fn search_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |search| {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui_handle.unwrap();

        reload_state(&mut data, &ui, &config, search.as_str());
        ui.global::<GContent>().set_dialogues([].into());
        ui.global::<GContent>().set_dialogue(UiDialogue::default());
    }
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
        ui.global::<GContent>().set_states(states.as_slice().into());
    }
}
