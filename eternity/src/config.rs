use std::fs;
use std::path::Path;

use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Matrix {
    pub homeserver_url: String,
    pub username: String,
    pub access_token: String,
    pub store_path: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Gitlab {
    pub gitlab_url: String,
    pub access_token: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    pub matrix: Matrix,
    pub gitlab: Option<Gitlab>,
    pub plugins_path: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
        let config: Self = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
