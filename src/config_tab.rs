use crate::{
    AppData,
    AppWindow,
    Config,
    GConfig,
    common::{
        NotiLevel,
        show_noti,
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

// TODO: Why is this delay?
pub fn reload_config_ui(config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) -> impl Fn() {
    move || {
        let config = config.borrow();
        let ui = ui.unwrap();

        reload_lang_list(&config, &ui);
    }
}

pub fn add_lang(config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |lang| {
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        if let Some(lang) = Language::from_639_3(lang.as_str()) {
            if !config.langs.contains(&lang) {
                config.langs.insert(lang);
                reload_lang_list(&config, &ui);
            }
        } else {
            show_noti(
                &ui,
                NotiLevel::Error,
                format!("Invalid language code: {}", lang).as_str(),
            );
        }
    }
}

pub fn delete_lang(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui: Weak<AppWindow>,
) -> impl Fn(SharedString) {
    move |lang| {
        let data = data.borrow();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        if let Some(lang) = Language::from_639_3(lang.as_str()) {
            'outer: for (_, class) in data.dialogues.iter() {
                for (_, state) in class.iter() {
                    for dialogue in state.iter() {
                        if dialogue.contents.contains_key(&lang) {
                            show_noti(&ui, NotiLevel::Warn, "The language is still in used");
                            break 'outer;
                        }
                    }
                }
            }
            config.langs.remove(&lang);
            reload_lang_list(&config, &ui);
        } else {
            show_noti(
                &ui,
                NotiLevel::Warn,
                format!("Invalid language code: {}", lang).as_str(),
            );
        }
    }
}

pub fn reload_lang_list(config: &Config, ui: &AppWindow) {
    let mut lang_list: Vec<SharedString> = Vec::new();
    for lang in config.langs.iter() {
        lang_list.push(lang.to_string().into());
    }

    ui.global::<GConfig>().set_lang_list(lang_list.as_slice().into());
}
