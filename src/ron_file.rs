use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) trait RonFile: Serialize + for<'de> Deserialize<'de> + Send + 'static {
    fn load_from(&mut self, file_path: &Path) -> anyhow::Result<()> {
        let file = File::open(file_path)?;
        *self = ron::de::from_reader(file)?;
        Ok(())
    }

    fn save_to(&self, file_path: PathBuf) -> anyhow::Result<()> {
        let pretty = PrettyConfig::default();
        let ron_str = to_string_pretty(self, pretty)?;
        if let Some(parent_dir) = file_path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        let mut file = File::create(file_path)?;
        file.write_all(ron_str.as_bytes())?;
        Ok(())
    }
}
