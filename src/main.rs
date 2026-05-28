// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bin_file;
mod ron_file;

use crate::{bin_file::BinFile, ron_file::RonFile};
use serde::{Deserialize, Serialize};
use slint::{ModelRc, SharedString, VecModel};
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    path::{Path, PathBuf},
    rc::Rc,
};

slint::include_modules!();

#[derive(Serialize, Deserialize, Default, Clone)]
struct Dialogue {
    content: String,
    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    affect: Option<(u32, u32)>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AppData {
    dialogues: HashMap<u32, BTreeMap<u32, Vec<Dialogue>>>,
    name_map: HashMap<u32, String>,
}

impl BinFile for AppData {}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    #[serde(default)]
    file_path: PathBuf,
}

impl RonFile for Config {}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let mut config = Config::default();
    let _ = config.load_from(Path::new("./dialog-editor.ron"));
    ui.set_file_path(config.file_path.to_str().unwrap_or_default().into());

    let mut data = AppData::default();

    ui.on_request_load({
        // let ui_handle = ui.as_weak();
        |file_path| {
            config.file_path = PathBuf::from(file_path.as_str());

            let mut classes: Vec<(i32, SharedString)> = Vec::new();
            if config.file_path.is_file() && data.load_from(&config.file_path).is_ok() {
                for class_id in data.dialogues.keys() {
                    let name = match data.name_map.get(class_id) {
                        Some(ret) => ret.clone(),
                        None => String::new(),
                    };
                    classes.push((*class_id as i32, name.into()));
                }
            }
            ui.set_classes(classes.as_slice().into());
            data.load_from(&config.file_path);
        }
    });

    ui.on_request_save({
        let _ = data.save_to(&config.file_path);
        move || {}
    });

    ui.run()?;

    Ok(())
}
