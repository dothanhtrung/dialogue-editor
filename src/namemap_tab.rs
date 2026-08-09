use crate::{
    AppData,
    AppWindow,
    Config,
    GContent,
    GNameMap,
    ListItem,
    StringId,
    common::{
        NameType,
        NotiLevel,
        class_to_id,
        event_to_id,
        id_to_class,
        id_to_state,
        new_regex,
        show_noti,
        state_to_id,
    },
    history::{
        Action,
        ActionTarget,
        ActionType,
    },
};
use slint::{
    ComponentHandle,
    SharedString,
    Weak,
};
use std::{
    cell::RefCell,
    rc::Rc,
};
use xxhash_rust::xxh3::xxh3_64;

// TODO: Update selected class, selected state

pub fn setup(ui_namemap: &mut GNameMap, data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) {
    ui_namemap.on_refresh(refresh(data.clone(), ui.clone()));
    ui_namemap.on_delete_class(delete_class_map(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_delete_state(delete_state_map(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_delete_event(delete_event_map(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_update_class_id(update_class_id(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_update_state_id(update_state_id(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_update_event_id(update_event_id(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_new_class(add_new_class(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_new_state(add_new_state(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_new_event(add_new_event(data.clone(), config.clone(), ui.clone()));
    ui_namemap.on_search_class(search_class_map(data.clone(), ui.clone()));
    ui_namemap.on_search_state(search_state_map(data.clone(), ui.clone()));
    ui_namemap.on_search_event(search_event_map(data.clone(), ui.clone()));
}

fn delete_class_map(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        let Some(&class_id) = data.class_name_map.get(name.as_str()) else {
            return;
        };
        if data.dialogues.contains_key(&class_id) {
            show_noti(
                &ui,
                NotiLevel::Error,
                format!("Class '{}' is still in used", name).as_str(),
            );
            return;
        }

        data.class_name_map.remove(name.as_str());
        reload_class_map(&data, &ui, "");

        config.history.add_undo(
            Action {
                action: ActionType::AddId(class_id),
                target: ActionTarget::NamemapClass(name.to_string()),
            },
            &ui,
        );
    }
}

fn delete_state_map(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        let Some(&state_id) = data.state_name_map.get(name.as_str()) else {
            return;
        };
        for (&class_id, class) in data.dialogues.iter() {
            if class.contains_key(&state_id) {
                show_noti(
                    &ui,
                    NotiLevel::Error,
                    format!(
                        "State '{}' is still in used in class '{}'",
                        name,
                        id_to_class(class_id, &data).unwrap_or_default()
                    )
                    .as_str(),
                );
                return;
            }
        }

        data.state_name_map.remove(name.as_str());
        reload_state_map(&data, &ui, "");

        config.history.add_undo(
            Action {
                action: ActionType::AddId(state_id),
                target: ActionTarget::NamemapState(name.to_string()),
            },
            &ui,
        );
    }
}

fn delete_event_map(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        let Some(&event_id) = data.event_name_map.get(name.as_str()) else {
            return;
        };
        for (&class_id, class) in data.dialogues.iter() {
            for (&state_id, state) in class.iter() {
                for (i, dialogue) in state.iter().enumerate() {
                    if dialogue.events.contains(&event_id) {
                        show_noti(
                            &ui,
                            NotiLevel::Error,
                            format!(
                                "Event '{}' is still in used in dialogue {} of class '{}', state '{}'",
                                name,
                                i,
                                id_to_class(class_id, &data).unwrap_or_default(),
                                id_to_state(state_id, &data).unwrap_or_default(),
                            )
                            .as_str(),
                        );
                        return;
                    }
                }
            }
        }

        data.event_name_map.remove(name.as_str());
        reload_event_map(&data, &ui, "");

        config.history.add_undo(
            Action {
                action: ActionType::AddId(event_id),
                target: ActionTarget::NamemapEvent(name.to_string()),
            },
            &ui,
        );
    }
}

fn update_class_id(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();
        let old_id = class_to_id(name.as_str(), &mut data);

        if let Ok(new_id) = update_id(&mut data, &ui, name.to_string(), old_id, id.as_str(), NameType::Class)
            && let Some(value) = data.dialogues.shift_remove(&old_id)
        {
            data.dialogues.insert(new_id, value);
            config.history.add_undo(
                Action {
                    action: ActionType::UpdateId(new_id, old_id),
                    target: ActionTarget::NamemapClass(name.to_string()),
                },
                &ui,
            );
        }
    }
}

fn update_state_id(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();
        let old_id = state_to_id(name.as_str(), &mut data);

        if let Ok(new_id) = update_id(&mut data, &ui, name.to_string(), old_id, id.as_str(), NameType::State) {
            for (_, class) in data.dialogues.iter_mut() {
                if let Some(value) = class.shift_remove(&old_id) {
                    class.insert(new_id, value);
                }
            }
            config.history.add_undo(
                Action {
                    action: ActionType::UpdateId(new_id, old_id),
                    target: ActionTarget::NamemapState(name.to_string()),
                },
                &ui,
            );
        }
    }
}

fn update_event_id(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();
        let old_id = event_to_id(name.as_str(), &mut data);

        if let Ok(new_id) = update_id(&mut data, &ui, name.to_string(), old_id, id.as_str(), NameType::Event) {
            for (_, class) in data.dialogues.iter_mut() {
                for (_, state) in class.iter_mut() {
                    for dialogue in state.iter_mut() {
                        dialogue.events.remove(&old_id);
                        dialogue.events.insert(new_id);
                    }
                }
            }
            config.history.add_undo(
                Action {
                    action: ActionType::UpdateId(new_id, old_id),
                    target: ActionTarget::NamemapEvent(name.to_string()),
                },
                &ui,
            );
        }
    }
}

fn add_new_class(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        if name.is_empty() {
            return;
        }

        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        if let Ok(new_id) = add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::Class) {
            reload_class_map(&data, &ui, "");
            config.history.add_undo(
                Action {
                    action: ActionType::DeleteId(new_id),
                    target: ActionTarget::NamemapClass(name.to_string()),
                },
                &ui,
            );
        }
    }
}

fn add_new_state(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        if name.is_empty() {
            return;
        }

        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        if let Ok(new_id) = add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::State) {
            reload_state_map(&data, &ui, "");

            config.history.add_undo(
                Action {
                    action: ActionType::DeleteId(new_id),
                    target: ActionTarget::NamemapState(name.to_string()),
                },
                &ui,
            );
        }
    }
}

fn add_new_event(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |name, id| {
        if name.is_empty() {
            return;
        }

        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        if let Ok(new_id) = add_new(&mut data, &ui, name.to_string(), id.as_str(), NameType::Event) {
            reload_event_map(&data, &ui, "");

            config.history.add_undo(
                Action {
                    action: ActionType::DeleteId(new_id),
                    target: ActionTarget::NamemapEvent(name.to_string()),
                },
                &ui,
            );
        }
    }
}

fn search_class_map(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |search| {
        let data = data.borrow();
        let ui = ui.unwrap();
        reload_class_map(&data, &ui, search.as_str());
    }
}

fn search_state_map(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |search| {
        let data = data.borrow();
        let ui = ui.unwrap();
        reload_state_map(&data, &ui, search.as_str());
    }
}
fn search_event_map(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |search| {
        let data = data.borrow();
        let ui = ui.unwrap();
        reload_event_map(&data, &ui, search.as_str());
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

fn refresh(data: Rc<RefCell<AppData>>, ui: Weak<AppWindow>) -> impl Fn() {
    move || {
        let data = data.borrow();
        let ui = ui.unwrap();
        reload_all_map(&data, &ui);
    }
}

pub fn reload_all_map(data: &AppData, ui: &AppWindow) {
    reload_class_map(data, ui, "");
    reload_state_map(data, ui, "");
    reload_event_map(data, ui, "");
}

pub fn reload_class_map(data: &AppData, ui: &AppWindow, search: &str) {
    let class_map = reload_map(data, NameType::Class, search);
    ui.global::<GNameMap>().set_classes(class_map.as_slice().into());

    let mut class_list = Vec::new();
    for class in data.class_name_map.keys() {
        class_list.push(ListItem {
            text: class.into(),
            ..Default::default()
        });
    }
    ui.global::<GContent>().set_class_list(class_list.as_slice().into());
}

pub fn reload_state_map(data: &AppData, ui: &AppWindow, search: &str) {
    let class_map = reload_map(data, NameType::State, search);
    ui.global::<GNameMap>().set_states(class_map.as_slice().into());

    let mut state_list = Vec::new();
    for state in data.state_name_map.keys() {
        state_list.push(ListItem {
            text: state.into(),
            ..Default::default()
        });
    }
    ui.global::<GContent>().set_state_list(state_list.as_slice().into());
}

fn reload_event_map(data: &AppData, ui: &AppWindow, search: &str) {
    let event_map = reload_map(data, NameType::Event, search);

    ui.global::<GNameMap>().set_events(event_map.as_slice().into());

    let mut event_list: Vec<ListItem> = Vec::new();
    for event in data.event_name_map.keys() {
        event_list.push(ListItem {
            text: event.into(),
            ..Default::default()
        });
    }
    ui.global::<GContent>().set_event_list(event_list.as_slice().into());
}

fn reload_map(data: &AppData, name_type: NameType, search: &str) -> Vec<StringId> {
    let data = match name_type {
        NameType::Class => &data.class_name_map,
        NameType::State => &data.state_name_map,
        NameType::Event => &data.event_name_map,
    };

    let mut ret = Vec::new();
    let re = new_regex(search);

    for (name, id) in data.iter() {
        if search.is_empty() || (re.is_ok() && re.as_ref().unwrap().is_match(name.as_str())) {
            ret.push(StringId {
                id: id.to_string().into(),
                name: name.clone().into(),
            })
        }
    }
    ret
}
