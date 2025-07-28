use std::fmt;
use std::io;

use crate::thread_pool;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ThreadPool(thread_pool::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O Error: {e}"),
            Error::ThreadPool(e) => write!(f, "Thread pool error: {e}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<thread_pool::Error> for Error {
    fn from(error: thread_pool::Error) -> Self {
        Error::ThreadPool(error)
    }
}
