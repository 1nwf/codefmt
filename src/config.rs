use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

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

pub fn get_config(path: Option<impl AsRef<Path>>) -> Result<Config> {
    let mut config = Config::default();

    if let Some(path) = path {
        let data = std::fs::read_to_string(path)?;
        let value: Config = toml::from_str(&data)?;
        config.merge(value);
    }

    Ok(config)
}
