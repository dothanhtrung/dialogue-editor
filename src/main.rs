// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bin_file;
mod ron_file;

use isolang::Language;
use regex_lite::Regex;
use rfd::FileDialog;
use serde::{
    Deserialize,
    Serialize,
};
use slint::{
    Model,
    SharedString,
};
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
use tracing::error;
use tracing_subscriber::EnvFilter;
use xxhash_rust::xxh3::xxh3_64;

slint::include_modules!();

#[derive(Serialize, Deserialize, Default, Clone)]
struct Dialogue {
    contents: BTreeMap<Language, String>,
    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    affects: BTreeMap<u64, u64>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AppData {
    dialogues: HashMap<u64, BTreeMap<u64, Vec<Dialogue>>>,
    class_name_map: HashMap<u64, String>,
    state_name_map: HashMap<u64, String>,
}

#[repr(i32)]
#[derive(Default, Serialize, Deserialize, Clone, Copy)]
enum FileFormat {
    #[default]
    Ron = 0,
    Bin,
}

impl From<i32> for FileFormat {
    fn from(number: i32) -> Self {
        match number {
            0 => Self::Ron,
            1 => Self::Bin,
            _ => Self::Ron,
        }
    }
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

    {
        ui.on_file_picker({
            let config = config.clone();
            let ui_handle = ui.as_weak();
            move || {
                let ui = ui_handle.unwrap();
                let mut config = config.borrow_mut();
                config.file_path = FileDialog::new()
                    .set_directory(
                        config
                            .file_path
                            .parent()
                            .unwrap_or(Path::new("/"))
                            .to_str()
                            .unwrap_or("/"),
                    )
                    .pick_file()
                    .unwrap_or_default();
                ui.set_file_path(config.file_path.to_str().unwrap_or_default().into());

                // Poor implementation but good enough
                let Some(ext) = config.file_path.extension() else {
                    return;
                };
                let Some(ext) = ext.to_str() else {
                    return;
                };
                let ext = ext.to_string();
                if ext.eq_ignore_ascii_case("ron") {
                    config.file_format = FileFormat::Ron;
                } else {
                    config.file_format = FileFormat::Bin;
                }
                ui.set_file_format(config.file_format as i32);
            }
        });

        ui.on_request_load({
            // TODO: Loading icon
            // TODO: Warn if there is unsave content
            let data = data.clone();
            let config = config.clone();
            let ui_handle = ui.as_weak();
            move |file_path, file_format, encrypt_key| {
                let ui = ui_handle.unwrap();
                let mut config = config.borrow_mut();
                let mut data = data.borrow_mut();

                config.file_format = file_format.into();
                config.encrypt_key = encrypt_key.to_string();
                config.file_path = PathBuf::from(file_path.as_str());
                config.save();

                if config.file_path.is_file() {
                    // TODO: Noti if fail to load
                    *data = match config.file_format {
                        FileFormat::Bin => {
                            bin_file::load_from(&config.file_path, &config.encrypt_key).unwrap_or_default()
                        }
                        FileFormat::Ron => ron_file::load_from(&config.file_path).unwrap_or_default(),
                    };
                    if (config.selected_class == 0 || !data.class_name_map.contains_key(&config.selected_class))
                        && let Some(first_class) = data.dialogues.keys().next()
                    {
                        config.selected_class = *first_class;
                    }

                    if (config.selected_state == 0 || !data.state_name_map.contains_key(&config.selected_state))
                        && let Some(selected_class) = data.dialogues.get(&config.selected_class)
                        && let Some((first_state, _)) = selected_class.first_key_value()
                    {
                        config.selected_class = *first_state;
                    }

                    reload_all(&data, &ui, &config, "", "");
                    ui.set_is_saved(true);
                }
            }
        });

        ui.on_request_save({
            // TODO: show save status
            let data = data.clone();
            let config = config.clone();
            let ui_handle = ui.as_weak();
            move |file_path, file_format, encrypt_key| {
                let mut config = config.borrow_mut();
                let data = data.borrow();
                let ui = ui_handle.unwrap();

                config.file_format = file_format.into();
                config.encrypt_key = encrypt_key.to_string();
                config.file_path = PathBuf::from(file_path.as_str());
                config.save();

                // TODO: Noti if fail to save
                match config.file_format {
                    FileFormat::Bin => {
                        if let Err(e) = bin_file::save_to::<AppData>(&data, &config.file_path, &config.encrypt_key) {
                            error!("Failed to save: {:?}", e);
                        } else {
                            ui.set_is_saved(true);
                        }
                    }
                    FileFormat::Ron => {
                        if let Err(e) = ron_file::save_to::<AppData>(&data, &config.file_path) {
                            error!("Failed to save: {:?}", e);
                        } else {
                            ui.set_is_saved(true);
                        }
                    }
                }
            }
        });
    }

