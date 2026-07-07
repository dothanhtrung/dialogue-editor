// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod common;
mod config_tab;
mod content_tab;
mod file_handle;
mod namemap_tab;

use crate::common::*;
use crate::config_tab::*;
use crate::content_tab::class_ui::*;
use crate::content_tab::dialogue_ui::*;
use crate::content_tab::state_ui::*;
use crate::file_handle::*;
use crate::namemap_tab::*;
use isolang::Language;
use serde::{
    Deserialize,
    Serialize,
};
use slint::Model;
use std::collections::BTreeSet;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    error::Error,
    path::{
        Path,
        PathBuf,
    },
    rc::Rc,
};
use tracing_subscriber::EnvFilter;

slint::include_modules!();

#[derive(Serialize, Deserialize, Default, Clone)]
struct Dialogue {
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    contents: BTreeMap<Language, String>,
    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    affects: BTreeMap<u64, u64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    events: Vec<u64>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AppData {
    #[serde(default)]
    dialogues: BTreeMap<u64, BTreeMap<u64, Vec<Dialogue>>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    class_name_map: BTreeMap<String, u64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    state_name_map: BTreeMap<String, u64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(default)]
    event_name_map: BTreeMap<String, u64>,
}

impl AppData {
    pub fn clear_name_map(&mut self) {
        self.class_name_map.clear();
        self.state_name_map.clear();
        self.event_name_map.clear();
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    #[serde(default)]
    file_path: PathBuf,
    #[serde(default)]
    selected_class: u64, // TODO: Support history of select
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
    save_without_name: bool,
    #[serde(default)]
    langs: BTreeSet<Language>,
}

impl Config {
    fn save(&self) {
        let _ = ron_file::save_to::<Config>(self, Path::new("./dialog-editor.ron"));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_thread_ids(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let ui = AppWindow::new()?;
    let config: Config = ron_file::load_from(Path::new("./dialog-editor.ron")).unwrap_or_default();

    let data = AppData::default();

    // ======== File section ===========
    ui.global::<GFile>()
        .set_file_path(config.file_path.to_str().unwrap_or_default().into());
    ui.global::<GFile>().set_encrypt_key(config.encrypt_key.as_str().into());
    ui.global::<GFile>().set_file_format(config.file_format as i32);
    ui.global::<GFile>().set_save_without_name(config.save_without_name);

    #[cfg(feature = "crypt")]
    ui.global::<GFile>().set_enable_crypt(true);

    reload_lang_list(&config, &ui);

    let config = Rc::new(RefCell::new(config));
    let data = Rc::new(RefCell::new(data));

    ui.window().on_close_requested(on_close(ui.as_weak()));
    ui.on_file_picker(file_picker(config.clone(), ui.as_weak()));
    ui.global::<GFile>()
        .on_load(request_load(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GFile>()
        .on_save(request_save(data.clone(), config.clone(), ui.as_weak()));

    // ======== Content tab ===========
    // TODO: Move these to content_tab internally
    ui.global::<GContent>()
        .on_add_class(add_class(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_rename_class(rename_class(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_remove_class(remove_class(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_select_class(select_class(data.clone(), config.clone(), ui.as_weak()));

    ui.global::<GContent>()
        .on_add_state(add_state(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_select_state(select_state(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_remove_state(remove_state(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_rename_state(rename_state(data.clone(), config.clone(), ui.as_weak()));

    ui.global::<GContent>()
        .on_add_dialog(add_dialogue(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_select_dialog(select_dialogue(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_remove_dialog(remove_dialogue(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_add_lang_content(add_lang_content(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_add_affect(add_affect(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_update_content(update_content(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_delete_content(delete_content(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_delete_affect(delete_affect(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_add_event(add_event(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_delete_event(delete_event(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_search_class(search_class(data.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_search_state(search_state(data.clone(), config.clone(), ui.as_weak()));
    ui.global::<GContent>()
        .on_search_dialogue(search_dialogue(data.clone(), config.clone(), ui.as_weak()));

    // --------------------- Namemap Tab -------------------------

    ui.global::<GNameMap>()
        .on_delete_class(delete_class_map(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_delete_state(delete_state_map(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_delete_event(delete_event_map(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_update_class_id(update_class_id(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_update_state_id(update_state_id(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_update_event_id(update_event_id(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_new_class(add_new_class(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_new_state(add_new_state(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_new_event(add_new_event(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_search_class(search_class_map(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_search_state(search_state_map(data.clone(), ui.as_weak()));
    ui.global::<GNameMap>()
        .on_search_event(search_event_map(data.clone(), ui.as_weak()));

    // ----------------------- Config Tab ---------------------------
    ui.global::<GConfig>()
        .on_add_lang(add_lang(config.clone(), ui.as_weak()));
    ui.global::<GConfig>()
        .on_delete_lang(delete_lang(data.clone(), config.clone(), ui.as_weak()));

    ui.run()?;

    Ok(())
}

impl Dialogue {
    pub fn from(ui_dialog: UiDialogue, data: &mut AppData) -> Self {
        let mut ret = Self::default();
        for affect in ui_dialog.affects.iter() {
            let class_id = class_to_id(affect.class.as_str(), data);
            let state_id = state_to_id(affect.state.as_str(), data);
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
