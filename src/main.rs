// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod class_ui;
mod file_handle;
mod reload_ui;
mod state_ui;

use crate::{
    class_ui::{
        add_class,
        remove_class,
        rename_class,
        select_class,
    },
    state_ui::{
        add_state,
        remove_state,
        rename_state,
        select_state,
    },
};
use file_handle::*;
use isolang::Language;
use reload_ui::*;
use serde::{
    Deserialize,
    Serialize,
};
use slint::Model;
use std::{
    cell::RefCell,
    collections::{
        BTreeMap,
        HashMap,
    },
    error::Error,
    path::{
        Path,
        PathBuf,
    },
    rc::Rc,
};
use tracing_subscriber::EnvFilter;
use xxhash_rust::xxh3::xxh3_64;

slint::include_modules!();

#[derive(Serialize, Deserialize, Default, Clone)]
struct Dialogue {
    #[serde(default)]
    contents: BTreeMap<Language, String>,
    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    affects: BTreeMap<u64, u64>,
    #[serde(default)]
    events: Vec<u64>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AppData {
    #[serde(default)]
    dialogues: HashMap<u64, BTreeMap<u64, Vec<Dialogue>>>,
    #[serde(default)]
    class_name_map: HashMap<u64, String>,
    #[serde(default)]
    state_name_map: HashMap<u64, String>,
    #[serde(default)]
    event_name_map: HashMap<u64, String>,
}

// TODO: Config UI
#[derive(Serialize, Deserialize, Default)]
struct Config {
    #[serde(default)]
    file_path: PathBuf,
    #[serde(default)]
    selected_class: u64,
    #[serde(default)]
    selected_state: u64,
    #[serde(default)]
    selected_dialog: usize,
    #[serde(default)]
    /// Used when file_format is Bin
    encrypt_key: String,
    #[serde(default)]
    file_format: FileFormat,
    #[serde(default)]
    langs: Vec<Language>,
}

impl Config {
    fn save(&self) {
        let _ = ron_file::save_to::<Config>(self, Path::new("./dialog-editor.ron"));
    }
}

#[derive(Default)]
struct DataCache {
    name_map: HashMap<String, u64>,
}

// TODO: Warning to save before exit

fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_thread_ids(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let ui = AppWindow::new()?;
    let cache = DataCache::default();
    let config: Config = ron_file::load_from(Path::new("./dialog-editor.ron")).unwrap_or_default();

    let data = AppData::default();

    ui.set_file_path(config.file_path.to_str().unwrap_or_default().into());
    ui.set_encrypt_key(config.encrypt_key.as_str().into());
    ui.set_file_format(config.file_format as i32);

    #[cfg(feature = "crypt")]
    ui.set_enable_crypt(true);

    let cache = Rc::new(RefCell::new(cache));
    let config = Rc::new(RefCell::new(config));
    let data = Rc::new(RefCell::new(data));

    ui.on_file_picker(file_picker(config.clone(), ui.as_weak()));
    ui.on_request_load(request_load(data.clone(), config.clone(), ui.as_weak()));
    ui.on_request_save(request_save(data.clone(), config.clone(), ui.as_weak()));

    ui.on_add_class(add_class(data.clone(), config.clone(), cache.clone(), ui.as_weak()));
    ui.on_rename_class(rename_class(data.clone(), config.clone(), cache.clone(), ui.as_weak()));
    ui.on_remove_class(remove_class(data.clone(), cache.clone(), ui.as_weak()));
    ui.on_select_class(select_class(data.clone(), config.clone(), cache.clone(), ui.as_weak()));

    ui.on_add_state(add_state(data.clone(), config.clone(), cache.clone(), ui.as_weak()));
    ui.on_select_state(select_state(data.clone(), config.clone(), cache.clone(), ui.as_weak()));
    ui.on_remove_state(remove_state(data.clone(), config.clone(), cache.clone(), ui.as_weak()));
    ui.on_rename_state(rename_state(data.clone(), config.clone(), cache.clone(), ui.as_weak()));

    ui.on_add_dialog({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        move || {
            let ui = ui_handle.unwrap();
            let mut data = data.borrow_mut();
            let mut config = config.borrow_mut();

            if let Some(state_list) = data.dialogues.get_mut(&config.selected_class)
                && let Some(dialogues) = state_list.get_mut(&config.selected_state)
            {
                dialogues.push(Dialogue::default());
                config.selected_dialog = dialogues.len() - 1;
                reload_dialogue(&data, &ui, &config);
            }
        }
    });

    ui.on_select_dialog({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        move |dialog_id| {
            let ui = ui_handle.unwrap();
            let data = data.borrow_mut();
            let mut config = config.borrow_mut();

            config.selected_dialog = dialog_id as usize;
            reload_dialogue_detail(&data, &ui, &config);
        }
    });

    ui.on_remove_dialog({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        move |dialog_id| {
            let dialog_id = dialog_id as usize;
            let ui = ui_handle.unwrap();
            let mut data = data.borrow_mut();
            let config = config.borrow();
            if let Some(class) = data.dialogues.get_mut(&config.selected_class)
                && let Some(state) = class.get_mut(&config.selected_state)
                && dialog_id < state.len()
            {
                state.remove(dialog_id);
                reload_dialogue(&data, &ui, &config);
            }
        }
    });

    ui.on_new_lang_content({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        move |lang, content| {
            let ui = ui_handle.unwrap();
            let mut data = data.borrow_mut();
            let config = config.borrow();

            if let Some(class) = data.dialogues.get_mut(&config.selected_class)
                && let Some(state) = class.get_mut(&config.selected_state)
                && let Some(dialogue) = state.get_mut(config.selected_dialog)
            {
                let lang = Language::from_639_3(lang.as_str()).unwrap_or_default();
                dialogue.contents.insert(lang, content.to_string());
                reload_dialogue_detail(&data, &ui, &config);
            }
        }
    });

    ui.on_new_affect({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        let cache = cache.clone();
        move |class_name, state_name| {
            let ui = ui_handle.unwrap();
            let mut data = data.borrow_mut();
            let config = config.borrow();
            let mut cache = cache.borrow_mut();
            if let Some(class) = data.dialogues.get_mut(&config.selected_class)
                && let Some(state) = class.get_mut(&config.selected_state)
                && let Some(dialogue) = state.get_mut(config.selected_dialog)
            {
                let class_id = string_to_id(class_name.as_str(), &mut cache);
                let state_id = string_to_id(state_name.as_str(), &mut cache);

                dialogue.affects.insert(class_id, state_id);
                reload_dialogue_detail(&data, &ui, &config);
            }
        }
    });

    ui.on_update_content({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        move |ui_dialogue| {
            let ui = ui_handle.unwrap();
            let mut data = data.borrow_mut();
            let config = config.borrow();

            if let Some(class) = data.dialogues.get_mut(&config.selected_class)
                && let Some(state) = class.get_mut(&config.selected_state)
                && let Some(dialogue) = state.get_mut(config.selected_dialog)
            {
                *dialogue = Dialogue::from(ui_dialogue);
                reload_dialogue_detail(&data, &ui, &config);
            }
        }
    });

    ui.on_delete_content({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        move |lang| {
            let ui = ui_handle.unwrap();
            let mut data = data.borrow_mut();
            let config = config.borrow();

            if let Some(class) = data.dialogues.get_mut(&config.selected_class)
                && let Some(state) = class.get_mut(&config.selected_state)
                && let Some(dialogue) = state.get_mut(config.selected_dialog)
            {
                dialogue
                    .contents
                    .remove(&Language::from_639_3(lang.to_string().as_str()).unwrap_or_default());
            }
            reload_dialogue_detail(&data, &ui, &config);
        }
    });

    ui.on_delete_affect({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        let cache = cache.clone();
        move |class_name| {
            let ui = ui_handle.unwrap();
            let mut data = data.borrow_mut();
            let config = config.borrow();
            let mut cache = cache.borrow_mut();

            if let Some(class) = data.dialogues.get_mut(&config.selected_class)
                && let Some(state) = class.get_mut(&config.selected_state)
                && let Some(dialogue) = state.get_mut(config.selected_dialog)
            {
                let class_id = string_to_id(class_name.as_str(), &mut cache);
                dialogue.affects.remove(&class_id);
            }
            reload_dialogue_detail(&data, &ui, &config);
        }
    });

    ui.on_search({
        let ui_handle = ui.as_weak();
        let data = data.clone();
        let config = config.clone();
        move |search_class, search_state| {
            let ui = ui_handle.unwrap();
            let data = data.borrow();
            let mut config = config.borrow_mut();
            reload_all(&data, &ui, &config, search_class.as_str(), search_state.as_str());

            if !search_class.is_empty() {
                config.selected_class = 0;
                config.selected_state = 0;
            } else if !search_state.is_empty() {
                config.selected_state = 0;
            }
        }
    });

    ui.run()?;

    Ok(())
}

impl From<UiDialogue> for Dialogue {
    fn from(ui_dialog: UiDialogue) -> Self {
        let mut ret = Self::default();
        for affect in ui_dialog.affects.iter() {
            let class_id = xxh3_64(affect.class.to_lowercase().as_bytes());
            let state_id = xxh3_64(affect.state.to_lowercase().as_bytes());
            ret.affects.insert(class_id, state_id);
        }
        for content in ui_dialog.contents.iter() {
            ret.contents.insert(
                Language::from_639_3(content.language.as_str()).unwrap_or_default(),
                content.content.to_string(),
            );
        }
        ret
    }
}
