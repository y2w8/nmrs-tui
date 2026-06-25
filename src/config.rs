use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use ron::ser::{self};
use ron::{
    de::{self},
    ser::PrettyConfig,
};
use serde::{Deserialize, Serialize};

use crate::ui::Ui;

const DEFAULT_CONFIG: &str = include_str!("../config.ron");

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub ui: Ui,
}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        let config: Self = de::from_str(DEFAULT_CONFIG)
            .map_err(|e| format!("Failed to parse default RON config: {}", e)).unwrap();
        config
    }
}

impl Config {
    fn get_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("nmrs-tui/config.ron");
            p
        })
    }

    pub fn load() -> Result<Self, String> {
        if let Some(config_path) = Self::get_path()
            && config_path.exists()
        {
            let config_file = File::open(config_path)
                .map_err(|e| format!("Failed to open config file: {}", e))?;

            let config: Self = de::from_reader(config_file)
                .map_err(|e| format!("Failed to parse RON config: {}", e))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn create(&self) -> Result<(), String> {
        let config_path = Self::get_path().ok_or("Cannot find config directory!")?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let mut config_file =
            File::create(config_path).map_err(|e| format!("Failed to create/open file: {}", e))?;

        let pretty = PrettyConfig::new();
        let ron_string = ser::to_string_pretty(self, pretty)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        config_file
            .write_all(ron_string.as_bytes())
            .map_err(|e| format!("Failed to write default config: {}", e))?;
        Ok(())
    }
}
