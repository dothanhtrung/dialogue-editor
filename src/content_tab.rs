pub mod class_ui;
pub mod dialogue_ui;
pub mod state_ui;

use crate::{
    AppData,
    AppWindow,
    Config,
    GContent,
    content_tab::{
        class_ui::reload_class,
        dialogue_ui::{
            reload_dialogue,
            reload_dialogue_detail,
        },
        state_ui::reload_state,
    },
};
use slint::{
    ComponentHandle,
    SharedString,
};

pub fn reload_content(data: &mut AppData, ui: &AppWindow, config: &Config) {
    reload_class(data, ui, "");
    reload_state(data, ui, config, "");
    reload_dialogue(data, ui, config, "");
    reload_dialogue_detail(data, ui, config);

    let mut lang_list: Vec<SharedString> = Vec::new();
    for lang in config.langs.iter() {
        lang_list.push(lang.to_639_3().to_string().into());
    }
    ui.global::<GContent>().set_lang_list(lang_list.as_slice().into());

    let mut event_list: Vec<SharedString> = Vec::new();
    for event in data.event_name_map.keys() {
        event_list.push(event.into());
    }
    ui.global::<GContent>().set_event_list(event_list.as_slice().into());

    // TODO: state list by affected class
    let mut state_list: Vec<SharedString> = Vec::new();
    for state in data.state_name_map.keys() {
        state_list.push(state.into());
    }
    ui.global::<GContent>().set_state_list(state_list.as_slice().into());
}
