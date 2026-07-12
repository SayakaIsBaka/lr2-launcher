use serde::{Deserialize, Serialize};
use serde_with::{serde_as, BoolFromInt};
use bitflags::bitflags;

// https://github.com/bitflags/bitflags/issues/108#issuecomment-3539935954
mod serde_bits {
    use bitflags::Flags;
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn serialize<T: Flags + Clone, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T::Bits: Serialize,
    {
        value.clone().intersection(T::all()).bits().serialize(serializer)
    }

    pub(super) fn deserialize<'de, T: Flags, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T::Bits: Deserialize<'de> + std::fmt::Debug,
    {
        let raw: T::Bits = T::Bits::deserialize(deserializer)?;
        T::from_bits(raw)
            .ok_or_else(|| serde::de::Error::custom(format!("Unexpected flags value {:?}", raw)))
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Folders: u8 {
        const Random =    0b00000001;
        const Favorite =  0b00000010;
        const Top10 =     0b00000100;
        const Level =     0b00001000;
        const Clear =     0b00010000;
        const Playrank =  0b00100000;
        const Ignore =    0b01000000;
        const InsaneBms = 0b10000000;

        const _ = !0;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub system: System,
    pub jukebox: Jukebox,
    pub play: Play,
    pub sound: Sound,
    pub player: Player,
    pub select: ConfigSelect,
    pub skin: Skin,
    pub network: Network,
    pub course: Course,
    pub tools: Tools,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct System {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub screenmode: u8, // 0: Fullscreen, 1: Windowed, 2: Borderless
    #[serde_as(as = "BoolFromInt")] pub vsync: bool,
    pub directdraw: String,
    pub maindisplay: String,
    pub highcolor: String,
    pub autoreload: u8, // 0: Disabled, 1: Auto Type 1, 2: Auto Type 2
    #[serde(with = "serde_bits")] pub customfolder: Folders,
    pub mainsleep: String,
    pub bmssleep: String,
    pub screenexrate: String,
    pub inputinterval: i32,
    pub disablesystemkey: String,
    pub outputlog: String,
    pub thread: String,
    pub eventmode: String,
    pub disableskinpreview: String,
    pub newsongfolder: String,
    pub titleflash: String,
    pub hptimer: String,
    pub disablebmsthread: String,
    pub disablefolderthread: String,
    pub language: String,
    pub windowsize_x: i32,
    pub windowsize_y: i32,
    pub softwarerendering: String,
}

#[derive(Serialize, Deserialize)]
pub struct Jukebox {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub path: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Play {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub hs: String,
    pub hstype: String,
    pub hsmax: i32,
    pub hsmin: i32,
    pub hsmargin: i32,
    pub shutter: String,
    pub shuttertype: String,
    pub shuttermargin: i32,
    pub gauge: String,
    pub random: String,
    pub effect: String,
    pub autoscratch: String,
    pub autokey: String,
    pub autojudgeadjust: String,
    pub judgetime: String,
    pub bga: String,
    pub bgasize: String,
    pub poorbga: i32,
    pub ghost: String,
    pub scoregraph: String,
    pub target: String,
    pub defaulttarget: String,
    pub replaysave: String,
    pub basespeed: i32,
    pub gomiscore: String,
    pub disableleftclickexit: String,
    pub disablecurspeedchange: String,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Sound {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub bufferlength: i32,
    pub numbuffers: String,
    pub disabledsp: String,
    pub output: i32,
    pub driver: i32,
    #[serde_as(as = "BoolFromInt")] pub disablefmod: bool,
    pub volumeflag: String,
    pub volumebgm: String,
    pub volumekey: String,
    pub volumemaster: String,
    pub eqflag: String,
    pub eqp0: String,
    pub eqp1: String,
    pub eqp2: String,
    pub eqp3: String,
    pub eqp4: String,
    pub eqp5: String,
    pub eqp6: String,
    pub pitchflag: String,
    pub pitchtype: String,
    pub pitchp: String,
    pub fxflag_0: String,
    pub fxtype_0: String,
    pub fxtarget_0: String,
    pub fxp1_0: String,
    pub fxp2_0: String,
    pub fxflag_1: String,
    pub fxtype_1: String,
    pub fxtarget_1: String,
    pub fxp1_1: String,
    pub fxp2_1: String,
    pub fxflag_2: String,
    pub fxtype_2: String,
    pub fxtarget_2: String,
    pub fxp1_2: String,
    pub fxp2_2: String,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub id: String,
    pub pass: String,
    pub name: Name,
    pub irid: Irid,
    pub irpass: Irpass,
}

#[derive(Serialize, Deserialize)]
pub struct Name {
}

#[derive(Serialize, Deserialize)]
pub struct Irid {
}

#[derive(Serialize, Deserialize)]
pub struct Irpass {
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ConfigSelect {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub sort: String,
    pub key: String,
    pub difficulty: String,
    pub searchmax: i32,
    pub randomclose: String,
    pub speedfirst: String,
    pub speednext: String,
    pub control: String,
    pub buttonselect: String,
    #[serde_as(as = "BoolFromInt")] pub folderlamp: bool,
    pub difficultychangetype: String,
    pub ignorekeyall: String,
    pub ignorekeysingle: String,
    pub ignorekeydouble: String,
    pub ignoredp: String,
    pub ignorepms: String,
    pub ignoredifficultyall: String,
    pub ignore5key: String,
    pub levelbarflash_7: String,
    pub levelbarflash_5: String,
    pub levelbarflash_9: String,
    pub disabledifficultyfilter: String,
    #[serde_as(as = "BoolFromInt")] pub preview: bool,
    pub disablesubtitle: String,
}

#[derive(Serialize, Deserialize)]
pub struct Skin {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub play_7: String,
    pub play_5: String,
    pub play_14: String,
    pub play_10: String,
    pub play_9: String,
    pub select: String,
    pub decide: String,
    pub result: String,
    pub keyconfig: String,
    pub skinselect: String,
    pub soundset: String,
    pub theme: Theme,
    pub play_7_b: String,
    pub play_5_b: String,
    pub play_9_b: Play9B,
    pub fontname: String,
    pub disableimagefont: String,
}

#[derive(Serialize, Deserialize)]
pub struct Theme {
}

#[derive(Serialize, Deserialize)]
pub struct Play9B {
}

#[derive(Serialize, Deserialize)]
pub struct Network {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub lr1ir: String,
    pub lr1id: String,
    pub lr1pass: String,
    pub lr2ir: String,
    pub mail: Mail,
    pub autoupdate: String,
    pub getrival: String,
}

#[derive(Serialize, Deserialize)]
pub struct Mail {
}

#[derive(Serialize, Deserialize)]
pub struct Course {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub optimumlevel_7: String,
    pub optimumlevel_5: String,
    pub optimumlevel_10: String,
    pub optimumlevel_14: String,
    pub optimumlevel_9: String,
    pub defaultconnection: String,
    pub defaultgauge: String,
    pub maxbpm: String,
    pub minbpm: String,
    pub bpmrange: String,
    pub maxlevel: String,
    pub minlevel: String,
    pub stage: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tools {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub bmse_body: String,
    pub bmse_option: BmseOption,
    pub movie_audio: String,
    pub movie_framerate: String,
    pub mp3_volume: String,
    pub mp3enc_body: String,
    pub mp3enc_option_movie: String,
    pub mp3enc_option_normal: String,
    pub oggdec_body: String,
    pub oggdec_option: OggdecOption,
    pub oggenc_body: String,
    pub oggenc_option: String,
    pub autowavtoogg: String,
    pub autobmptopng: String,
    pub autofumensearch: String,
}

#[derive(Serialize, Deserialize)]
pub struct BmseOption {
}

#[derive(Serialize, Deserialize)]
pub struct OggdecOption {
}
