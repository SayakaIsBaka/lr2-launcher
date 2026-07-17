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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
#[derive(Serialize, Deserialize, Default)]
pub struct System {
    pub screenmode: u8, // 0: Fullscreen, 1: Windowed, 2: Borderless
    #[serde_as(as = "BoolFromInt")] pub vsync: bool,
    pub directdraw: i32,
    pub maindisplay: i32,
    pub highcolor: i32,
    pub autoreload: u8, // 0: Disabled, 1: Auto Type 1, 2: Auto Type 2
    #[serde(with = "serde_bits")] pub customfolder: Folders,
    pub mainsleep: i32,
    pub bmssleep: i32,
    pub screenexrate: i32,
    pub inputinterval: i32,
    pub disablesystemkey: i32,
    pub outputlog: i32,
    pub thread: i32,
    pub eventmode: i32,
    pub disableskinpreview: i32,
    pub newsongfolder: String,
    pub titleflash: i32,
    pub hptimer: i32,
    pub disablebmsthread: i32,
    pub disablefolderthread: i32,
    pub language: i32,
    pub windowsize_x: i32,
    pub windowsize_y: i32,
    pub softwarerendering: i32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Jukebox {
    pub path: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Play {
    pub hs: i32,
    pub hstype: i32,
    pub hsmax: i32,
    pub hsmin: i32,
    pub hsmargin: i32,
    pub shutter: i32,
    pub shuttertype: i32,
    pub shuttermargin: i32,
    pub gauge: i32,
    pub random: i32,
    pub effect: i32,
    pub autoscratch: i32,
    pub autokey: i32,
    pub autojudgeadjust: i32,
    pub judgetime: i32,
    pub bga: i32,
    pub bgasize: i32,
    pub poorbga: i32,
    pub ghost: i32,
    pub scoregraph: i32,
    pub target: i32,
    pub defaulttarget: i32,
    pub replaysave: i32,
    pub basespeed: i32,
    pub gomiscore: i32,
    pub disableleftclickexit: i32,
    pub disablecurspeedchange: i32,
}

#[serde_as]
#[derive(Serialize, Deserialize, Default)]
pub struct Sound {
    pub bufferlength: i32,
    pub numbuffers: i32,
    pub disabledsp: i32,
    pub output: i32,
    pub driver: i32,
    #[serde_as(as = "BoolFromInt")] pub disablefmod: bool,
    pub volumeflag: i32,
    pub volumebgm: i32,
    pub volumekey: i32,
    pub volumemaster: i32,
    pub eqflag: i32,
    pub eqp0: i32,
    pub eqp1: i32,
    pub eqp2: i32,
    pub eqp3: i32,
    pub eqp4: i32,
    pub eqp5: i32,
    pub eqp6: i32,
    pub pitchflag: i32,
    pub pitchtype: i32,
    pub pitchp: i32,
    pub fxflag_0: i32,
    pub fxtype_0: i32,
    pub fxtarget_0: i32,
    pub fxp1_0: i32,
    pub fxp2_0: i32,
    pub fxflag_1: i32,
    pub fxtype_1: i32,
    pub fxtarget_1: i32,
    pub fxp1_1: i32,
    pub fxp2_1: i32,
    pub fxflag_2: i32,
    pub fxtype_2: i32,
    pub fxtarget_2: i32,
    pub fxp1_2: i32,
    pub fxp2_2: i32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Player {
    pub id: String,
    pub pass: String,
    pub name: Name,
    pub irid: Irid,
    pub irpass: Irpass,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Name {
}

#[derive(Serialize, Deserialize, Default)]
pub struct Irid {
}

#[derive(Serialize, Deserialize, Default)]
pub struct Irpass {
}

#[serde_as]
#[derive(Serialize, Deserialize, Default)]
pub struct ConfigSelect {
    pub sort: i32,
    pub key: i32,
    pub difficulty: i32,
    pub searchmax: i32,
    pub randomclose: i32,
    pub speedfirst: i32,
    pub speednext: i32,
    pub control: i32,
    pub buttonselect: i32,
    #[serde_as(as = "BoolFromInt")] pub folderlamp: bool,
    pub difficultychangetype: i32,
    pub ignorekeyall: i32,
    pub ignorekeysingle: i32,
    pub ignorekeydouble: i32,
    pub ignoredp: i32,
    pub ignorepms: i32,
    pub ignoredifficultyall: i32,
    pub ignore5key: i32,
    pub levelbarflash_7: i32,
    pub levelbarflash_5: i32,
    pub levelbarflash_9: i32,
    pub disabledifficultyfilter: i32,
    #[serde_as(as = "BoolFromInt")] pub preview: bool,
    pub disablesubtitle: i32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Skin {
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
    pub disableimagefont: i32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Theme {
}

#[derive(Serialize, Deserialize, Default)]
pub struct Play9B {
}

#[derive(Serialize, Deserialize, Default)]
pub struct Network {
    pub lr1ir: i32,
    pub lr1id: String,
    pub lr1pass: String,
    pub lr2ir: i32,
    pub mail: Mail,
    pub autoupdate: i32,
    pub getrival: i32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Mail {
}

#[derive(Serialize, Deserialize, Default)]
pub struct Course {
    pub optimumlevel_7: i32,
    pub optimumlevel_5: i32,
    pub optimumlevel_10: i32,
    pub optimumlevel_14: i32,
    pub optimumlevel_9: i32,
    pub defaultconnection: i32,
    pub defaultgauge: i32,
    pub maxbpm: i32,
    pub minbpm: i32,
    pub bpmrange: i32,
    pub maxlevel: i32,
    pub minlevel: i32,
    pub stage: i32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Tools {
    pub bmse_body: String,
    pub bmse_option: BmseOption,
    pub movie_audio: i32,
    pub movie_framerate: i32,
    pub mp3_volume: i32,
    pub mp3enc_body: String,
    pub mp3enc_option_movie: String,
    pub mp3enc_option_normal: String,
    pub oggdec_body: String,
    pub oggdec_option: OggdecOption,
    pub oggenc_body: String,
    pub oggenc_option: String,
    pub autowavtoogg: i32,
    pub autobmptopng: i32,
    pub autofumensearch: i32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BmseOption {
}

#[derive(Serialize, Deserialize, Default)]
pub struct OggdecOption {
}

impl Default for Config {
    fn default() -> Config {
        Config {
            system: System {
                autoreload: 2,
                mainsleep: 3,
                bmssleep: 3,
                screenexrate: 100,
                inputinterval: 16,
                titleflash: 24,
                windowsize_x: 640,
                windowsize_y: 480,
                newsongfolder: "NEW SONG\\".into(),
                ..Default::default()
            },
            jukebox: Jukebox::default(),
            play: Play {
                hs: 100,
                hsmax: 900,
                hsmin: 10,
                hsmargin: 10,
                shuttermargin: 10,
                bga: 1,
                poorbga: 500,
                defaulttarget: 90,
                basespeed: 100,
                ..Default::default()
            },
            sound: Sound {
                bufferlength: 384,
                numbuffers: 4,
                disabledsp: 1,
                volumeflag: 1,
                volumebgm: 100,
                volumekey: 100,
                volumemaster: 100,
                ..Default::default()
            },
            player: Player::default(),
            select: ConfigSelect {
                key: 1,
                searchmax: 1000,
                speedfirst: 300,
                speednext: 70,
                levelbarflash_7: 12,
                levelbarflash_5: 9,
                levelbarflash_9: 42,
                preview: true,
                ..Default::default()
            },
            skin: Skin {
                fontname: "Ariel".into(),
                ..Default::default()
            },
            network: Network::default(),
            course: Course {
                bpmrange: 10,
                stage: 5,
                ..Default::default()
            },
            tools: Tools {
                bmse_body: "bmse.exe".into(),
                movie_framerate: 30,
                mp3_volume: 50,
                mp3enc_body: "lame.exe".into(),
                mp3enc_option_movie: "--preset cbr 192".into(),
                mp3enc_option_normal: "--preset fast extreme".into(),
                oggdec_body: "oggdec.exe".into(),
                oggenc_body: "oggenc2.exe".into(),
                oggenc_option: "-q 6".into(),
                ..Default::default()
            }
        }
    }
}