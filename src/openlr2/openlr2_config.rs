use std::{fs::File, io::{BufReader, Write}, path::{Path, PathBuf}};
use anyhow::Result;
use encoding_rs::SHIFT_JIS;
use quick_xml::se::EmptyElementHandling;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, BoolFromInt};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub system: System,
    pub play: Play,
    pub skin: Skin,
    pub network: Network,
}

#[derive(Serialize, Deserialize, Default)]
pub struct System {
    pub resolution: u8,
}

#[serde_as]
#[derive(Serialize, Deserialize, Default)]
pub struct Play {
    #[serde_as(as = "BoolFromInt")] pub gaugeautoshift: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Skin {
    pub courseresult: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Network {
    pub display_ir: String,
}

impl Config {
    pub fn load(lr2_folder_path: &PathBuf) -> Result<Config> {
        let config_file_path = lr2_folder_path.join("LR2files\\Config\\openlr2-config.xml");
        let config_file = File::open(config_file_path)?;
        let config: Config = quick_xml::de::from_reader(BufReader::new(config_file))?;
        Ok(config)
    }

    pub fn write(&self, lr2_folder_path: &Path) -> Result<()> {
        let config_path = lr2_folder_path.join("LR2files\\Config\\openlr2-config.xml");
        let mut config_file = File::create(config_path)?;

        let mut buffer = String::from("<?xml version=\"1.0\" encoding=\"shift_jis\"?>\n");
        let mut ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("config"))?;
        ser.empty_element_handling(EmptyElementHandling::Expanded);
        ser.indent('\t', 1);
        self.serialize(ser).unwrap();
        buffer = buffer.replace("\n", "\r\n");

        let config_encoded = SHIFT_JIS.encode(&buffer);
        config_file.write_all(&config_encoded.0)?;

        Ok(())
    }
}