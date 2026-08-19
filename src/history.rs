use crate::{
    AppData,
    AppWindow,
    Config,
    Dialogue,
    SequenceItem,
    common::name_to_id,
    content_tab::{
        class_ui::update_class,
        reload_content,
        state_ui::update_state,
    },
    namemap_tab::reload_all_map,
    sequence_tab::{
        reload_sequence,
        reload_sequence_items,
    },
};
use indexmap::IndexMap;
use isolang::Language;
use serde::{
    Deserialize,
    Serialize,
};
use slint::Weak;
use std::{
    cell::RefCell,
    rc::Rc,
};
use tracing::warn;

#[derive(Default, Clone)]
pub enum ActionType {
    #[default]
    None,
    AddStr(String),
    DeleteStr(String),
    UpdateStr(String, String),
    AddId(u64),
    DeleteId(u64),
    UpdateId(u64, u64),
}

#[derive(Default, Clone)]
pub enum ActionTarget {
    #[default]
    None,
    ContentClass(Option<IndexMap<u64, Vec<Dialogue>>>),
    ContentState(u64, Option<Vec<Dialogue>>),
    ContentDialogue(u64, u64, usize, Dialogue),
    ContentLang(u64, u64, usize, Language),
    ContentAffect(u64, u64, usize, u64),
    ContentEvent(u64, u64, usize),
    Sequence(Option<Vec<SequenceItem>>),
    SequenceItem(usize, Option<SequenceItem>),
    NamemapClass(String),
    NamemapState(String),
    NamemapSequence(String),
    NamemapEvent(String),
}

#[derive(Default)]
pub struct Action {
    pub action: ActionType,
    pub target: ActionTarget,
}

impl Action {
    pub fn reverse(mut self) -> Self {
        self.action = match self.action {
            ActionType::None => ActionType::None,
            ActionType::AddStr(value) => ActionType::DeleteStr(value),
            ActionType::DeleteStr(value) => ActionType::AddStr(value),
            ActionType::UpdateStr(old_value, new_value) => ActionType::UpdateStr(new_value, old_value),
            ActionType::AddId(id) => ActionType::DeleteId(id),
            ActionType::DeleteId(id) => ActionType::AddId(id),
            ActionType::UpdateId(old_id, new_id) => ActionType::UpdateId(new_id, old_id),
        };
        self
    }
}

