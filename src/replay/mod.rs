pub mod error;
pub mod frame;
mod header;
pub mod info;
pub mod note;
mod read_utils;
pub mod vector;

pub use error::BsorError;
pub use frame::{Frame, Frames};
use header::Header;
pub use info::Info;
pub use note::{Note, Notes};
use std::io::Read;

pub type ReplayInt = i32;
pub type ReplayFloat = f32;
pub type ReplayTime = ReplayFloat;

pub type Result<T> = std::result::Result<T, BsorError>;

#[derive(Debug)]
pub struct Replay {
    pub version: u8,
    pub info: Info,
    pub frames: Frames,
    pub notes: Notes,
}

impl Replay {
    pub fn load<R: Read>(r: &mut R) -> Result<Replay> {
        let header = Header::load(r)?;
        let info = Info::load(r)?;
        let frames = Frames::load(r)?;
        let notes = Notes::load(r)?;

        Ok(Replay {
            version: header.version,
            info,
            frames,
            notes,
        })
    }
}
