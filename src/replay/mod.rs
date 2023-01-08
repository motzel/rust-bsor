pub mod error;
mod header;
pub mod info;
mod read_utils;

use crate::replay::error::BsorError;
use crate::replay::header::Header;
use crate::replay::info::Info;
use std::io::Read;

#[derive(Debug)]
pub struct Replay {
    pub version: u8,
    pub info: Info,
}

impl Replay {
    pub fn load<R: Read>(r: &mut R) -> Result<Replay, BsorError> {
        let header = Header::load(r)?;
        let info = Info::load(r)?;

        Ok(Replay {
            version: header.version,
            info,
        })
    }
}
