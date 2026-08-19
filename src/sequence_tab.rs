use crate::{
    AppData,
    AppWindow,
    Config,
    GSequence,
    SequenceInfo,
    SequenceItem,
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
    slint_generatedAppWindow::UiSequenceItem,
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

pub fn setup(ui_content: &mut GSequence, data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) {
    ui_content.on_select(select(data.clone(), config.clone(), ui.clone()));
    ui_content.on_rename(rename(data.clone(), config.clone(), ui.clone()));
    ui_content.on_add(add(data.clone(), config.clone(), ui.clone()));
    ui_content.on_delete(delete(data.clone(), config.clone(), ui.clone()));
    ui_content.on_search(search(data.clone(), config.clone(), ui.clone()));
    ui_content.on_add_item(add_item(data.clone(), config.clone(), ui.clone()));
    ui_content.on_delete_item(delete_item(data.clone(), config.clone(), ui.clone()));
}

fn select(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui_handle.unwrap();

        let sequence_id = name_to_id(&name, &mut data.sequence_name_map);
        config.selected_sequence = sequence_id;

        reload_sequence_items(&mut data, &ui, sequence_id);
    }
}

fn rename(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |old_name, new_name| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui_handle.unwrap();

        let old_id = name_to_id(&old_name, &mut data.sequence_name_map);
        let new_id = name_to_id(&new_name, &mut data.sequence_name_map);

        if let Some(sequence) = data.sequences.shift_remove(&old_id) {
            data.sequences.insert(new_id, sequence);
            reload_sequence(&mut data, &ui, "");

            config.history.add_undo(
                Action {
                    action: ActionType::UpdateStr(new_name.to_string(), old_name.to_string()),
                    target: ActionTarget::Sequence(None),
                },
                &ui,
            );
        }
    }
}

fn add(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui_handle: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui_handle.unwrap();

        let sequence_id = name_to_id(&name, &mut data.sequence_name_map);
        if data.sequences.contains_key(&sequence_id) {
            show_noti(
                &ui,
                NotiLevel::Error,
                format!("Sequence {} already exists", name).as_str(),
            );
        } else {
            data.sequences.insert(sequence_id, Vec::new());
            config.selected_sequence = sequence_id;

            config.history.add_undo(
                Action {
                    action: ActionType::DeleteStr(name.to_string()),
                    target: ActionTarget::Sequence(None),
                },
                &ui,
            );

            ui.global::<GSequence>().set_selected_sequence(name);
            reload_sequence(&mut data, &ui, "");
        }
    }
}

fn delete(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |name| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui_handle.unwrap();

        let sequence_id = name_to_id(&name, &mut data.sequence_name_map);
        if let Some(_item) = data.sequences.shift_remove(&sequence_id) {
            config.selected_sequence = 0;
            ui.global::<GSequence>().set_selected_sequence(SharedString::new());
            reload_sequence(&mut data, &ui, "");

            config.history.add_undo(
                Action {
                    action: ActionType::AddStr(name.to_string()),
                    target: ActionTarget::Sequence(None),
                },
                &ui,
            );
        } else {
            show_noti(
                &ui,
                NotiLevel::Error,
                format!("Sequence {} does not exist", name).as_str(),
            );
        }
    }
}

fn search(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |search| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui_handle.unwrap();

        config.selected_sequence = 0;
        ui.global::<GSequence>().set_selected_sequence(SharedString::new());
        reload_sequence(&mut data, &ui, &search);
    }
}

fn add_item(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString, SharedString) {
    move |class_name, state_name, dialogue_pos| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui_handle.unwrap();

        let class_id = name_to_id(&class_name, &mut data.class_name_map);
        let state_id = name_to_id(&state_name, &mut data.state_name_map);
        let dialogue_pos =
            if dialogue_pos.is_empty() { None } else { Some(dialogue_pos.parse::<usize>().unwrap_or(0)) };
        let selected_sequence = config.selected_sequence;

        if let Some(sequence) = data.sequences.get_mut(&config.selected_sequence) {
            let item = SequenceItem {
                class: class_id,
                state: state_id,
                dialogue: dialogue_pos,
            };
            sequence.push(item);

            config.history.add_undo(
                Action {
                    action: ActionType::DeleteId(selected_sequence),
                    target: ActionTarget::SequenceItem(sequence.len() - 1, None),
                },
                &ui,
            );
        }
        reload_sequence_items(&mut data, &ui, config.selected_sequence);
    }
}

fn delete_item(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui_handle: Weak<AppWindow>) -> impl Fn(i32) {
    move |item_index| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui_handle.unwrap();
        let selected_sequence = config.selected_sequence;

        if let Some(sequence) = data.sequences.get_mut(&config.selected_sequence)
            && item_index >= 0
            && (item_index as usize) < sequence.len()
        {
            let item = sequence.remove(item_index as usize);
            config.history.add_undo(
                Action {
                    action: ActionType::AddId(selected_sequence),
                    target: ActionTarget::SequenceItem(item_index as usize, Some(item)),
                },
                &ui,
            );
        }
        reload_sequence_items(&mut data, &ui, config.selected_sequence);
    }
}

pub fn reload_sequence(data: &mut AppData, ui: &AppWindow, search: &str) {
    let mut sequence_list: Vec<SharedString> = Vec::new();
    let re = new_regex(search);

    for sequence_id in data.sequences.keys() {
        if let Some(sequence_name) = id_to_name(*sequence_id, &data.sequence_name_map) {
            if search.is_empty() {
                sequence_list.push(sequence_name.into());
            } else if let Ok(re) = re.as_ref()
                && re.is_match(sequence_name.as_str())
            {
                sequence_list.push(sequence_name.into());
            }
        }
    }
    ui.global::<GSequence>().set_sequences(sequence_list.as_slice().into());
    ui.global::<GSequence>().set_items([].into());
    ui.global::<GSequence>().set_info(SequenceInfo::default());
}

pub fn reload_sequence_items(data: &mut AppData, ui: &AppWindow, sequence_id: u64) {
    if let Some(sequence) = data.sequences.get(&sequence_id) {
        let mut items = Vec::new();
        for item in sequence.iter() {
            let ui_item = UiSequenceItem {
                class: id_to_name(item.class, &data.class_name_map).unwrap_or_default().into(),
                state: id_to_name(item.state, &data.state_name_map).unwrap_or_default().into(),
                dialogue: if let Some(dialogue_pos) = item.dialogue {
                    dialogue_pos.to_string().into()
                } else {
                    SharedString::from("None")
                },
            };
            items.push(ui_item);
        }
        ui.global::<GSequence>().set_items(items.as_slice().into());

        let name = id_to_name(sequence_id, &data.sequence_name_map)
            .unwrap_or_default()
            .into();
        let info = SequenceInfo {
            name,
            id: sequence_id.to_string().into(),
        };
        ui.global::<GSequence>().set_info(info);
    }
}
