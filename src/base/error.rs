use core::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseMap {
        path: PathBuf,
        line: usize,
        msg: &'static str,
    },
    ParseScen {
        path: PathBuf,
        line: usize,
        msg: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "failed to read {}: {source}", path.display()),
            Error::ParseMap { path, line, msg } => {
                write!(f, "{}:{line}: invalid map: {msg}", path.display())
            }
            Error::ParseScen { path, line, msg } => {
                write!(f, "{}:{line}: invalid scenario: {msg}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
