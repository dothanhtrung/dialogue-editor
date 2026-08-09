pub mod class_ui;
pub mod dialogue_ui;
pub mod state_ui;

use crate::{
    AppData,
    AppWindow,
    Config,
    GContent,
    ListItem,
    common::{
        class_to_id,
        state_to_id,
    },
};
use class_ui::*;
use dialogue_ui::*;
use slint::{
    ComponentHandle,
    SharedString,
    Weak,
};
use state_ui::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn setup(ui_content: &mut GContent, data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) {
    ui_content.on_refresh(refresh(data.clone(), config.clone(), ui.clone()));
    ui_content.on_add_class(add_class(data.clone(), config.clone(), ui.clone()));
    ui_content.on_rename_class(rename_class(data.clone(), config.clone(), ui.clone()));
    ui_content.on_remove_class(remove_class(data.clone(), config.clone(), ui.clone()));
    ui_content.on_select_class(select_class(data.clone(), config.clone(), ui.clone()));

    ui_content.on_add_state(add_state(data.clone(), config.clone(), ui.clone()));
    ui_content.on_select_state(select_state(data.clone(), config.clone(), ui.clone()));
    ui_content.on_remove_state(remove_state(data.clone(), config.clone(), ui.clone()));
    ui_content.on_rename_state(rename_state(data.clone(), config.clone(), ui.clone()));

    ui_content.on_add_dialog(add_dialogue(data.clone(), config.clone(), ui.clone()));
    ui_content.on_select_dialog(select_dialogue(data.clone(), config.clone(), ui.clone()));
    ui_content.on_remove_dialog(remove_dialogue(data.clone(), config.clone(), ui.clone()));
    ui_content.on_add_lang_content(add_lang_content(data.clone(), config.clone(), ui.clone()));
    ui_content.on_add_affect(add_affect(data.clone(), config.clone(), ui.clone()));
    ui_content.on_update_content(update_content(data.clone(), config.clone(), ui.clone()));
    ui_content.on_delete_content(delete_content(data.clone(), config.clone(), ui.clone()));
    ui_content.on_delete_affect(delete_affect(data.clone(), config.clone(), ui.clone()));
    ui_content.on_add_event(add_event(data.clone(), config.clone(), ui.clone()));
    ui_content.on_delete_event(delete_event(data.clone(), config.clone(), ui.clone()));
    ui_content.on_search_class(search_class(data.clone(), ui.clone()));
    ui_content.on_search_state(search_state(data.clone(), config.clone(), ui.clone()));
    ui_content.on_search_dialogue(search_dialogue(data.clone(), config.clone(), ui.clone()));

    ui_content.on_goto(goto(data.clone(), config.clone(), ui.clone()));
}

fn refresh(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) -> impl Fn() {
    move || {
        let mut data = data.borrow_mut();
        let config = config.borrow();
        let ui = ui.unwrap();

        reload_content(&mut data, &ui, &config);
    }
}

fn goto(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString) {
    move |class_name, state_name| {
        let mut data = data.borrow_mut();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        config.selected_class = class_to_id(class_name.as_str(), &mut data);
        config.selected_state = state_to_id(&state_name, &mut data);
        config.selected_dialogue = 0;

        ui.global::<GContent>().set_selecting_class(class_name);
        ui.global::<GContent>().set_selecting_state(state_name);
        ui.global::<GContent>().set_selecting_dialogue(0);

        reload_content(&mut data, &ui, &config);
    }
}

pub fn reload_content(data: &mut AppData, ui: &AppWindow, config: &Config) {
    reload_class(data, ui, "");
    reload_state(data, ui, config, "");
    reload_dialogue(data, ui, config, "");
    reload_dialogue_detail(data, ui, config);

    let mut event_list = Vec::new();
    for event in data.event_name_map.keys() {
        event_list.push(ListItem {
            text: event.into(),
            ..Default::default()
        });
    }
    ui.global::<GContent>().set_event_list(event_list.as_slice().into());

    // TODO: state list by affected class
    let mut state_list = Vec::new();
    for state in data.state_name_map.keys() {
        state_list.push(ListItem {
            text: state.into(),
            ..Default::default()
        });
    }
    ui.global::<GContent>().set_state_list(state_list.as_slice().into());

    let mut class_list = Vec::new();
    for class in data.class_name_map.keys() {
        class_list.push(ListItem {
            text: class.into(),
            ..Default::default()
        });
    }
    ui.global::<GContent>().set_class_list(class_list.as_slice().into());
}
