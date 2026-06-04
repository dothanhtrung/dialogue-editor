// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bin_file;
mod ron_file;

use isolang::Language;
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

#[derive(Default, Serialize, Deserialize)]
enum FileFormat {
    #[default]
    Ron,
    Bin,
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

// TODO: Cache the xxh3_64
// TODO: Warning to save before exit

fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_thread_ids(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let ui = AppWindow::new()?;
    let config: Config = ron_file::load_from(Path::new("./dialog-editor.ron")).unwrap_or_default();

    let data = AppData::default();

    ui.set_file_path(config.file_path.to_str().unwrap_or_default().into());

    let config = Rc::new(RefCell::new(config));
    let data = Rc::new(RefCell::new(data));

    {
        ui.on_request_load({
            // TODO: Loading icon
            let data = data.clone();
            let config = config.clone();
            let ui_handle = ui.as_weak();
            move |file_path| {
                let ui = ui_handle.unwrap();
                let mut config = config.borrow_mut();
                let mut data = data.borrow_mut();
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

                    reload_all(&data, &ui, &config);
                }
            }
        });

        ui.on_request_save({
            // TODO: show save status
            let data = data.clone();
            let config = config.clone();
            move || {
                let config = config.borrow();
                let data = data.borrow();
                config.save();

                // TODO: Noti if fail to save
                match config.file_format {
                    FileFormat::Bin => {
                        let _ = bin_file::save_to::<AppData>(&data, &config.file_path, &config.encrypt_key);
                    }
                    FileFormat::Ron => {
                        let _ = ron_file::save_to::<AppData>(&data, &config.file_path);
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
            move |class_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut config = config.borrow_mut();
                let class_id = xxh3_64(class_name.as_bytes());
                // TODO: Notify if class already exist
                data.class_name_map.entry(class_id).or_insert(class_name.to_string());
                data.dialogues.entry(class_id).or_insert(BTreeMap::new());
                config.selected_class = class_id;
                config.selected_state = 0;
                reload_all(&data, &ui, &config);
            }
        });

        ui.on_rename_class({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            move |old_name, new_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut config = config.borrow_mut();
                let old_class_id = xxh3_64(old_name.as_bytes());
                let new_class_id = xxh3_64(new_name.as_bytes());

                if !data.dialogues.contains_key(&new_class_id) {
                    if let Some(value) = data.dialogues.remove(&old_class_id) {
                        data.dialogues.insert(new_class_id, value);
                        data.class_name_map.entry(new_class_id).or_insert(new_name.to_string());
                        data.class_name_map.remove(&old_class_id);

                        config.selected_class = new_class_id;
                        reload_all(&data, &ui, &config);
                    }
                } else {
                    // TODO: Noti new name already exists
                }
            }
        });

        ui.on_remove_class({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            move |class_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let class_id = xxh3_64(class_name.as_bytes());
                data.class_name_map.remove(&class_id);
                data.dialogues.remove(&class_id);
                reload_class(&data, &ui);
            }
        });

        ui.on_select_class({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            move |class_name| {
                let ui = ui_handle.unwrap();
                let data = data.borrow();
                let mut config = config.borrow_mut();
                let class_id = xxh3_64(class_name.as_bytes());
                config.selected_class = class_id;
                reload_state(&data, &ui, &config);
            }
        });
    }

