use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use ron::{
    Value,
    ser::{self},
};
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

impl Default for Config {
    fn default() -> Self {
        let config: Self = de::from_str(DEFAULT_CONFIG)
            .map_err(|e| format!("Failed to parse default RON config: {}", e))
            .unwrap();
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
        let default_value: Value = de::from_str(DEFAULT_CONFIG)
            .map_err(|e| format!("Failed to parse default RON config: {}", e))?;

        let merged = if let Some(config_path) = Self::get_path()
            && config_path.exists()
        {
            let user_str = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;
            let user_value: Value = de::from_str(&user_str)
                .map_err(|e| format!("Failed to parse RON config: {}", e))?;
            Self::merge_ron(default_value, user_value)
        } else {
            default_value
        };
        merged
            .into_rust()
            .map_err(|e| format!("Failed to build config: {}", e))
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

    pub fn merge_ron(default: Value, user: Value) -> Value {
        match (default, user) {
            (Value::Map(mut default_map), Value::Map(user_map)) => {
                for (k, v) in user_map.iter() {
                    let merged = match default_map.get(k) {
                        Some(dv) => Self::merge_ron(dv.clone(), v.clone()),
                        None => v.clone(),
                    };
                    default_map.insert(k.clone(), merged);
                }
                Value::Map(default_map)
            }
            // scalars/seqs: user value wins outright
            (_, user_value) => user_value,
        }
    }
}
