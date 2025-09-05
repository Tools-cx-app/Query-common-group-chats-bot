use std::{collections::HashSet, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::defs::CONFIG_FILE;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainConfig {
    pub groups: HashSet<i64>,
    pub admins: HashSet<i64>,
}

impl MainConfig {
    pub fn init() {
        let config = Path::new(CONFIG_FILE);

        if !config.exists() {
            Self::rewrite_config(None);
        }
    }

    pub fn rewrite_config(c: Option<Self>) {
        let default = match c {
            Some(c) => c,
            None => Self {
                groups: HashSet::new(),
                admins: HashSet::new(),
            },
        };
        let config = Path::new(CONFIG_FILE);
        let s = toml::to_string(&default).unwrap();

        fs::write(config, s).unwrap();
    }

    pub fn read_config() -> Self {
        let config = Path::new(CONFIG_FILE);
        let file = fs::read_to_string(config).unwrap();

        let toml: Self = match toml::from_str(file.as_str()) {
            Ok(s) => s,
            Err(_) => {
                Self::rewrite_config(None);
                Self::read_config();
                let file = fs::read_to_string(config).unwrap();
                let toml: Self = toml::from_str(file.as_str()).unwrap();
                toml
            }
        };

        toml
    }
}
