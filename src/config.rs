use crate::checks::DifficultyTier;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default difficulty tier for new characters
    #[serde(default)]
    pub default_difficulty: DifficultyTier,
    /// Show interjections in hook-check output (stderr)
    #[serde(default = "default_true")]
    pub show_interjections: bool,
    /// Show inner voice commentary after checks
    #[serde(default = "default_true")]
    pub show_inner_voice: bool,
    /// Atrophy threshold in days (0 = disabled)
    #[serde(default = "default_atrophy_days")]
    pub atrophy_days: u32,
}

fn default_true() -> bool {
    true
}
fn default_atrophy_days() -> u32 {
    7
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_difficulty: DifficultyTier::Normal,
            show_interjections: true,
            show_inner_voice: true,
            atrophy_days: 7,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("inland-empire")
        .join("config.toml")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            return Config::default();
        }
        let s = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&s).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let s = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, s).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn path() -> PathBuf {
        config_path()
    }
}
