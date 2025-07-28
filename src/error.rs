use std::fmt;
use std::io;

#[derive(Debug)]
pub enum MarkrsError {
    IO(io::Error),
}

impl fmt::Display for MarkrsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarkrsError::IO(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<io::Error> for MarkrsError {
    fn from(error: io::Error) -> Self {
        MarkrsError::IO(error)
    }
}

