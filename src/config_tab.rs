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
    str::FromStr,
};

pub fn setup(ui_config: &mut GConfig, data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) {
    ui_config.on_add_lang(add_lang(config.clone(), ui.clone()));
    ui_config.on_delete_lang(delete_lang(data.clone(), config.clone(), ui.clone()));
    ui_config.on_set_max_undo(set_max_undo(config.clone()));
    ui_config.on_set_autosave_interval(set_autosave_interval(config.clone()));
    ui_config.on_set_save_without_namemap(set_save_without_namemap(config.clone()));

    let config = config.borrow();
    let ui = ui.unwrap();
    refresh(&config, &ui);
}

fn refresh(config: &Config, ui: &AppWindow) {
    ui.global::<GConfig>().set_max_undo(config.history.limit as i32);
    ui.global::<GConfig>().set_autosave_interval(config.autosave_interval as i32);
    ui.global::<GConfig>().set_save_without_namemap(config.save_without_namemap);

    reload_lang_list(config, ui);
}

fn add_lang(config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |lang| {
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        let langs: Vec<&str> = lang.split(" ").collect();

        for lang_str in langs.iter() {
            if lang_str.is_empty() {
                continue;
            }
            if let Ok(lang) = Language::from_str(lang_str) {
                if !config.langs.contains(&lang) {
                    config.langs.insert(lang);
                }
            } else {
                show_noti(
                    &ui,
                    NotiLevel::Error,
                    format!("Invalid language code: {}", lang_str).as_str(),
                );
                continue;
            }
        }

        reload_lang_list(&config, &ui);
        config.save();
    }
}

fn delete_lang(data: Rc<RefCell<AppData>>, config: Rc<RefCell<Config>>, ui: Weak<AppWindow>) -> impl Fn(SharedString) {
    move |lang| {
        let data = data.borrow();
        let mut config = config.borrow_mut();
        let ui = ui.unwrap();

        if let Ok(lang) = Language::from_str(lang.as_str()) {
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
        config.save();
    }
}

pub fn reload_lang_list(config: &Config, ui: &AppWindow) {
    let mut lang_list: Vec<SharedString> = Vec::new();
    for lang in config.langs.iter() {
        lang_list.push(lang.to_string().into());
    }

    ui.global::<GConfig>().set_lang_list(lang_list.as_slice().into());
}

fn set_max_undo(config: Rc<RefCell<Config>>) -> impl Fn(i32) {
    move |max_undo| {
        let mut config = config.borrow_mut();
        config.history.limit = max_undo as usize;
        config.save();
    }
}

fn set_autosave_interval(config: Rc<RefCell<Config>>) -> impl Fn(i32) {
    move |interval| {
        let mut config = config.borrow_mut();
        config.autosave_interval = interval as u32;
        config.save();
    }
}

fn set_save_without_namemap(config: Rc<RefCell<Config>>) -> impl Fn(bool) {
    move |save_without_namemap| {
        let mut config = config.borrow_mut();
        config.save_without_namemap = save_without_namemap;
        config.save();
    }
}