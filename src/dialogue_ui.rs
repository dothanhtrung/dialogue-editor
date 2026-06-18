use std::{cell::RefCell, rc::Rc};
use isolang::Language;
use slint::{SharedString, Weak};
use crate::{AppData, AppWindow, Config, DataCache, Dialogue, UiDialogue, reload_ui::{reload_dialogue, reload_dialogue_detail, string_to_id}};

pub fn add_dialogue(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui_handle: Weak<AppWindow>) -> impl Fn() {
    move || {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();

        if let Some(state_list) = data.dialogues.get_mut(&config.selected_class)
            && let Some(dialogues) = state_list.get_mut(&config.selected_state)
        {
            dialogues.push(Dialogue::default());
            config.selected_dialog = dialogues.len() - 1;
            reload_dialogue(&data, &ui, &config);
        }
    }
}

pub fn select_dialogue(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(i32) {
    move |dialog_id| {
        let ui = ui_handle.unwrap();
        let data = data.borrow_mut();
        let mut config = config.borrow_mut();

        config.selected_dialog = dialog_id as usize;
        reload_dialogue_detail(&data, &ui, &config);
    }
}

pub fn remove_dialogue(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(i32) {
    move |dialog_id| {
        let dialog_id = dialog_id as usize;
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();
        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && dialog_id < state.len()
        {
            state.remove(dialog_id);
            reload_dialogue(&data, &ui, &config);
        }
    }
}

pub fn add_lang_content(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |lang, content| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            let lang = Language::from_639_3(lang.as_str()).unwrap_or_default();
            dialogue.contents.insert(lang, content.to_string());
            reload_dialogue_detail(&data, &ui, &config);
        }
    }
}

pub fn add_affect(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |class_name, state_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let mut cache = cache.borrow_mut();
        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            let class_id = string_to_id(class_name.as_str(), &mut cache);
            let state_id = string_to_id(state_name.as_str(), &mut cache);

            dialogue.affects.insert(class_id, state_id);
            reload_dialogue_detail(&data, &ui, &config);
        }
    }
}

pub fn update_content(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(UiDialogue) {
    move |ui_dialogue| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            *dialogue = Dialogue::from(ui_dialogue);
            reload_dialogue_detail(&data, &ui, &config);
        }
    }
}

pub fn delete_content(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |lang| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            dialogue
                .contents
                .remove(&Language::from_639_3(lang.to_string().as_str()).unwrap_or_default());
        }
        reload_dialogue_detail(&data, &ui, &config);
    }
}

pub fn delete_affect(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    cache: Rc<RefCell<DataCache>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let mut cache = cache.borrow_mut();

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            let class_id = string_to_id(class_name.as_str(), &mut cache);
            dialogue.affects.remove(&class_id);
        }
        reload_dialogue_detail(&data, &ui, &config);
    }
}
