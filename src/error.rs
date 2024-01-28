use std::{
    error::Error,
    fmt,
};

/// The `PalconError` enum represents a set of possible errors that can occur using the RCON features of this library.
/// Each variant represents a different kind of error.
#[derive(Debug)]
pub enum PalconError {
    /// Represents an I/O operation error.
    IoError(std::io::Error),
    /// Represents an error that occurs when a sequence of bytes is not valid UTF-8.
    Utf8Error(std::str::Utf8Error),
    /// Represents a timeout error.
    TimeoutError,
    /// Represents an error that occurs when a connection ends unexpectedly.
    ConnectionEnded,
    /// Represents an error that occurs when the application fails to read a response.
    FailedToReadResponse,
    /// Represents an error that occurs when authentication fails.
    AuthenticationError,
    /// Represents an error that occurs when the application is already authenticated.
    AlreadyAuthenticated,
}

/// Implements the `Display` trait for `PalconError`.
/// This provides a human-readable description of the error.
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

/// Implements the `Error` trait for `PalconError`.
/// This provides a method to get the lower-level source of the error, if any.
impl Error for PalconError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::IoError(ref err) => Some(err),
            Self::Utf8Error(ref err) => Some(err),
            _ => None,
        }
    }
}

/// Implements the `From` trait for `std::io::Error`.
/// This allows a `std::io::Error` to be converted into a `PalconError`.
impl From<std::io::Error> for PalconError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

/// Implements the `From` trait for `std::str::Utf8Error`.
/// This allows a `std::str::Utf8Error` to be converted into a `PalconError`.
impl From<std::str::Utf8Error> for PalconError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}