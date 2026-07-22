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
    CloseRequestResponse,
    ComponentHandle,
    SharedString,
    Weak,
};
use std::{
    cell::RefCell,
    path::{
        Path,
        PathBuf,
    },
    rc::Rc,
};

// TODO: Autosave

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

pub fn file_picker(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(bool, bool) {
    // TODO: Loading icon
    move |load_true_save_false, force| {
        let mut data = data.borrow_mut();
        let ui = ui_handle.unwrap();
        let mut config = config.borrow_mut();

        let is_saved = ui.get_is_saved();
        if load_true_save_false && !is_saved && !force {
            let Ok(dialog) = UnsaveLoadDialog::new() else {
                ui.global::<GFile>().set_is_loading(false);
                return;
            };
            dialog.on_cancel_clicked({
                let dialog_handle = dialog.as_weak();
                move || {
                    let dialog = dialog_handle.unwrap();
                    let _ = dialog.hide();
                }
            });
            dialog.on_yes_clicked({
                let dialog_handle = dialog.as_weak();
                let ui_handle = ui.as_weak();
                move || {
                    let ui = ui_handle.unwrap();
                    let ui = ui.global::<GFile>();
                    let dialog = dialog_handle.unwrap();
                    let _ = dialog.hide();
                    ui.invoke_file_picker(true, true);
                }
            });
            let _ = dialog.run();
            ui.global::<GFile>().set_is_loading(false);
            return;
        }

        let file_dialog = FileDialog::new()
            .add_filter("Ron", &["ron"])
            .add_filter("Bin", &["bin"])
            .set_directory(
                config
                    .file_path
                    .parent()
                    .unwrap_or(Path::new("/"))
                    .to_str()
                    .unwrap_or("/"),
            );
        config.file_path = if load_true_save_false {
            let Some(file_path) = file_dialog.pick_file() else {
                return;
            };
            file_path
        } else {
            let Some(file_path) = file_dialog.save_file() else {
                return;
            };
            file_path
        };

        ui.global::<GFile>()
            .set_file_path(config.file_path.to_str().unwrap_or_default().into());

        config.file_format = get_file_format(&config.file_path);
        ui.global::<GFile>().set_file_format(config.file_format as i32);

        if load_true_save_false {
            load(&mut data, &mut config, &ui);
        } else {
            // Show warning when data is empty and the save file is not empty
            save(&data, &config, &ui);
        }

        ui.global::<GFile>().set_is_loading(false);
    }
}

fn load(data: &mut AppData, config: &mut Config, ui: &AppWindow) {
    *data = match config.file_format {
        FileFormat::Bin => match bin_file::load_from(&config.file_path, &config.encrypt_key) {
            Ok(ret) => ret,
            Err(e) => {
                show_noti(ui, NotiLevel::Error, format!("Failed to load: {}", e).as_str());
                AppData::default()
            }
        },
        FileFormat::Ron => match ron_file::load_from(&config.file_path) {
            Ok(ret) => ret,
            Err(e) => {
                show_noti(ui, NotiLevel::Error, format!("Failed to load: {}", e).as_str());
                AppData::default()
            }
        },
    };
    if (config.selected_class == 0 || id_to_class(config.selected_class, data).is_none())
        && let Some(first_class) = data.dialogues.keys().next()
    {
        config.selected_class = *first_class;
    }

    if (config.selected_state == 0 || id_to_state(config.selected_state, data).is_none())
        && let Some(selected_class) = data.dialogues.get(&config.selected_class)
        && let Some((first_state, _)) = selected_class.first_key_value()
    {
        config.selected_state = *first_state;
    }

    reload_content(data, ui, config);
    reload_all_map(data, ui);
    ui.set_is_saved(true);
    ui.global::<GFile>().set_is_loading(false);
}

// TODO: Shortcut Ctrl-S
pub fn request_save(
    data: Rc<RefCell<AppData>>,
    config: Rc<RefCell<Config>>,
    ui_handle: Weak<AppWindow>,
) -> impl Fn(SharedString, SharedString, bool) {
    move |file_path, encrypt_key, save_without_name| {
        let mut config = config.borrow_mut();
        let data = data.borrow();
        let ui = ui_handle.unwrap();

        config.encrypt_key = encrypt_key.to_string();
        config.file_path = PathBuf::from(file_path.as_str());
        config.save_without_name = save_without_name;
        config.file_format = get_file_format(&config.file_path);

        save(&data, &config, &ui);
    }
}

fn save(data: &AppData, config: &Config, ui: &AppWindow) {
    config.save();
    match config.file_format {
        FileFormat::Bin => {
            if let Err(e) = bin_file::save_to::<AppData>(data, &config.file_path, &config.encrypt_key) {
                show_noti(ui, NotiLevel::Error, format!("Failed to save: {:?}", e).as_str());
            } else {
                if config.save_without_name {
                    let mut data_without_name = data.clone();
                    data_without_name.clear_name_map();
                    let file_path = config.file_path.with_extension("no_name.bin");
                    if let Err(e) = bin_file::save_to(&data_without_name, &file_path, &config.encrypt_key) {
                        show_noti(ui, NotiLevel::Error, format!("Failed to save: {:?}", e).as_str());
                    }
                }

                ui.set_is_saved(true);
                show_noti(
                    ui,
                    NotiLevel::Info,
                    format!("Success save {}", &config.file_path.display()).as_str(),
                );
            }
        }
        FileFormat::Ron => {
            if let Err(e) = ron_file::save_to::<AppData>(data, &config.file_path) {
                show_noti(ui, NotiLevel::Error, format!("Failed to save: {:?}", e).as_str());
            } else {
                if config.save_without_name {
                    let mut data_without_name = data.clone();
                    data_without_name.clear_name_map();
                    let file_path = config.file_path.with_extension("no_name.ron");
                    if let Err(e) = ron_file::save_to(&data_without_name, &file_path) {
                        show_noti(ui, NotiLevel::Error, format!("Failed to save: {:?}", e).as_str());
                    }
                }

                ui.set_is_saved(true);
                show_noti(
                    ui,
                    NotiLevel::Info,
                    format!("Success save {}", &config.file_path.display()).as_str(),
                );
            }
        }
    }
    ui.global::<GFile>().set_is_loading(false);
}

pub fn on_close(ui_handle: Weak<AppWindow>) -> impl FnMut() -> CloseRequestResponse + 'static {
    move || {
        let ui = ui_handle.unwrap();
        let is_saved = ui.get_is_saved();

        if is_saved {
            return CloseRequestResponse::HideWindow;
        }

        // Show warning dialog if there is unsaved work
        let Ok(dialog) = UnsaveLoadDialog::new() else {
            return CloseRequestResponse::HideWindow;
        };
        dialog.on_cancel_clicked({
            let dialog_handle = dialog.as_weak();
            move || {
                let dialog = dialog_handle.unwrap();
                let _ = dialog.hide();
            }
        });
        dialog.on_yes_clicked({
            let dialog_handle = dialog.as_weak();
            let ui_handle = ui.as_weak();
            move || {
                let ui = ui_handle.unwrap();
                let dialog = dialog_handle.unwrap();

                ui.set_is_saved(true);
                let _ = ui.hide();
                let _ = dialog.hide();
            }
        });
        let _ = dialog.run();

        CloseRequestResponse::KeepWindowShown
    }
}

// TODO: Check mime type instead of file extension
fn get_file_format(path: &Path) -> FileFormat {
    let Some(ext) = path.extension() else {
        return FileFormat::Ron;
    };
    let Some(ext) = ext.to_str() else {
        return FileFormat::Ron;
    };
    let ext = ext.to_string();
    if ext.eq_ignore_ascii_case("ron") { FileFormat::Ron } else { FileFormat::Bin }
}
