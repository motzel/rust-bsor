pub mod error;
mod header;
mod read_utils;

use crate::replay::error::BsorError;
use crate::replay::header::Header;
use std::io::Read;

#[derive(Debug)]
pub struct Replay {
    pub version: u8,
}

impl Replay {
    pub fn load<R: Read>(r: &mut R) -> Result<Replay, BsorError> {
        let header = Header::load(r)?;

        Ok(Replay {
            version: header.version,
        })
    }
}
