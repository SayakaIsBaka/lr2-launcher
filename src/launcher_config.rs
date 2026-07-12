use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Clone)]
pub enum Language {
    English = 0,
    Japanese,
    Korean
}

impl TryFrom<i32> for Language {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == Language::English as i32 => Ok(Language::English),
            x if x == Language::Japanese as i32 => Ok(Language::Japanese),
            x if x == Language::Korean as i32 => Ok(Language::Korean),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub lr2_path: PathBuf,
    pub dark_mode: bool,
    pub language: Language,
    pub disable_score: bool,
}