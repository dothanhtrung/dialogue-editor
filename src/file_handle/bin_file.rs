use serde::{
    Deserialize,
    Serialize,
};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn load_from<T>(file_path: &Path, encr_key: &str) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let enc_saved = fs::read(file_path)?;

    let decrypted = if encr_key.is_empty() {
        enc_saved
    } else {
        #[cfg(feature = "crypt")]
        return simple_crypt::decrypt(enc_saved.as_slice(), encr_key.as_bytes())?;
        #[cfg(not(feature = "crypt"))]
        enc_saved
    };
    let data = postcard::from_bytes(decrypted.as_slice())?;
    Ok(data)
}

pub fn save_to<T>(data: &T, file_path: &Path, encr_key: &str) -> anyhow::Result<()>
where
    T: Serialize,
{
    let data = postcard::to_allocvec(data)?;

    let enc_saved = if encr_key.is_empty() {
        data
    } else {
        #[cfg(feature = "crypt")]
        return simple_crypt::encrypt(data.as_slice(), encr_key.as_bytes())?;
        #[cfg(not(feature = "crypt"))]
        data
    };

    if let Some(parent_dir) = file_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    File::create(file_path).and_then(|mut file| file.write_all(enc_saved.as_slice()))?;
    Ok(())
}
