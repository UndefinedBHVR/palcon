use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PalconError {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    TimeoutError,
    ConnectionEnded,
    FailedToReadResponse,
    AuthenticationError,
    AlreadyAuthenticated,
}

impl fmt::Display for PalconError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::IoError(ref err) => write!(f, "IO error: {}", err),
            Self::Utf8Error(ref err) => write!(f, "UTF8 error: {}", err),
            Self::TimeoutError => write!(f, "Read timed out"),
            Self::ConnectionEnded => write!(f, "Ended connection"),
            Self::FailedToReadResponse => write!(f, "Failed to read response"),
            Self::AuthenticationError => write!(f, "Failed to authenticate"),
            Self::AlreadyAuthenticated => write!(f, "Already authenticated"),
        }
    }
}

impl Error for PalconError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::IoError(ref err) => Some(err),
            Self::Utf8Error(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PalconError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<std::str::Utf8Error> for PalconError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}
