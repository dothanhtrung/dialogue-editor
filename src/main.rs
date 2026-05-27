// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bin_file;
mod ron_file;

use crate::ron_file::RonFile;
use serde::{Deserialize, Serialize};
use std::error::Error;

slint::include_modules!();

#[derive( Serialize, Deserialize, Default, Clone)]
pub struct Dialogue {
    pub content: String,
    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    pub affect: Option<(u32, u32)>,
}

impl RonFile for Dialogue {}

// impl BinFile for Dialogue{}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.on_request_load({
        let ui_handle = ui.as_weak();
        move |file_path| {

        }
    });

    ui.on_request_save({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
        //     ui.set_counter(ui.get_counter() + 1);
        }
    });

    ui.run()?;

    Ok(())
}
