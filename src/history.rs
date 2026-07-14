use crate::{
    AppData,
    Dialogue,
    common::{
        class_to_id,
        state_to_id,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::BTreeMap;

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
}

fn apply_action(action: &Action, data: &mut AppData) {
    match &action.action {
        ActionType::None => {}
        ActionType::Add(name) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(states) => {
                let id = class_to_id(name.as_str(), data);
                data.dialogues.insert(id, states.clone().unwrap_or_default());
            }
            ActionTarget::ContentState(class_id, dialogues) => {
                let state_id = state_to_id(name.as_str(), data);
                if let Some(class) = data.dialogues.get_mut(class_id) {
                    class.insert(state_id, dialogues.clone().unwrap_or_default());
                }
            }
        },
        ActionType::Delete(_) => todo!(),
        ActionType::Update(_, _) => todo!(),
    }
}
