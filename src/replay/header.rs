use super::{error::BsorError, read_utils, Result};
use crate::replay::BSOR_MAGIC;
use std::io::Read;

pub(crate) struct Header {
    pub version: u8,
}

impl Header {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Header> {
        let magic = read_utils::read_int(r)?;
        let version = read_utils::read_byte(r)?;

        if magic != BSOR_MAGIC {
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
    use crate::replay::{ReplayInt, BSOR_MAGIC};
    use rand::random;
    use std::io::Cursor;

    #[test]
    fn it_return_error_when_header_magic_is_invalid() {
        let mut buf = ReplayInt::to_le_bytes(BSOR_MAGIC + 1).to_vec();
        buf.push(1);

        let result = Header::load(&mut Cursor::new(buf));

        assert!(matches!(result, Err(BsorError::InvalidBsor)));
    }

    #[test]
    fn it_return_error_when_header_version_is_invalid() {
        let invalid_version = random::<u8>();

        let mut buf = ReplayInt::to_le_bytes(BSOR_MAGIC).to_vec();
        buf.push(invalid_version);

        let result = Header::load(&mut Cursor::new(buf));

        assert!(matches!(result, Err(BsorError::UnsupportedVersion(_))));

        let reported_version = match result {
            Ok(_) => panic!("should be error!"),
            Err(BsorError::UnsupportedVersion(v)) => v,
            _ => panic!("invalid error"),
        };

        assert_eq!(invalid_version, reported_version)
    }

    #[test]
    fn it_can_load_header() -> Result<()> {
        let mut buf = ReplayInt::to_le_bytes(BSOR_MAGIC).to_vec();
        buf.push(1);

        let file = &mut Cursor::new(buf);
        let result = Header::load(file);

        assert!(!result.is_err());
        assert_eq!(result?.version, 1);

        Ok(())
    }
}