    {
        ui.on_add_class({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            let cache = cache.clone();
            move |class_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut config = config.borrow_mut();
                let mut cache = cache.borrow_mut();
                // TODO: Cache the id to avoid hashing too many times.
                let class_id = string_to_id(class_name.as_str(), &mut cache);
                // TODO: Notify if class already exist
                data.class_name_map.entry(class_id).or_insert(class_name.to_string());
                data.dialogues.entry(class_id).or_insert(BTreeMap::new());
                config.selected_class = class_id;
                config.selected_state = 0;
                reload_class(&data, &ui, "");
            }
        });

        ui.on_rename_class({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            let cache = cache.clone();
            move |old_name, new_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut config = config.borrow_mut();
                let mut cache = cache.borrow_mut();
                let old_class_id = string_to_id(old_name.as_str(), &mut cache);
                let new_class_id = string_to_id(new_name.as_str(), &mut cache);

                if !data.dialogues.contains_key(&new_class_id) {
                    if let Some(value) = data.dialogues.remove(&old_class_id) {
                        data.dialogues.insert(new_class_id, value);
                        data.class_name_map.entry(new_class_id).or_insert(new_name.to_string());
                        data.class_name_map.remove(&old_class_id);

                        config.selected_class = new_class_id;
                        reload_all(&data, &ui, &config, "", "");
                    }
                } else {
                    // TODO: Noti new name already exists
                }
            }
        });

        ui.on_remove_class({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let cache = cache.clone();
            move |class_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut cache = cache.borrow_mut();
                let class_id = string_to_id(class_name.as_str(), &mut cache);
                data.class_name_map.remove(&class_id);
                // TODO: Remove state name map if no class contain it
                data.dialogues.remove(&class_id);
                reload_class(&data, &ui, "");
            }
        });

        ui.on_select_class({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            let cache = cache.clone();
            move |class_name| {
                let ui = ui_handle.unwrap();
                let data = data.borrow();
                let mut config = config.borrow_mut();
                let mut cache = cache.borrow_mut();
                let class_id = string_to_id(class_name.as_str(), &mut cache);
                config.selected_class = class_id;
                reload_state(&data, &ui, &config, "");
            }
        });
    }

    {
        ui.on_add_state({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            let cache = cache.clone();
            move |state_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut config = config.borrow_mut();
                let mut cache = cache.borrow_mut();
                let state_id = string_to_id(state_name.as_str(), &mut cache);
                // Notify if state already exists
                config.selected_state = state_id;
                data.state_name_map.entry(state_id).or_insert(state_name.to_string());
                if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
                    class.entry(state_id).or_insert(Vec::new());
                    reload_state(&data, &ui, &config, "");
                }
            }
        });

        ui.on_select_state({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            let cache = cache.clone();
            move |state_name| {
                let ui = ui_handle.unwrap();
                let data = data.borrow();
                let mut config = config.borrow_mut();
                let mut cache = cache.borrow_mut();
                let state_id = string_to_id(state_name.as_str(), &mut cache);
                config.selected_state = state_id;
                reload_dialogue(&data, &ui, &config);
            }
        });

        ui.on_remove_state({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            let cache = cache.clone();
            move |state_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let config = config.borrow();
                let mut cache = cache.borrow_mut();
                let state_id = string_to_id(state_name.as_str(), &mut cache);
                if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
                    class.remove(&state_id);
                    // TODO: Remove state name map if no class contain it
                    reload_state(&data, &ui, &config, "");
                }
            }
        });

        ui.on_rename_state({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            let cache = cache.clone();
            move |old_name, new_name| {
                let mut cache = cache.borrow_mut();
                let old_id = string_to_id(old_name.as_str(), &mut cache);
                let new_id = string_to_id(new_name.as_str(), &mut cache);

                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut config = config.borrow_mut();

                if !data.dialogues.contains_key(&new_id)
                    && let Some(dialog) = data.dialogues.remove(&old_id)
                {
                    data.dialogues.insert(new_id, dialog);
                    data.state_name_map.entry(new_id).or_insert(new_name.to_string());
                    config.selected_state = new_id;
                    reload_all(&data, &ui, &config, "", "");
                } else {
                    // TODO: Noti new name exists
                }
            }
        });
    }

    {
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
    }

    {
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
    }

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

