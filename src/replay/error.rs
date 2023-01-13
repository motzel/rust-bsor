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
    DecodingError(Box<dyn error::Error>),
}

impl fmt::Display for BsorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BsorError::InvalidBsor => write!(f, "invalid bsor"),
            BsorError::UnsupportedVersion(v) => write!(f, "invalid bsor version ({})", v),
            BsorError::Io(e) => write!(f, "io error: {}", e),
            BsorError::DecodingError(e) => write!(f, "decoding error: {}", e),
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
        BsorError::DecodingError(Box::new(error))
    }
}

impl From<Utf8Error> for BsorError {
    fn from(error: Utf8Error) -> Self {
        BsorError::DecodingError(Box::new(error))
    }
}

impl From<TryFromSliceError> for BsorError {
    fn from(error: TryFromSliceError) -> Self {
        BsorError::DecodingError(Box::new(error))
    }
}

impl error::Error for BsorError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self {
            BsorError::InvalidBsor => None,
            BsorError::UnsupportedVersion(_) => None,
            BsorError::Io(e) => Some(e),
            BsorError::DecodingError(e) => e.source(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_convert_io_error_to_bsor_error() {
        let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "Test error");

        match BsorError::try_from(io_err) {
            Ok(BsorError::Io(_)) => {}
            _ => panic!("conversion error"),
        };
    }

    #[test]
    fn it_can_convert_parse_int_error_to_bsor_error() {
        let val = "invalid".parse::<i32>();
        match BsorError::try_from(val.expect_err("conversion error")) {
            Ok(BsorError::DecodingError(_)) => assert!(true),
            _ => panic!("conversion error"),
        };
    }

    #[test]
    fn it_can_convert_parse_utf8_error_to_bsor_error() {
        let val = std::str::from_utf8(&[0xffu8, 0xff]);

        match BsorError::try_from(val.expect_err("conversion error")) {
            Ok(BsorError::DecodingError(_)) => assert!(true),
            _ => panic!("conversion error"),
        };
    }

    #[test]
    fn it_can_convert_from_slice_error_to_bsor_error() {
        let arr: &[u8] = &[0u8];
        let val: Result<[u8; 4], TryFromSliceError> = arr.try_into();

        match BsorError::try_from(val.expect_err("conversion error")) {
            Ok(BsorError::DecodingError(_)) => assert!(true),
            _ => panic!("conversion error"),
        };
    }

    #[test]
    fn it_can_get_source_from_bsor_error() {
        let err: Box<dyn error::Error> = Box::new(BsorError::InvalidBsor);
        err.source();

        let err: Box<dyn error::Error> = Box::new(BsorError::UnsupportedVersion(1));
        err.source();

        let err: Box<dyn error::Error> = Box::new(BsorError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Test error",
        )));
        err.source();

        let err: Box<dyn error::Error> =
            Box::new(BsorError::DecodingError(Box::new(BsorError::InvalidBsor)));
        err.source();

        assert!(true);
    }

    #[test]
    fn it_can_format_output_string_bsor_error() {
        let err: Box<dyn error::Error> = Box::new(BsorError::InvalidBsor);
        assert_eq!(format!("{}", err), "invalid bsor");

        let err: Box<dyn error::Error> = Box::new(BsorError::UnsupportedVersion(1));
        assert_eq!(format!("{}", err), "invalid bsor version (1)");

        let err: Box<dyn error::Error> = Box::new(BsorError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Test error",
        )));
        assert_eq!(format!("{}", err), "io error: Test error");

        let err: Box<dyn error::Error> = Box::new(BsorError::DecodingError(Box::new(
            io::Error::new(io::ErrorKind::UnexpectedEof, "Test error"),
        )));
        assert_eq!(format!("{}", err), "decoding error: Test error");
    }
}
