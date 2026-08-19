use crate::{
    Affect,
    AppData,
    AppWindow,
    Config,
    ContentLang,
    Dialogue,
    GContent,
    UiDialogue,
    common::{
        NotiLevel,
        id_to_name,
        name_to_id,
        new_regex,
        show_noti,
    },
    history::{
        Action,
        ActionTarget,
        ActionType,
    },
};
use isolang::Language;
use slint::{
    ComponentHandle,
    SharedString,
    Weak,
};
use std::{
    cell::RefCell,
    rc::Rc,
    str::FromStr,
};

pub fn add_dialogue(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui_handle: Weak<AppWindow>) -> impl Fn() {
    move || {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();

        if let Some(state_list) = data.dialogues.get_mut(&config.selected_class)
            && let Some(dialogues) = state_list.get_mut(&config.selected_state)
        {
            dialogues.push(Dialogue::default());
            config.selected_dialogue = dialogues.len() - 1;
            reload_dialogue(&data, &ui, &config, "");

            let class_id = config.selected_class;
            let state_id = config.selected_state;
            let dialogue_pos = config.selected_dialogue;
            config.history.add_undo(
                Action {
                    action: ActionType::DeleteStr(String::new()),
                    target: ActionTarget::ContentDialogue(class_id, state_id, dialogue_pos, Dialogue::default()),
                },
                &ui,
            );

            ui.global::<GContent>()
                .set_selecting_dialogue(config.selected_dialogue as i32);
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

        config.selected_dialogue = dialog_id as usize;
        reload_dialogue_detail(&data, &ui, &config);

        ui.global::<GContent>()
            .set_selecting_dialogue(config.selected_dialogue as i32);
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
        let mut config = config.borrow_mut();
        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && dialog_id < state.len()
        {
            let removed_dialogue = state.remove(dialog_id);
            config.selected_dialogue = 0;
            reload_dialogue(&data, &ui, &config, "");

            let class_id = config.selected_class;
            let state_id = config.selected_state;
            config.history.add_undo(
                Action {
                    action: ActionType::AddStr(String::new()),
                    target: ActionTarget::ContentDialogue(class_id, state_id, dialog_id, removed_dialogue),
                },
                &ui,
            );

            ui.global::<GContent>()
                .set_selecting_dialogue(config.selected_dialogue as i32);
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
        let mut config = config.borrow_mut();

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialogue)
        {
            let lang = Language::from_str(lang.as_str()).unwrap_or_default();
            if dialogue.contents.contains_key(&lang) {
                show_noti(
                    &ui,
                    NotiLevel::Warn,
                    format!("Dialogue for language {} already exists", lang).as_str(),
                );
                return;
            }

            dialogue.contents.insert(lang, content.to_string());
            reload_dialogue_detail(&data, &ui, &config);

            let class_id = config.selected_class;
            let state_id = config.selected_state;
            let dialogue_pos = config.selected_dialogue;
            config.history.add_undo(
                Action {
                    action: ActionType::DeleteStr(content.to_string()),
                    target: ActionTarget::ContentLang(class_id, state_id, dialogue_pos, lang),
                },
                &ui,
            );
        }
    }
}

pub fn add_affect(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |class_name, state_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();

        let affect_class = name_to_id(class_name.as_str(), &mut data.class_name_map);
        let affect_state = name_to_id(state_name.as_str(), &mut data.state_name_map);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialogue)
        {
            dialogue.affects.insert(affect_class, affect_state);
            reload_dialogue_detail(&data, &ui, &config);

            let class_id = config.selected_class;
            let state_id = config.selected_state;
            let dialogue_pos = config.selected_dialogue;
            config.history.add_undo(
                Action {
                    action: ActionType::DeleteId(affect_state),
                    target: ActionTarget::ContentAffect(class_id, state_id, dialogue_pos, affect_class),
                },
                &ui,
            );
        }
    }
}

