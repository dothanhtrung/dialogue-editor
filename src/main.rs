// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bin_file;
mod ron_file;

use crate::{bin_file::BinFile, ron_file::RonFile};
use serde::{Deserialize, Serialize};
use slint::SharedString;
use std::{
    cell::{RefCell, RefMut},
    collections::{BTreeMap, HashMap},
    error::Error,
    path::{Path, PathBuf},
    rc::Rc,
};
use xxhash_rust::xxh3::xxh3_64;

slint::include_modules!();

#[derive(Serialize, Deserialize, Default, Clone)]
struct Dialogue {
    content: String,
    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    affect: Option<(u64, u64)>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AppData {
    dialogues: HashMap<u64, BTreeMap<u64, Vec<Dialogue>>>,
    class_name_map: HashMap<u64, String>,
    state_name_map: HashMap<u64, String>,
}

impl RonFile for AppData {}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    #[serde(default)]
    file_path: PathBuf,
    selected_class: u64,
    selected_state: u64,
}

impl RonFile for Config {}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let mut config = Config::default();
    let _ = config.load_from(Path::new("./dialog-editor.ron"));

    ui.set_file_path(config.file_path.to_str().unwrap_or_default().into());

    let data = AppData::default();

    let config = Rc::new(RefCell::new(config));
    let config_ref1 = config.clone();
    let config_ref2 = config.clone();

    let data = Rc::new(RefCell::new(data));
    let data_ref1 = data.clone();
    let data_ref2 = data.clone();
    let data_ref3 = data.clone();
    let data_ref4 = data.clone();
    let data_ref5 = data.clone();

    ui.on_request_load({
        // TODO: Loading icon
        let ui_handle = ui.as_weak();
        move |file_path| {
            let ui = ui_handle.unwrap();
            let mut config = config_ref1.borrow_mut();
            let mut data = data_ref1.borrow_mut();
            config.file_path = PathBuf::from(file_path.as_str());
            let _ = config.save_to(Path::new("./dialog-editor.ron"));

            if config.file_path.is_file() && data.load_from(&config.file_path).is_ok() {
                if (config.selected_class == 0
                    || !data.class_name_map.contains_key(&config.selected_class))
                    && let Some(first_class) = data.dialogues.keys().next()
                {
                    config.selected_class = *first_class;
                }

                if (config.selected_state == 0
                    || !data.state_name_map.contains_key(&config.selected_state))
                    && let Some(selected_class) = data.dialogues.get(&config.selected_class)
                    && let Some((first_state, _)) = selected_class.first_key_value()
                {
                    config.selected_class = *first_state;
                }

                reload(&mut data, &ui);
            }
        }
    });

    ui.on_request_save({
        // TODO: show save status
        move || {
            let config = config_ref2.borrow_mut();
            let data = data_ref2.borrow_mut();

            let _ = data.save_to(&config.file_path);
        }
    });

    ui.on_add_class({
        let ui_handle = ui.as_weak();
        move |class_name| {
            let ui = ui_handle.unwrap();
            let mut data = data_ref3.borrow_mut();
            let class_id = xxh3_64(class_name.as_bytes());
            data.class_name_map.insert(class_id, class_name.to_string());
            data.dialogues.insert(class_id, BTreeMap::new());

            reload(&mut data, &ui);
        }
    });

    ui.on_add_state({
        let ui_handle = ui.as_weak();
        move |state_name| {
            let ui = ui_handle.unwrap();
            let mut data = data_ref4.borrow_mut();
            let state_id = xxh3_64(state_name.as_bytes());
            data.state_name_map.insert(state_id, state_name.to_string());
        }
    });

    ui.run()?;

    Ok(())
}

fn reload(data: &mut RefMut<'_, AppData>, ui: &AppWindow) {
    let mut classes: Vec<SharedString> = Vec::new();
    for (_, name) in data.class_name_map.iter() {
        classes.push(name.into());
    }

    ui.set_classes(classes.as_slice().into());
}
