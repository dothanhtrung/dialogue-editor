use ron::ser::{
    PrettyConfig,
    to_string_pretty,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::fs::File;
use std::io::Write;
use std::path::Path;

// TODO: Load/save in another thread
pub fn load_from<T>(file_path: &Path) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(file_path)?;
    let data = ron::de::from_reader(file)?;
    Ok(data)
}

pub fn save_to<T>(data: &T, file_path: &Path) -> anyhow::Result<()>
where
    T: Serialize,
{
    let pretty = PrettyConfig::default();
    let ron_str = to_string_pretty(data, pretty)?;
    if let Some(parent_dir) = file_path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let mut file = File::create(file_path)?;
    file.write_all(ron_str.as_bytes())?;
    Ok(())
}
