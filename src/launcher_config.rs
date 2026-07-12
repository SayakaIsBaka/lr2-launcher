use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Language {
    English,
    Japanese,
    Korean
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub lr2_path: PathBuf,
    pub dark_mode: bool,
    pub language: Language,
    pub disable_score: bool,
}