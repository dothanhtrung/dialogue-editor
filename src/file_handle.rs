pub mod bin_file;
pub mod ron_file;

use crate::{
    AppData,
    AppWindow,
    Config,
    GFile,
    UnsaveLoadDialog,
    common::{
        NotiLevel,
        id_to_class,
        id_to_state,
        show_noti,
    },
    content_tab::reload_content,
    namemap_tab::reload_all_map,
};
use rfd::FileDialog;
use serde::{
    Deserialize,
    Serialize,
};
use slint::{
    CloseRequestResponse, ComponentHandle, SharedString, Weak
};
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
        ui.global::<GFile>()
            .set_file_path(config.file_path.to_str().unwrap_or_default().into());

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
        ui.global::<GFile>().set_file_format(config.file_format as i32);
    }
}

pub fn request_load(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, i32, SharedString, bool) {
    // TODO: Loading icon
    // TODO: Warn if there is unsave content

    move |file_path, file_format, encrypt_key, force| {
        let ui = ui_handle.unwrap();
        let mut config = config.borrow_mut();
        let mut data = data.borrow_mut();

        config.file_format = file_format.into();
        config.encrypt_key = encrypt_key.to_string();
        config.file_path = PathBuf::from(file_path.as_str());
        config.save();

        if !config.file_path.is_file() {
            show_noti(
                &ui,
                NotiLevel::Error,
                format!("Not a file: {}", config.file_path.display()).as_str(),
            );
            return;
        }

        let is_saved = ui.get_is_saved();
        if !force && !is_saved {
            let Ok(dialog) = UnsaveLoadDialog::new() else {
                return;
            };
            let _ = dialog.run();
            return;
        }

        *data = match config.file_format {
            FileFormat::Bin => match bin_file::load_from(&config.file_path, &config.encrypt_key) {
                Ok(ret) => ret,
                Err(e) => {
                    show_noti(&ui, NotiLevel::Error, format!("Failed to load: {}", e).as_str());
                    AppData::default()
                }
            },
            FileFormat::Ron => match ron_file::load_from(&config.file_path) {
                Ok(ret) => ret,
                Err(e) => {
                    show_noti(&ui, NotiLevel::Error, format!("Failed to load: {}", e).as_str());
                    AppData::default()
                }
            },
        };
        if (config.selected_class == 0 || id_to_class(config.selected_class, &data).is_none())
            && let Some(first_class) = data.dialogues.keys().next()
        {
            config.selected_class = *first_class;
        }

        if (config.selected_state == 0 || id_to_state(config.selected_state, &data).is_none())
            && let Some(selected_class) = data.dialogues.get(&config.selected_class)
            && let Some((first_state, _)) = selected_class.first_key_value()
        {
            config.selected_class = *first_state;
        }

        reload_content(&mut data, &ui, &config);
        reload_all_map(&data, &ui);
        ui.set_is_saved(true);
    }
}

// TODO: Shortcut Ctrl-S
pub fn request_save(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, i32, SharedString, bool, bool) {
    move |file_path, file_format, encrypt_key, save_without_name, _force| {
        let mut config = config.borrow_mut();
        let data = data.borrow();
        let ui = ui_handle.unwrap();

        config.file_format = file_format.into();
        config.encrypt_key = encrypt_key.to_string();
        config.file_path = PathBuf::from(file_path.as_str());
        config.save_without_name = save_without_name;
        config.save();

        match config.file_format {
            FileFormat::Bin => {
                if let Err(e) = bin_file::save_to::<AppData>(&data, &config.file_path, &config.encrypt_key) {
                    show_noti(&ui, NotiLevel::Error, format!("Failed to save: {:?}", e).as_str());
                } else {
                    if save_without_name {
                        let mut data_without_name = data.clone();
                        data_without_name.clear_name_map();
                        let file_path = config.file_path.with_extension("no_name.bin");
                        if let Err(e) = bin_file::save_to(&data_without_name, &file_path, &config.encrypt_key) {
                            show_noti(&ui, NotiLevel::Error, format!("Failed to save: {:?}", e).as_str());
                        }
                    }

                    ui.set_is_saved(true);
                    show_noti(
                        &ui,
                        NotiLevel::Info,
                        format!("Success save {}", &config.file_path.display()).as_str(),
                    );
                }
            }
            FileFormat::Ron => {
                if let Err(e) = ron_file::save_to::<AppData>(&data, &config.file_path) {
                    show_noti(&ui, NotiLevel::Error, format!("Failed to save: {:?}", e).as_str());
                } else {
                    let mut data_without_name = data.clone();
                    data_without_name.clear_name_map();
                    let file_path = config.file_path.with_extension("no_name.ron");
                    if let Err(e) = ron_file::save_to(&data_without_name, &file_path) {
                        show_noti(&ui, NotiLevel::Error, format!("Failed to save: {:?}", e).as_str());
                    }

                    ui.set_is_saved(true);
                    show_noti(
                        &ui,
                        NotiLevel::Info,
                        format!("Success save {}", &config.file_path.display()).as_str(),
                    );
                }
            }
        }
    }
}

// pub fn on_close() -> impl FnMut() -> CloseRequestResponse + 'static {
//     move || {

//     }
// }