use serde::{Deserialize, Serialize};
use simple_crypt::encrypt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) trait BinFile: Serialize + for<'de> Deserialize<'de> + Send + 'static {
    const ENCR_KEY: &'static str = "";

    fn load_from(&mut self, file_path: &Path) -> anyhow::Result<()> {
        let enc_saved = fs::read(file_path)?;

        let decrypted = if Self::ENCR_KEY.is_empty() {
            enc_saved
        } else {
            simple_crypt::decrypt(enc_saved.as_slice(), Self::ENCR_KEY.as_bytes())?
        };
        *self = postcard::from_bytes(decrypted.as_slice())?;
        Ok(())
    }

    fn save_to(&self, file_path: PathBuf) -> anyhow::Result<()> {
        let data = postcard::to_allocvec(self)?;

        let enc_saved = if Self::ENCR_KEY.is_empty() {
            data
        } else {
            encrypt(data.as_slice(), Self::ENCR_KEY.as_bytes())?
        };

        if let Some(parent_dir) = file_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        File::create(file_path).and_then(|mut file| file.write_all(enc_saved.as_slice()))?;
        Ok(())
    }
}
