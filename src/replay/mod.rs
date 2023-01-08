pub mod error;
pub mod frame;
mod header;
pub mod info;
mod read_utils;
pub mod vector;

pub use error::BsorError;
pub use frame::{Frame, Frames};
use header::Header;
pub use info::Info;
use std::io::Read;

#[derive(Debug)]
pub struct Replay {
    pub version: u8,
    pub info: Info,
    pub frames: Frames,
}

impl Replay {
    pub fn load<R: Read>(r: &mut R) -> Result<Replay, BsorError> {
        let header = Header::load(r)?;
        let info = Info::load(r)?;
        let frames = Frames::load(r)?;

        Ok(Replay {
            version: header.version,
            info,
            frames,
        })
    }
}
