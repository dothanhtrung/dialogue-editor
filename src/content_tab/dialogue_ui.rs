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
        class_to_id,
        event_to_id,
        id_to_class,
        id_to_event,
        id_to_state,
        new_regex,
        state_to_id,
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
            config.selected_dialog = dialogues.len() - 1;
            reload_dialogue(&data, &ui, &config, "");
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
            reload_dialogue(&data, &ui, &config, "");
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
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |class_name, state_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();

        let class_id = class_to_id(class_name.as_str(), &mut data);
        let state_id = state_to_id(state_name.as_str(), &mut data);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            dialogue.affects.insert(class_id, state_id);
            reload_dialogue_detail(&data, &ui, &config);
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
        let config = config.borrow();
        let event_id = event_to_id(event_name.as_str(), &mut data);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            dialogue.events.push(event_id);
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
        let ui_dialogue = Dialogue::from(ui_dialogue, &mut data);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            *dialogue = ui_dialogue;
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
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |class_name| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();

        let class_id = class_to_id(class_name.as_str(), &mut data);

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            dialogue.affects.remove(&class_id);
        }
        reload_dialogue_detail(&data, &ui, &config);
    }
}

pub fn delete_event(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(i32) {
    move |index| {
        let ui = ui_handle.unwrap();
        let mut data = data.borrow_mut();
        let config = config.borrow();

        if let Some(class) = data.dialogues.get_mut(&config.selected_class)
            && let Some(state) = class.get_mut(&config.selected_state)
            && let Some(dialogue) = state.get_mut(config.selected_dialog)
        {
            dialogue.events.remove(index as usize);
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
        let config = config.borrow();
        let ui = ui_handle.unwrap();

        reload_dialogue(&data, &ui, &config, search.as_str());
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
            } else if let Some((_, content)) = dialog.contents.first_key_value() {
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
        && let Some(dialog) = dialog_list.get(config.selected_dialog)
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
            let class_name = id_to_class(*class, data).unwrap_or(class.to_string());
            let state_name = id_to_state(*state, data).unwrap_or(state.to_string());
            affects.push(Affect {
                class: class_name.into(),
                state: state_name.into(),
            });
        }
        for event in dialog.events.iter() {
            let event = id_to_event(*event, data).unwrap_or(event.to_string());
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
