use crate::{
    AppData,
    AppWindow,
    Config,
    Dialogue,
    common::{
        class_to_id,
        state_to_id,
    },
    content_tab::reload_content,
    namemap_tab::reload_all_map,
};
use serde::{Deserialize, Serialize};
use slint::Weak;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
};

#[derive(Default, Clone)]
pub enum ActionType {
    #[default]
    None,
    Add(String),
    Delete(String),
    Update(String, String),
}

#[derive(Default, Clone)]
pub enum ActionTarget {
    #[default]
    None,
    ContentClass(Option<BTreeMap<u64, Vec<Dialogue>>>),
    ContentState(u64, Option<Vec<Dialogue>>),
    // TODO: Dialogue
    NamemapClass(String),
    NamemapState(String),
    NamemapEvent(String),
}

#[derive(Default)]
pub struct Action {
    pub action: ActionType,
    pub target: ActionTarget,
}

impl Action {
    pub fn get_reverse_action(&self) -> Action {
        let action = match &self.action {
            ActionType::None => ActionType::None,
            ActionType::Add(name) => ActionType::Delete(name.clone()),
            ActionType::Delete(name) => ActionType::Add(name.clone()),
            ActionType::Update(old_name, new_name) => ActionType::Update(new_name.clone(), old_name.clone()),
        };

        Action {
            action,
            target: self.target.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct History {
    #[serde(skip)]
    pub undo_actions: Vec<Action>,
    #[serde(skip)]
    pub redo_actions: Vec<Action>,
    /// Maximum undo actions
    pub limit: usize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            undo_actions: Vec::new(),
            redo_actions: Vec::new(),
            limit: 24,
        }
    }
}

impl History {
    pub fn undo(&mut self, data: &mut AppData) {
        let Some(undo_action) = self.undo_actions.pop() else {
            return;
        };
        apply_action(&undo_action, data);
        self.redo_actions.push(undo_action.get_reverse_action());
    }

    pub fn redo(&mut self, data: &mut AppData) {
        let Some(redo_action) = self.redo_actions.pop() else {
            return;
        };
        apply_action(&redo_action, data);
        self.undo_actions.push(redo_action.get_reverse_action());
    }

    pub fn add_undo(&mut self, action: Action, ui: &AppWindow) {
        if self.undo_actions.len() > self.limit {
            self.undo_actions.remove(0);
        }

        self.undo_actions.push(action);
        self.redo_actions.clear();
        ui.set_undo_available(true);
        ui.set_redo_available(false);
    }
}

fn apply_action(action: &Action, data: &mut AppData) {
    match &action.action {
        ActionType::None => {}
        ActionType::Add(value) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(states) => {
                let class_id = class_to_id(value.as_str(), data);
                data.dialogues.insert(class_id, states.clone().unwrap_or_default());
            }
            ActionTarget::ContentState(class_id, dialogues) => {
                let state_id = state_to_id(value.as_str(), data);
                if let Some(class) = data.dialogues.get_mut(class_id) {
                    class.insert(state_id, dialogues.clone().unwrap_or_default());
                }
            }
            ActionTarget::NamemapClass(class_name) => {
                let Ok(class_id) = value.parse() else {
                    return;
                };
                data.class_name_map.insert(class_name.clone(), class_id);
            }
            ActionTarget::NamemapState(state_name) => {
                let Ok(state_id) = value.parse() else {
                    return;
                };
                data.state_name_map.insert(state_name.clone(), state_id);
            }
            ActionTarget::NamemapEvent(event_name) => {
                let Ok(event_id) = value.parse() else {
                    return;
                };
                data.event_name_map.insert(event_name.clone(), event_id);
            }
        },
        ActionType::Delete(value) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(_) => {
                let class_id = class_to_id(value.as_str(), data);
                data.dialogues.remove(&class_id);
            }
            ActionTarget::ContentState(class_id, _) => {
                let state_id = state_to_id(value.as_str(), data);
                if let Some(class) = data.dialogues.get_mut(class_id) {
                    class.remove(&state_id);
                }
            }
            ActionTarget::NamemapClass(class_name) => {
                data.class_name_map.remove(class_name);
            }
            ActionTarget::NamemapState(state_name) => {
                data.state_name_map.remove(state_name);
            }
            ActionTarget::NamemapEvent(event_name) => {
                data.event_name_map.remove(event_name);
            }
        },
        ActionType::Update(old_value, new_value) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(_) => {
                let old_id = class_to_id(old_value, data);
                let new_id = class_to_id(new_value, data);
                if let Some(states) = data.dialogues.remove(&old_id) {
                    data.dialogues.insert(new_id, states);
                }
            }
            ActionTarget::ContentState(class_id, _) => {
                let old_id = state_to_id(old_value, data);
                let new_id = state_to_id(new_value, data);
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(dialogues) = class.remove(&old_id)
                {
                    class.insert(new_id, dialogues);
                }
            }
            ActionTarget::NamemapClass(class_name) => {
                let Ok(new_id) = new_value.parse() else {
                    return;
                };
                data.class_name_map.insert(class_name.clone(), new_id);
            }
            ActionTarget::NamemapState(state_name) => {
                let Ok(new_id) = new_value.parse() else {
                    return;
                };
                data.state_name_map.insert(state_name.clone(), new_id);
            }
            ActionTarget::NamemapEvent(event_name) => {
                let Ok(new_id) = new_value.parse() else {
                    return;
                };
                data.event_name_map.insert(event_name.clone(), new_id);
            }
        },
    }
}

pub fn undo(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) -> impl Fn() {
    move || {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        config.history.undo(&mut data);
        reload_content(&mut data, &ui, &config);
        reload_all_map(&data, &ui);

        if config.history.undo_actions.is_empty() {
            ui.set_undo_available(false);
        }
        ui.set_redo_available(true);
    }
}

pub fn redo(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) -> impl Fn() {
    move || {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        config.history.redo(&mut data);
        reload_content(&mut data, &ui, &config);
        reload_all_map(&data, &ui);

        if config.history.redo_actions.is_empty() {
            ui.set_redo_available(false);
        }
        ui.set_undo_available(true);
    }
}
