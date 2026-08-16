// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod common;
mod config_tab;
mod content_tab;
mod file_handle;
mod history;
mod namemap_tab;
mod sequence_tab;

use crate::file_handle::{
    FileFormat,
    ron_file,
};
use crate::history::{
    History,
    redo,
    undo,
};
use clap::Parser;
use indexmap::IndexMap;
use isolang::Language;
use serde::{
    Deserialize,
    Serialize,
};
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
use tracing::warn;
use tracing_subscriber::EnvFilter;

slint::include_modules!();

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[clap(short, long, default_value = "./dialogue-editor.ron")]
    config: PathBuf,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct Dialogue {
    #[serde(default)]
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    contents: IndexMap<Language, String>,
    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    affects: BTreeMap<u64, u64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    events: BTreeSet<u64>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SequenceItem {
    pub class: u64,
    pub state: u64,
    pub dialogue: Option<usize>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AppData {
    #[serde(default)]
    dialogues: IndexMap<u64, IndexMap<u64, Vec<Dialogue>>>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    sequences: IndexMap<u64, Vec<SequenceItem>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    class_name_map: BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    state_name_map: BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    event_name_map: BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    sequence_name_map: BTreeMap<String, u64>,
}

impl AppData {
    pub fn clear_name_map(&mut self) {
        self.class_name_map.clear();
        self.state_name_map.clear();
        self.event_name_map.clear();
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(skip)]
    config_path: PathBuf,
    #[serde(default)]
    file_path: PathBuf,
    #[serde(default, skip)]
    selected_class: u64, // TODO: Support history of select
    #[serde(default, skip)]
    selected_state: u64,
    #[serde(default, skip)]
    selected_dialogue: usize,
    #[serde(default, skip)]
    selected_sequence: u64,
    #[serde(default)]
    /// Used when file_format is Bin
    encrypt_key: String,
    #[serde(default)]
    file_format: FileFormat,
    #[serde(default)]
    save_without_name: bool,
    #[serde(default = "default_lang")]
    langs: BTreeSet<Language>,
    #[serde(default)]
    history: History,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("dialogue_editor.ron"),
            file_path: PathBuf::new(),
            selected_class: 0,
            selected_state: 0,
            selected_dialogue: 0,
            selected_sequence: 0,
            encrypt_key: String::new(),
            file_format: FileFormat::default(),
            save_without_name: false,
            langs: BTreeSet::from([Language::Eng]),
            history: History::default(),
        }
    }
}

impl Config {
    fn load(path: &Path) -> Self {
        if !path.is_file() {
            warn!("Config file {:?} does not exist", path);
        }
        let mut ret = match ron_file::load_from(path) {
            Ok(ret) => ret,
            Err(e) => {
                warn!("Failed to load config file {:?}: {}", path, e);
                Config::default()
            }
        };
        ret.config_path = PathBuf::from(path);
        ret
    }

    fn save(&self) {
        let _ = ron_file::save_to::<Config>(self, &self.config_path);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_thread_ids(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Cli::parse();

    let ui = AppWindow::new()?;
    let config = Config::load(&args.config);

    let data = AppData::default();

    ui.global::<GConfig>().set_max_undo(config.history.limit as i32);

    // ======== File section ===========
    ui.global::<GFile>().set_encrypt_key(config.encrypt_key.as_str().into());
    ui.global::<GFile>().set_file_format(config.file_format as i32);
    ui.global::<GFile>().set_save_without_name(config.save_without_name);

    #[cfg(feature = "crypt")]
    ui.global::<GFile>().set_enable_crypt(true);

    config_tab::reload_lang_list(&config, &ui);

    let config = Rc::new(RefCell::new(config));
    let data = Rc::new(RefCell::new(data));

    ui.window().on_close_requested(file_handle::on_close(ui.as_weak()));

    ui.on_undo(undo(data.clone(), config.clone(), ui.as_weak()));
    ui.on_redo(redo(data.clone(), config.clone(), ui.as_weak()));

    // ======== File ============
    let mut ui_file = ui.global::<GFile>();
    file_handle::setup(&mut ui_file, data.clone(), config.clone(), ui.as_weak());

    // ======== Content tab ===========
    let mut ui_content = ui.global::<GContent>();
    content_tab::setup(&mut ui_content, data.clone(), config.clone(), ui.as_weak());

    // ========= Sequence ===========
    let mut ui_sequence = ui.global::<GSequence>();
    sequence_tab::setup(&mut ui_sequence, data.clone(), config.clone(), ui.as_weak());

    // --------------------- Namemap Tab -------------------------
    let mut ui_namemap = ui.global::<GNameMap>();
    namemap_tab::setup(&mut ui_namemap, data.clone(), config.clone(), ui.as_weak());

    // ----------------------- Config Tab ---------------------------
    let mut ui_config = ui.global::<GConfig>();
    config_tab::setup(&mut ui_config, data.clone(), config.clone(), ui.as_weak());

    ui.run()?;

    Ok(())
}

fn default_lang() -> BTreeSet<Language> {
    BTreeSet::from([Language::Eng])
}
