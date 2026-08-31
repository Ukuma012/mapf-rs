use core::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ParseMap { path: PathBuf },
    ParseScen { path: PathBuf },
    InvalidInstance,
    NotEnoughAgents { requested: usize, available: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{e}"),
            Error::ParseMap { path } => write!(f, "{}: invalid map file", path.display()),
            Error::ParseScen { path } => write!(f, "{}: invalid scen file", path.display()),
            Error::InvalidInstance => write!(f, "invalid instance"),
            Error::NotEnoughAgents {
                requested,
                available,
            } => write!(
                f,
                "requested {requested} agents but only {available} available"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
