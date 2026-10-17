use std::fmt;

/// Failure to create a platform keyboard input source.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Input devices exist, but the process cannot read them.
    PermissionDenied,
    /// No readable keyboard devices were found.
    NoDevices,
    /// A platform listener could not be created.
    Platform(String),
    /// An operating-system I/O operation failed.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => f.write_str("permission denied while opening keyboard input devices"),
            Self::NoDevices => f.write_str("no readable keyboard input devices found"),
            Self::Platform(message) => write!(f, "keyboard input creation failed: {message}"),
            Self::Io(error) => write!(f, "keyboard input I/O error: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