pub fn add_event(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |event_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let event_id = name_to_id(event_name.as_str(), &mut data.event_name_map);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialogue)
        {
            dialogue.events.insert(event_id);
            reload_dialogue_detail(&data, &ui, &config);

            let class_id = config.selected_class;
            let state_id = config.selected_state;
            let dialogue_pos = config.selected_dialogue;
            config.history.add_undo(
                Action {
                    action: ActionType::DeleteId(event_id),
                    target: ActionTarget::ContentEvent(class_id, state_id, dialogue_pos),
                },
                &ui,
            );
        }
    }
}

pub fn update_content(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |update_lang, update_content| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();

        let class_id = config.selected_class;
        let state_id = config.selected_state;
        let dialogue_pos = config.selected_dialogue;

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialogue)
        {
            let Ok(lang) = Language::from_str(&update_lang) else {
                show_noti(
                    &ui,
                    NotiLevel::Error,
                    format!("Invalid language: {}", update_lang).as_str(),
                );
                return;
            };
            if let Some(old_content) = dialogue.contents.get(&lang) {
                config.history.add_undo(
                    Action {
                        action: ActionType::UpdateStr(update_content.to_string(), old_content.clone()),
                        target: ActionTarget::ContentLang(class_id, state_id, dialogue_pos, lang),
                    },
                    &ui,
                );
            }
            dialogue.contents.insert(lang, update_content.into());
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
        let mut config = config.borrow_mut();

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialogue)
        {
            let Ok(lang) = Language::from_str(lang.to_string().as_str()) else {
                return;
            };
            if let Some(content) = dialogue.contents.shift_remove(&lang) {
                let class_id = config.selected_class;
                let state_id = config.selected_state;
                let dialogue_pos = config.selected_dialogue;
                config.history.add_undo(
                    Action {
                        action: ActionType::AddStr(content),
                        target: ActionTarget::ContentLang(class_id, state_id, dialogue_pos, lang),
                    },
                    &ui,
                );
            }

            reload_dialogue_detail(&data, &ui, &config);
        }
    }
}

pub fn delete_affect(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let affect_class = name_to_id(class_name.as_str(), &mut data.class_name_map);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialogue)
        {
            if let Some(affect_state) = dialogue.affects.remove(&affect_class) {
                let class_id = config.selected_class;
                let state_id = config.selected_state;
                let dialogue_pos = config.selected_dialogue;
                config.history.add_undo(
                    Action {
                        action: ActionType::AddId(affect_state),
                        target: ActionTarget::ContentAffect(class_id, state_id, dialogue_pos, affect_class),
                    },
                    &ui,
                );
            }
            reload_dialogue_detail(&data, &ui, &config);
        }
    }
}

pub fn delete_event(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |event| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let event_id = name_to_id(event.as_str(), &mut data.event_name_map);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialogue)
        {
            dialogue.events.remove(&event_id);

            let class_id = config.selected_class;
            let state_id = config.selected_state;
            let dialogue_pos = config.selected_dialogue;
            config.history.add_undo(
                Action {
                    action: ActionType::AddId(event_id),
                    target: ActionTarget::ContentEvent(class_id, state_id, dialogue_pos),
                },
                &ui,
            );
        }
        reload_dialogue_detail(&data, &ui, &config);
    }
}

pub fn search_dialogue(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |search| {
        let data = data.borrow();
        let mut config = config.borrow_mut();
        let ui = ui_handle.unwrap();

        config.selected_dialogue = 0;
        reload_dialogue(&data, &ui, &config, search.as_str());

        ui.global::<GContent>()
            .set_selecting_dialogue(config.selected_dialogue as i32);
    }
}

pub fn reload_dialogue(data: &AppData, ui: &AppWindow, config: &Config, search: &str) {
    if let Some(state) = data.dialogues.get(&config.selected_class)
        && let Some(state_dialogs) = state.get(&config.selected_state)
    {
        let mut dialogues: Vec<SharedString> = Vec::new();
        let re = new_regex(search);
        for dialog in state_dialogs {
            let content = if !config.langs.is_empty()
                && let Some(content) = dialog.contents.get(config.langs.first().unwrap())
            {
                content.into()
            } else if let Some((_, content)) = dialog.contents.first() {
                content.into()
            } else {
                SharedString::new()
            };

            if search.is_empty() || (re.is_ok() && re.as_ref().unwrap().is_match(&content)) {
                dialogues.push(content);
            }
        }

        ui.global::<GContent>().set_dialogues(dialogues.as_slice().into());
        ui.global::<GContent>().set_dialogue(UiDialogue::default());
    }
}

