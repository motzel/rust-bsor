use super::error::BsorError;
use super::read_utils;
use std::io::Read;

pub(crate) struct Header {
    pub version: u8,
}

impl Header {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Header, BsorError> {
        let magic = read_utils::read_int(r)?;
        let version = read_utils::read_byte(r)?;

        if magic != 0x442d3d69 {
            return Err(BsorError::InvalidBsor);
        }

        if version != 1 {
            return Err(BsorError::UnsupportedVersion(version));
        }

        Ok(Self { version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_return_error_when_header_magic_is_invalid() {
        let result = Header::load(&mut Cursor::new([0x68, 0x3d, 0x2d, 0x44, 1]));

        assert!(matches!(result, Err(BsorError::InvalidBsor)));
    }

    #[test]
    fn it_return_error_when_header_version_is_invalid() {
        let result = Header::load(&mut Cursor::new([0x69, 0x3d, 0x2d, 0x44, 10]));

        assert!(matches!(result, Err(BsorError::UnsupportedVersion(_))));
    }

    #[test]
    fn it_can_load_header() {
        let file = &mut Cursor::new([0x69, 0x3d, 0x2d, 0x44, 1]);
        let result = Header::load(file);

        assert!(!result.is_err());
    }
}
