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
pub use frame::{Frame, Frames};
use header::Header;
pub use height::{Height, Heights};
pub use info::Info;
pub use note::{Note, Notes};
pub use pause::{Pause, Pauses};
use std::io::Read;
pub use wall::{Wall, Walls};

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
