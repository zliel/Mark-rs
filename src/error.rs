use std::fmt;
use std::io;

use crate::thread_pool;

#[derive(Debug)]
pub enum Error {
    ThreadPool(thread_pool::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ThreadPool(e) => write!(f, "Thread pool error: {e}"),
        }
    }
}

    fn from(error: io::Error) -> Self {
    }
}

impl From<thread_pool::Error> for Error {
    fn from(error: thread_pool::Error) -> Self {
        Error::ThreadPool(error)
    }
}
