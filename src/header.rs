use crate::fundamental_types::FileTime;
use crate::reader::{SaveFileReader, read_filetime};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct Header {
    pub version: u32,
    pub save_number: u32,
    pub player_name: String,
    pub player_level: u32,
    pub player_location: String,
    pub game_date: String,
    pub player_race_editor_id: String,
    pub player_sex: PlayerSex,
    pub player_cur_exp: f32,
    pub player_lvl_up_exp: f32,
    pub filetime: FileTime,
    pub shot_width: u32,
    pub shot_height: u32,
    pub compression_type: u16,
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerSex {
    Male,
    Female
}

impl From<u16> for PlayerSex {
    fn from(x: u16) -> Self {
       match x {
           0 => PlayerSex::Male,
           1 => PlayerSex::Female,
           _ => {
               println!("Error while reading playerSex");
               PlayerSex::Female
           }
       }
    }
}

impl Display for PlayerSex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*format!("{:?}", self))
    }
}

pub fn read_header(sfr: &mut SaveFileReader) -> Header {
    Header {
        version: sfr.read_u32(),
        save_number: sfr.read_u32(),
        player_name: sfr.read_w_string().content,
        player_level: sfr.read_u32(),
        player_location: sfr.read_w_string().content,
        game_date: sfr.read_w_string().content,
        player_race_editor_id: sfr.read_w_string().content,
        player_sex: sfr.read_u16().into(),
        player_cur_exp: sfr.read_f32(),
        player_lvl_up_exp: sfr.read_f32(),
        filetime: read_filetime(sfr),
        shot_width: sfr.read_u32(),
        shot_height: sfr.read_u32(),
        compression_type: sfr.read_u16(),
    }
}