fn reload_all(data: &AppData, ui: &AppWindow, config: &Config, search_class: &str, search_state: &str) {
    reload_class(data, ui, search_class);
    reload_state(data, ui, config, search_state);
    reload_dialogue(data, ui, config);
    reload_dialogue_detail(data, ui, config);
}

/// Reload class section and clear state/dialogue section
fn reload_class(data: &AppData, ui: &AppWindow, search_class: &str) {
    let mut classes: Vec<SharedString> = Vec::new();
    let re = Regex::new(search_class);
    for (_, name) in data.class_name_map.iter() {
        if search_class.is_empty() {
            classes.push(name.into());
        } else if let Ok(re) = re.as_ref()
            && re.is_match(name)
        {
            classes.push(name.into());
        }
    }

    ui.set_classes(classes.as_slice().into());
    ui.set_states([].into());
    ui.set_dialogues([].into());
    ui.set_dialogue(UiDialogue::default());
}

/// Reload state section and clear dialogue section
fn reload_state(data: &AppData, ui: &AppWindow, config: &Config, search_state: &str) {
    let re = Regex::new(search_state);

    if let Some(class) = data.dialogues.get(&config.selected_class) {
        let mut states: Vec<SharedString> = Vec::new();
        for state_id in class.keys() {
            let state_name =
                if let Some(ret) = data.state_name_map.get(state_id) { ret.clone() } else { state_id.to_string() };
            if search_state.is_empty() {
                states.push(state_name.into());
            } else if let Ok(re) = re.as_ref()
                && re.is_match(state_name.as_str())
            {
                states.push(state_name.into());
            }
        }
        ui.set_states(states.as_slice().into());
    }
    ui.set_dialogues([].into());
    ui.set_dialogue(UiDialogue::default());
}

fn reload_dialogue(data: &AppData, ui: &AppWindow, config: &Config) {
    if let Some(state) = data.dialogues.get(&config.selected_class)
        && let Some(state_dialogs) = state.get(&config.selected_state)
    {
        let mut dialogues: Vec<SharedString> = Vec::new();
        for dialog in state_dialogs {
            if let Some((_, content)) = dialog.contents.first_key_value() {
                dialogues.push(content.into());
            } else {
                dialogues.push(SharedString::new());
            }
        }

        ui.set_dialogues(dialogues.as_slice().into());
        ui.set_dialogue(UiDialogue::default());
    }
}

fn reload_dialogue_detail(data: &AppData, ui: &AppWindow, config: &Config) {
    if let Some(state) = data.dialogues.get(&config.selected_class)
        && let Some(dialog_list) = state.get(&config.selected_state)
        && let Some(dialog) = dialog_list.get(config.selected_dialog)
    {
        let mut contents: Vec<ContentLang> = Vec::new();
        let mut affects: Vec<Affect> = Vec::new();
        for (lang, content) in dialog.contents.iter() {
            contents.push(ContentLang {
                language: lang.to_639_3().to_string().into(),
                content: content.into(),
            });
        }
        for (class, state) in dialog.affects.iter() {
            if let Some(class_name) = data.class_name_map.get(class)
                && let Some(state_name) = data.state_name_map.get(state)
            {
                affects.push(Affect {
                    class: class_name.into(),
                    state: state_name.into(),
                });
            }
        }

        let ui_dialogue = UiDialogue {
            contents: contents.as_slice().into(),
            affects: affects.as_slice().into(),
        };
        ui.set_dialogue(ui_dialogue);

        let mut lang_list: Vec<SharedString> = Vec::new();
        for lang in config.langs.iter() {
            lang_list.push(lang.to_639_3().to_string().into());
        }
        ui.set_lang_list(lang_list.as_slice().into());

        let mut state_list: Vec<SharedString> = Vec::new();
        for (_, state) in data.state_name_map.iter() {
            state_list.push(state.into());
        }
        ui.set_state_list(state_list.as_slice().into());
    }
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

// TODO: Allow to manually set id of string without hashing
fn string_to_id(name: &str, cache: &mut DataCache) -> u64 {
    let lower = name.to_lowercase();
    if let Some(id) = cache.name_map.get(lower.as_str()) {
        *id
    } else {
        let id = xxh3_64(lower.as_bytes());
        cache.name_map.insert(lower, id);
        id
    }
}
