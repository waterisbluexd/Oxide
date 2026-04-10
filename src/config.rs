use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct OxideConfig {
    pub palette_size: usize,
}

impl Default for OxideConfig {
    fn default() -> Self {
        Self { palette_size: 16 }
    }
}

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("oxide").join("config.toml")
}

pub fn load() -> OxideConfig {
    let path = config_path();
    if !path.exists() {
        return OxideConfig::default();
    }
    let contents = fs::read_to_string(&path).unwrap_or_default();
    toml::from_str(&contents).unwrap_or_default()
}

pub fn save(config: &OxideConfig) {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let contents = toml::to_string(config).unwrap();
    fs::write(&path, contents).unwrap();
    println!(
        "[i] config: Saved count = {} to {:?}",
        config.palette_size, path
    );
}
