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
    // TODO: Namemap
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

#[derive(Default)]
pub struct History {
    pub undo_actions: Vec<Action>,
    pub redo_actions: Vec<Action>,
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
        self.undo_actions.push(action);
        self.redo_actions.clear();
        ui.set_undo_available(true);
        ui.set_redo_available(false);
    }
}

fn apply_action(action: &Action, data: &mut AppData) {
    match &action.action {
        ActionType::None => {}
        ActionType::Add(name) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(states) => {
                let class_id = class_to_id(name.as_str(), data);
                data.dialogues.insert(class_id, states.clone().unwrap_or_default());
            }
            ActionTarget::ContentState(class_id, dialogues) => {
                let state_id = state_to_id(name.as_str(), data);
                if let Some(class) = data.dialogues.get_mut(class_id) {
                    class.insert(state_id, dialogues.clone().unwrap_or_default());
                }
            }
        },
        ActionType::Delete(name) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(_) => {
                let class_id = class_to_id(name.as_str(), data);
                data.dialogues.remove(&class_id);
            }
            ActionTarget::ContentState(class_id, _) => {
                let state_id = state_to_id(name.as_str(), data);
                if let Some(class) = data.dialogues.get_mut(class_id) {
                    class.remove(&state_id);
                }
            }
        },
        ActionType::Update(old_name, new_name) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(_) => {
                let old_id = class_to_id(old_name, data);
                let new_id = class_to_id(new_name, data);
                if let Some(states) = data.dialogues.remove(&old_id) {
                    data.dialogues.insert(new_id, states);
                }
            }
            ActionTarget::ContentState(class_id, _) => {
                let old_id = state_to_id(old_name, data);
                let new_id = state_to_id(new_name, data);
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(dialogues) = class.remove(&old_id)
                {
                    class.insert(new_id, dialogues);
                }
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
