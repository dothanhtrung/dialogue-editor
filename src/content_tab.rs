pub mod class_ui;
pub mod dialogue_ui;
pub mod state_ui;

use crate::{
    Affect,
    AppData,
    AppWindow,
    Config,
    ContentLang,
    GContent,
    UiDialogue, common::{NameType, id_to_class, id_to_event, id_to_state},
};
use regex_lite::Regex;
use slint::{
    ComponentHandle,
    SharedString,
};
