pub mod bin_file;
pub mod ron_file;

use crate::{
    AppData,
    AppWindow,
    Config,
    reload_ui::reload_all,
};
use rfd::FileDialog;
use serde::{
    Deserialize,
    Serialize,
};
use slint::{
    SharedString,
    Weak,
};
use tracing::error;
use std::{
    cell::RefCell,
    path::{
        Path,
        PathBuf,
    },
    rc::Rc,
};

#[repr(i32)]
#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub enum FileFormat {
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

pub fn file_picker(config: Rc<RefCell<Config>>, ui_handle: Weak<AppWindow>) -> impl Fn() {
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
}

pub fn request_load(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, i32, SharedString) {
    // TODO: Loading icon
    // TODO: Warn if there is unsave content

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
                FileFormat::Bin => bin_file::load_from(&config.file_path, &config.encrypt_key).unwrap_or_default(),
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
}

pub fn request_save(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, i32, SharedString) {
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
}
