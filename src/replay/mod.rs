pub mod error;
pub mod frame;
mod header;
pub mod height;
pub mod info;
pub mod note;
pub mod pause;
mod read_utils;
pub mod vector;
pub mod wall;

pub use error::BsorError;
use frame::Frames;
use header::Header;
use height::Heights;
use info::Info;
use note::Notes;
use pause::Pauses;
use std::io::Read;
use wall::Walls;

pub type ReplayInt = i32;
pub type ReplayLong = u64;
pub type ReplayFloat = f32;
pub type ReplayTime = ReplayFloat;
pub type LineValue = u8;

pub type Result<T> = std::result::Result<T, BsorError>;

#[derive(Debug)]
pub struct Replay {
    pub version: u8,
    pub info: Info,
    pub frames: Frames,
    pub notes: Notes,
    pub walls: Walls,
    pub heights: Heights,
    pub pauses: Pauses,
}

impl Replay {
    pub fn load<R: Read>(r: &mut R) -> Result<Replay> {
        let header = Header::load(r)?;
        let info = Info::load(r)?;
        let frames = Frames::load(r)?;
        let notes = Notes::load(r)?;
        let walls = Walls::load(r)?;
        let heights = Heights::load(r)?;
        let pauses = Pauses::load(r)?;

        Ok(Replay {
            version: header.version,
            info,
            frames,
            notes,
            walls,
            heights,
            pauses,
        })
    }
}

#[derive(Debug)]
pub struct ParsedReplay {
    pub version: u8,
    info: Info,
}

impl ParsedReplay {
    pub fn parse<R: Read>(r: &mut R) -> Result<ParsedReplay> {
        let header = Header::load(r)?;
        let info = Info::load(r)?;

        Ok(ParsedReplay {
            version: header.version,
            info,
        })
    }

    pub fn get_info(&self) -> &Info {
        &self.info
    }
}
