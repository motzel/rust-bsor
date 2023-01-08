use std::{error, fmt, io};

/// All possible error variants when parsing a BSOR replay
#[derive(Debug)]
pub enum BsorError {
    /// Invalid BSOR, i.e. magic variable is invalid
    InvalidBsor,
    /// BSOR version is unsupported. Enum value contains BSOR version
    UnsupportedVersion(u8),
    /// IO error. Enum value contains concrete [io::Error]
    Io(io::Error),
}

impl fmt::Display for BsorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BsorError::InvalidBsor => write!(f, "invalid bsor"),
            BsorError::UnsupportedVersion(v) => write!(f, "invalid bsor version ({})", v),
            BsorError::Io(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for BsorError {
    fn from(error: io::Error) -> Self {
        BsorError::Io(error)
    }
}

impl error::Error for BsorError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self {
            BsorError::InvalidBsor => None,
            BsorError::UnsupportedVersion(_) => None,
            BsorError::Io(e) => e.source(),
        }
    }
}
