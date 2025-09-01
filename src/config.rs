use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

const DEFAULT_CONFIG: &str = include_str!("../languages.toml");

#[derive(Deserialize, Debug, Clone)]
pub struct LanguageConfig {
    pub formatter: Vec<String>,
    pub comment_token: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub languages: HashMap<String, LanguageConfig>,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

impl Config {
    pub fn get_lang(&self, lang: &str) -> Option<&LanguageConfig> {
        self.languages.get(lang).or_else(|| {
            if let Some(alias) = self.aliases.get(lang) {
                self.languages.get(alias)
            } else {
                None
            }
        })
    }

    fn merge(&mut self, b: Config) {
        self.languages.extend(b.languages);
        self.aliases.extend(b.aliases);
    }
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap()
    }
}

pub fn get_config(path: Option<PathBuf>) -> Result<Config> {
    let mut config_path = default_config_path();
    if let Some(path) = path {
        config_path = Some(path);
    }

    let mut config = Config::default();

    if let Some(path) = config_path {
        let data = std::fs::read_to_string(path)?;
        let value: Config = toml::from_str(&data)?;
        config.merge(value);
    }

    Ok(config)
}

fn default_config_path() -> Option<PathBuf> {
    let path = dirs::home_dir()?.join(".config/codefmt/config.toml");
    let exists = std::fs::exists(&path).unwrap_or(false);
    match exists {
        true => Some(path),
        false => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_default_config() {
        Config::default();
    }
}