    {
        ui.on_add_state({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            move |state_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut config = config.borrow_mut();
                let state_id = xxh3_64(state_name.as_bytes());
                // Notify if state already exists
                config.selected_state = state_id;
                data.state_name_map.entry(state_id).or_insert(state_name.to_string());
                if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
                    class.entry(state_id).or_insert(Vec::new());
                    reload_state(&data, &ui, &config);
                }
            }
        });

        ui.on_select_state({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            move |state_name| {
                let ui = ui_handle.unwrap();
                let data = data.borrow();
                let mut config = config.borrow_mut();
                let state_id = xxh3_64(state_name.as_bytes());
                config.selected_state = state_id;
                reload_dialogue(&data, &ui, &config);
            }
        });

        ui.on_remove_state({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            move |state_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let config = config.borrow();
                let state_id = xxh3_64(state_name.as_bytes());
                if let Some(class) = data.dialogues.get_mut(&config.selected_class) {
                    class.remove(&state_id);
                    reload_state(&data, &ui, &config);
                }
            }
        });

        ui.on_rename_state({
            let ui_handle = ui.as_weak();
            let data = data.clone();
            let config = config.clone();
            move |old_name, new_name| {
                let old_id = xxh3_64(old_name.as_bytes());
                let new_id = xxh3_64(new_name.as_bytes());

                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let mut config = config.borrow_mut();

                if !data.dialogues.contains_key(&new_id)
                    && let Some(dialog) = data.dialogues.remove(&old_id)
                {
                    data.dialogues.insert(new_id, dialog);
                    data.state_name_map.entry(new_id).or_insert(new_name.to_string());
                    config.selected_state = new_id;
                    reload_all(&data, &ui, &config);
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
            move |class_name, state_name| {
                let ui = ui_handle.unwrap();
                let mut data = data.borrow_mut();
                let config = config.borrow();
                if let Some(class) = data.dialogues.get_mut(&config.selected_class)
                    && let Some(state) = class.get_mut(&config.selected_state)
                    && let Some(dialogue) = state.get_mut(config.selected_dialog)
                {
                    let class_id = xxh3_64(class_name.as_bytes());
                    let state_id = xxh3_64(state_name.as_bytes());

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
    }

    ui.run()?;

    Ok(())
}

fn reload_all(data: &AppData, ui: &AppWindow, config: &Config) {
    reload_class(data, ui);
    reload_state(data, ui, config);
    reload_dialogue(data, ui, config);
    reload_dialogue_detail(data, ui, config);
}

/// Reload class section and clear state/dialogue section
fn reload_class(data: &AppData, ui: &AppWindow) {
    let mut classes: Vec<SharedString> = Vec::new();
    for (_, name) in data.class_name_map.iter() {
        classes.push(name.into());
    }

    ui.set_classes(classes.as_slice().into());
    ui.set_states([].into());
    ui.set_dialogues([].into());
}

/// Reload state section and clear dialogue section
fn reload_state(data: &AppData, ui: &AppWindow, config: &Config) {
    if let Some(class) = data.dialogues.get(&config.selected_class) {
        let mut states: Vec<SharedString> = Vec::new();
        for state_id in class.keys() {
            let state_name =
                if let Some(ret) = data.state_name_map.get(state_id) { ret.clone() } else { state_id.to_string() };
            states.push(state_name.into());
        }
        ui.set_states(states.as_slice().into());
    }
    ui.set_dialogues([].into());
}

fn reload_dialogue(data: &AppData, ui: &AppWindow, config: &Config) {
    if let Some(state) = data.dialogues.get(&config.selected_class)
        && let Some(state_dialogs) = state.get(&config.selected_state)
    {
        let mut dialogues: Vec<SharedString> = Vec::new();
        for dialog in state_dialogs {
            if let Some((_, content)) = dialog.contents.first_key_value() {
                dialogues.push(content.into());
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
        let mut contents: Vec<(SharedString, SharedString)> = Vec::new();
        let mut affects: Vec<(SharedString, SharedString)> = Vec::new();
        for (lang, content) in dialog.contents.iter() {
            contents.push((lang.to_639_3().to_string().into(), content.into()));
        }
        for (class, state) in dialog.affects.iter() {
            if let Some(class_name) = data.class_name_map.get(class)
                && let Some(state_name) = data.state_name_map.get(state)
            {
                affects.push((class_name.into(), state_name.into()));
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
    }
}

impl From<UiDialogue> for Dialogue {
    fn from(ui_dialog: UiDialogue) -> Self {
        let mut ret = Self::default();
        for (class, state) in ui_dialog.affects.iter() {
            let class_id = xxh3_64(class.as_bytes());
            let state_id = xxh3_64(state.as_bytes());
            ret.affects.insert(class_id, state_id);
        }
        for (lang, content) in ui_dialog.contents.iter() {
            ret.contents.insert(
                Language::from_639_3(lang.as_str()).unwrap_or_default(),
                content.to_string(),
            );
        }
        ret
    }
}
