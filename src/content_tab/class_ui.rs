use crate::common::id_to_name;
use crate::content_tab::dialogue_ui::reload_dialogue_detail;
use crate::history::{
    Action,
    ActionTarget,
    ActionType,
};
use crate::namemap_tab::reload_class_map;
use crate::{
    AppData,
    AppWindow,
    Config,
    GContent,
    UiDialogue,
    common::{
        NotiLevel,
        name_to_id,
        new_regex,
        show_noti,
    },
    content_tab::state_ui::reload_state,
};
use indexmap::IndexMap;
use slint::{
    ComponentHandle,
    SharedString,
    Weak,
};
use std::{
    cell::RefCell,
    rc::Rc,
};

pub fn add_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        if class_name.is_empty() {
            return;
        }

        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let class_id = name_to_id(class_name.as_str(), &mut data.class_name_map);

        if data.dialogues.contains_key(&class_id) {
            show_noti(
                &ui,
                NotiLevel::Warn,
                format!("Dialogue for class {} already exists", class_name).as_str(),
            );
        } else {
            data.dialogues.insert(class_id, IndexMap::new());
            config.history.add_undo(
                Action {
                    action: ActionType::DeleteStr(class_name.to_string()),
                    target: ActionTarget::ContentClass(None),
                },
                &ui,
            );
        }

        config.selected_class = class_id;
        config.selected_state = 0;
        reload_class(&mut data, &ui, "");
        reload_class_map(&data, &ui, "");

        ui.global::<GContent>().set_selecting_class(class_name);
        ui.global::<GContent>().set_selecting_state(SharedString::new());

        ui.global::<GContent>().set_states([].into());
        ui.global::<GContent>().set_dialogues([].into());
        ui.global::<GContent>().set_dialogue(UiDialogue::default());
    }
}

pub fn rename_class(
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

        let old_class_id = name_to_id(old_name.as_str(), &mut data.class_name_map);
        let new_class_id = name_to_id(new_name.as_str(), &mut data.class_name_map);

        if !data.dialogues.contains_key(&new_class_id) {
            if update_class(&mut data, old_class_id, new_class_id).is_ok() {
                config.history.add_undo(
                    Action {
                        action: ActionType::UpdateStr(new_name.to_string(), old_name.to_string()),
                        target: ActionTarget::ContentClass(None),
                    },
                    &ui,
                );

                config.selected_class = new_class_id;
                reload_class(&mut data, &ui, "");
                reload_class_map(&data, &ui, "");
                reload_dialogue_detail(&data, &ui, &config);

                ui.global::<GContent>().set_selecting_class(new_name);
            }
        } else {
            show_noti(&ui, NotiLevel::Error, format!("Duplicated class {}", new_name).as_str());
        }

        // Avoid missing focus on global keybinding
        ui.invoke_focus();
    }
}

pub fn update_class(data: &mut AppData, old_class_id: u64, new_class_id: u64) -> Result<(), ()> {
    if let Some(value) = data.dialogues.shift_remove(&old_class_id) {
        data.dialogues.insert(new_class_id, value);

        // Update affect list
        for (_, class) in data.dialogues.iter_mut() {
            for (_, state) in class.iter_mut() {
                for dialogue in state.iter_mut() {
                    if let Some(affect_state) = dialogue.affects.remove(&old_class_id) {
                        dialogue.affects.insert(new_class_id, affect_state);
                    }
                }
            }
        }

        return Ok(());
    }
    Err(())
}

pub fn remove_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let class_id = name_to_id(class_name.as_str(), &mut data.class_name_map);

        let states = data.dialogues.shift_remove(&class_id);
        config.selected_class = 0;
        reload_class(&mut data, &ui, "");

        config.history.add_undo(
            Action {
                action: ActionType::AddStr(class_name.to_string()),
                target: ActionTarget::ContentClass(states),
            },
            &ui,
        );

        ui.global::<GContent>().set_selecting_class(SharedString::new());

        ui.global::<GContent>().set_states([].into());
        ui.global::<GContent>().set_dialogues([].into());
        ui.global::<GContent>().set_dialogue(UiDialogue::default());
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
        let class_id = name_to_id(class_name.as_str(), &mut data.class_name_map);
        config.selected_class = class_id;
        reload_state(&mut data, &ui, &config, "");

        ui.global::<GContent>().set_selecting_class(class_name);

        ui.global::<GContent>().set_dialogues([].into());
        ui.global::<GContent>().set_dialogue(UiDialogue::default());
    }
}

pub fn search_class(data: Rc<RefCell<AppData>>, ui_handle: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |search| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        reload_class(&mut data, &ui, search.as_str());
        ui.global::<GContent>().set_states([].into());
        ui.global::<GContent>().set_dialogues([].into());
        ui.global::<GContent>().set_dialogue(UiDialogue::default());
    }
}

pub fn reload_class(data: &mut AppData, ui: &AppWindow, search_class: &str) {
    let mut classes: Vec<SharedString> = Vec::new(); // For content tab

    let re = new_regex(search_class);
    for id in data.dialogues.keys() {
        let name = id_to_name(*id, &data.class_name_map);
        let name = match name {
            Some(ret) => ret,
            None => {
                let id_string = id.to_string();
                data.class_name_map.insert(id_string.clone(), *id);
                id_string
            }
        };

        if search_class.is_empty() {
            classes.push(name.into());
        } else if let Ok(re) = re.as_ref()
            && re.is_match(&name)
        {
            classes.push(name.into());
        }
    }

    ui.global::<GContent>().set_classes(classes.as_slice().into());
}