#[derive(Serialize, Deserialize)]
pub struct History {
    #[serde(skip)]
    pub undo_actions: Vec<Action>,
    #[serde(skip)]
    pub redo_actions: Vec<Action>,
    /// Maximum undo actions
    pub limit: usize, // TODO: make this configurable
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
        self.redo_actions.push(undo_action.reverse());
    }

    pub fn redo(&mut self, data: &mut AppData) {
        let Some(redo_action) = self.redo_actions.pop() else {
            return;
        };
        apply_action(&redo_action, data);
        self.undo_actions.push(redo_action.reverse());
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
        ActionType::AddStr(value) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(states) => {
                let class_id = name_to_id(value.as_str(), &mut data.class_name_map);
                data.dialogues.insert(class_id, states.clone().unwrap_or_default());
            }
            ActionTarget::ContentState(class_id, dialogues) => {
                let state_id = name_to_id(value.as_str(), &mut data.state_name_map);
                if let Some(class) = data.dialogues.get_mut(class_id) {
                    class.insert(state_id, dialogues.clone().unwrap_or_default());
                }
            }
            ActionTarget::ContentDialogue(class_id, state_id, _, dialogue) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state.push(dialogue.clone());
                }
            }
            ActionTarget::ContentLang(class_id, state_id, dialogue_pos, language) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state[*dialogue_pos].contents.insert(*language, value.clone());
                }
            }
            ActionTarget::Sequence(sequence) => {
                if let Some(sequence) = sequence {
                    let sequence_id = name_to_id(value.as_str(), &mut data.sequence_name_map);
                    data.sequences.insert(sequence_id, sequence.clone());
                }
            }

            _ => {}
        },
        ActionType::DeleteStr(value) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(_) => {
                let class_id = name_to_id(value.as_str(), &mut data.class_name_map);
                data.dialogues.shift_remove(&class_id);
            }
            ActionTarget::ContentState(class_id, _) => {
                let state_id = name_to_id(value.as_str(), &mut data.state_name_map);
                if let Some(class) = data.dialogues.get_mut(class_id) {
                    class.shift_remove(&state_id);
                }
            }
            ActionTarget::ContentDialogue(class_id, state_id, dialogue_pos, _) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state.remove(*dialogue_pos);
                }
            }
            ActionTarget::ContentLang(class_id, state_id, dialogue_pos, language) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state[*dialogue_pos].contents.shift_remove(language);
                }
            }
            ActionTarget::Sequence(_) => {
                let sequence_id = name_to_id(value.as_str(), &mut data.sequence_name_map);
                data.sequences.shift_remove(&sequence_id);
            }

            _ => {}
        },
        ActionType::UpdateStr(old_value, new_value) => match &action.target {
            ActionTarget::None => {}
            ActionTarget::ContentClass(_) => {
                let old_id = name_to_id(old_value, &mut data.class_name_map);
                let new_id = name_to_id(new_value, &mut data.class_name_map);
                let _ = update_class(data, old_id, new_id);
            }
            ActionTarget::ContentState(class_id, _) => {
                let old_id = name_to_id(old_value, &mut data.state_name_map);
                let new_id = name_to_id(new_value, &mut data.state_name_map);
                let _ = update_state(data, *class_id, old_id, new_id);
            }
            ActionTarget::ContentDialogue(_, _, _, _) => {
                warn!("Undo/Redo: Replacing whole dialogue is not supported");
            }
            ActionTarget::ContentLang(class_id, state_id, dialogue_pos, lang) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state[*dialogue_pos].contents.insert(*lang, new_value.clone());
                }
            }
            ActionTarget::Sequence(_) => {
                let old_id = name_to_id(old_value, &mut data.sequence_name_map);
                let new_id = name_to_id(new_value, &mut data.sequence_name_map);
                if let Some(sequence) = data.sequences.shift_remove(&old_id) {
                    data.sequences.insert(new_id, sequence);
                }
            }

            _ => {}
        },
        ActionType::AddId(value) => match &action.target {
            ActionTarget::ContentAffect(class_id, state_id, dialogue_pos, affect_class) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state[*dialogue_pos].affects.insert(*affect_class, *value);
                }
            }
            ActionTarget::ContentEvent(class_id, state_id, dialogue_pos) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state[*dialogue_pos].events.insert(*value);
                }
            }
            ActionTarget::SequenceItem(index, item) => {
                if let Some(sequence) = data.sequences.get_mut(value)
                    && let Some(item) = item
                {
                    sequence.insert(*index, item.clone());
                }
            }
            ActionTarget::NamemapClass(class_name) => {
                data.class_name_map.insert(class_name.clone(), *value);
            }
            ActionTarget::NamemapState(state_name) => {
                data.state_name_map.insert(state_name.clone(), *value);
            }
            ActionTarget::NamemapEvent(event_name) => {
                data.event_name_map.insert(event_name.clone(), *value);
            }
            ActionTarget::NamemapSequence(sequence_name) => {
                data.sequence_name_map.insert(sequence_name.clone(), *value);
            }
            _ => {}
        },
        ActionType::DeleteId(value) => match &action.target {
            ActionTarget::ContentAffect(class_id, state_id, dialogue_pos, affect_class) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state[*dialogue_pos].affects.remove(affect_class);
                }
            }
            ActionTarget::ContentEvent(class_id, state_id, dialogue_pos) => {
                if let Some(class) = data.dialogues.get_mut(class_id)
                    && let Some(state) = class.get_mut(state_id)
                {
                    state[*dialogue_pos].events.remove(value);
                }
            }
            ActionTarget::SequenceItem(index, _) => {
                if let Some(sequence) = data.sequences.get_mut(value) {
                    sequence.remove(*index);
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
            ActionTarget::NamemapSequence(sequence_name) => {
                data.sequence_name_map.remove(sequence_name);
            }
            _ => {}
        },
        ActionType::UpdateId(_, new_id) => match &action.target {
            ActionTarget::NamemapClass(class_name) => {
                data.class_name_map.insert(class_name.clone(), *new_id);
            }
            ActionTarget::NamemapState(state_name) => {
                data.state_name_map.insert(state_name.clone(), *new_id);
            }
            ActionTarget::NamemapEvent(event_name) => {
                data.event_name_map.insert(event_name.clone(), *new_id);
            }
            ActionTarget::NamemapSequence(sequence_name) => {
                data.sequence_name_map.insert(sequence_name.clone(), *new_id);
            }
            _ => {}
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
        reload_sequence(&mut data, &ui, "");
        reload_sequence_items(&mut data, &ui, config.selected_sequence);

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
        reload_sequence(&mut data, &ui, "");
        reload_sequence_items(&mut data, &ui, config.selected_sequence);

        if config.history.redo_actions.is_empty() {
            ui.set_redo_available(false);
        }
        ui.set_undo_available(true);
    }
}
