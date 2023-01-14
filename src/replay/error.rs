use std::array::TryFromSliceError;
use std::num::ParseIntError;
use std::str::Utf8Error;
use std::{error, fmt, io};

/// All possible error variants when parsing a BSOR replay
#[derive(Debug)]
pub enum BsorError {
    /// Invalid BSOR, i.e. the magic variable is invalid, or there was an error in the structure of the BSOR
    InvalidBsor,
    /// BSOR version is unsupported. Enum value contains BSOR version
    UnsupportedVersion(u8),
    /// IO error. Enum value contains concrete [io::Error]
    Io(io::Error),
    /// Decoding error
    Decoding(Box<dyn error::Error>),
}

impl fmt::Display for BsorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BsorError::InvalidBsor => write!(f, "invalid bsor"),
            BsorError::UnsupportedVersion(v) => write!(f, "invalid bsor version ({})", v),
            BsorError::Io(e) => write!(f, "io error: {}", e),
            BsorError::Decoding(e) => write!(f, "decoding error: {}", e),
        }
    }
}

impl From<io::Error> for BsorError {
    fn from(error: io::Error) -> Self {
        BsorError::Io(error)
    }
}

impl From<ParseIntError> for BsorError {
    fn from(error: ParseIntError) -> Self {
        BsorError::Decoding(Box::new(error))
    }
}

impl From<Utf8Error> for BsorError {
    fn from(error: Utf8Error) -> Self {
        BsorError::Decoding(Box::new(error))
    }
}

impl From<TryFromSliceError> for BsorError {
    fn from(error: TryFromSliceError) -> Self {
        BsorError::Decoding(Box::new(error))
    }
}

impl error::Error for BsorError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self {
            BsorError::InvalidBsor => None,
            BsorError::UnsupportedVersion(_) => None,
            BsorError::Io(e) => Some(e),
            BsorError::Decoding(e) => {
                if let Some(err) = e.downcast_ref::<ParseIntError>() {
                    return Some(err);
                } else if let Some(err) = e.downcast_ref::<TryFromSliceError>() {
                    return Some(err);
                } else if let Some(err) = e.downcast_ref::<Utf8Error>() {
                    return Some(err);
                }

                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn it_can_convert_io_error_to_bsor_error() {
        let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "Test error");

        let err = BsorError::try_from(io_err);
        assert!(matches!(err, Ok(BsorError::Io(_))));
        assert!(err.unwrap().source().unwrap().is::<io::Error>());
    }

    #[test]
    fn it_can_convert_parse_int_error_to_bsor_error() {
        let val = "invalid".parse::<i32>();

        let err = BsorError::try_from(val.expect_err("conversion error"));
        assert!(matches!(err, Ok(BsorError::Decoding(_))));
        assert!(err.unwrap().source().unwrap().is::<ParseIntError>());
    }

    #[test]
    fn it_can_convert_parse_utf8_error_to_bsor_error() {
        let val = std::str::from_utf8(&[0xffu8, 0xff]);

        let err = BsorError::try_from(val.expect_err("conversion error"));
        assert!(matches!(err, Ok(BsorError::Decoding(_))));
        assert!(err.unwrap().source().unwrap().is::<Utf8Error>());
    }

    #[test]
    fn it_can_convert_from_slice_error_to_bsor_error() {
        let arr: &[u8] = &[0u8];
        let val: Result<[u8; 4], TryFromSliceError> = arr.try_into();

        let err = BsorError::try_from(val.expect_err("conversion error"));
        assert!(matches!(err, Ok(BsorError::Decoding(_))));
        assert!(err.unwrap().source().unwrap().is::<TryFromSliceError>());
    }

    #[test]
    fn it_can_get_source_from_bsor_error() {
        let err: Box<dyn Error> = Box::new(BsorError::InvalidBsor);
        assert!(matches!(err.source(), None));

        let err: Box<dyn Error> = Box::new(BsorError::UnsupportedVersion(1));
        assert!(matches!(err.source(), None));
    }

    #[test]
    fn it_can_format_output_string_bsor_error() {
        let err: Box<dyn Error> = Box::new(BsorError::InvalidBsor);
        assert_eq!(format!("{}", err), "invalid bsor");

        let err: Box<dyn Error> = Box::new(BsorError::UnsupportedVersion(1));
        assert_eq!(format!("{}", err), "invalid bsor version (1)");

        let err: Box<dyn Error> = Box::new(BsorError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Test error",
        )));
        assert_eq!(format!("{}", err), "io error: Test error");

        let err: Box<dyn Error> = Box::new(BsorError::Decoding(Box::new(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Test error",
        ))));
        assert_eq!(format!("{}", err), "decoding error: Test error");
    }
}