pub fn reload_dialogue_detail(data: &AppData, ui: &AppWindow, config: &Config) {
    if let Some(state) = data.dialogues.get(&config.selected_class)
        && let Some(dialog_list) = state.get(&config.selected_state)
        && let Some(dialog) = dialog_list.get(config.selected_dialogue)
    {
        let mut contents: Vec<ContentLang> = Vec::new();
        let mut affects: Vec<Affect> = Vec::new();
        let mut events: Vec<SharedString> = Vec::new();
        for (lang, content) in dialog.contents.iter() {
            contents.push(ContentLang {
                language: lang.to_639_3().to_string().into(),
                content: content.into(),
            });
        }
        for (class, state) in dialog.affects.iter() {
            let class_name = id_to_name(*class, &data.class_name_map).unwrap_or(class.to_string());
            let state_name = id_to_name(*state, &data.state_name_map).unwrap_or(state.to_string());
            affects.push(Affect {
                class: class_name.into(),
                state: state_name.into(),
            });
        }
        for event in dialog.events.iter() {
            let event = id_to_name(*event, &data.event_name_map).unwrap_or(event.to_string());
            events.push(event.into());
        }

        let ui_dialogue = UiDialogue {
            contents: contents.as_slice().into(),
            affects: affects.as_slice().into(),
            events: events.as_slice().into(),
        };
        ui.global::<GContent>().set_dialogue(ui_dialogue);
    }
}

pub fn search_event(data: Rc<RefCell<AppData>>, ui_handle: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |search| {
        let data = data.borrow();
        let ui = ui_handle.unwrap();
        let mut events: Vec<SharedString> = Vec::new();
        let re = new_regex(search.as_str());

        for event in data.event_name_map.keys() {
            if search.is_empty() {
                events.push(event.into());
            } else if let Ok(re) = re.as_ref()
                && re.is_match(event)
            {
                events.push(event.into());
            }
        }

        ui.global::<GContent>().set_event_list(events.as_slice().into());
    }
}

pub fn search_affect_class(data: Rc<RefCell<AppData>>, ui_handle: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |search| {
        let data = data.borrow();
        let ui = ui_handle.unwrap();
        let mut classes = Vec::new();
        let re = new_regex(search.as_str());

        for class in data.class_name_map.keys() {
            if search.is_empty() {
                classes.push(class.into());
            } else if let Ok(re) = re.as_ref()
                && re.is_match(class)
            {
                classes.push(class.into());
            }
        }
        ui.global::<GContent>().set_class_list(classes.as_slice().into());
    }
}

pub fn search_affect_state(
    data: Rc<RefCell<AppData>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |search_class, search| {
        let mut data = data.borrow_mut();
        let ui = ui_handle.unwrap();
        let mut states = Vec::new();
        let re = new_regex(search.as_str());

        let mut base_list: Vec<String> = Vec::new();
        if search_class.is_empty() {
            base_list = data.state_name_map.keys().map(|keys| keys.clone()).collect();
        } else {
            let class_id = name_to_id(search_class.as_str(), &mut data.class_name_map);
            // Only search in the state of selected class
            if let Some(class) = data.dialogues.get(&class_id) {
                for state_id in class.keys() {
                    if let Some(state_name) = id_to_name(*state_id, &data.state_name_map) {
                        base_list.push(state_name);
                    }
                }
            } else {
                base_list = data.state_name_map.keys().map(|keys| keys.clone()).collect();
            }
        }

        for state in base_list {
            if search.is_empty() {
                states.push(state.into());
            } else if let Ok(re) = re.as_ref()
                && re.is_match(&state)
            {
                states.push(state.into());
            }
        }
        ui.global::<GContent>().set_state_list(states.as_slice().into());
    }
}
