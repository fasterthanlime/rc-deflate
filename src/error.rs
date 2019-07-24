use crate::parse;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Parse(String),
    IncompleteInput,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rc-flate error: {:?}", self)
    }
}

impl std::error::Error for Error {}

impl<'a> From<parse::Error<'a>> for Error {
    fn from(e: parse::Error) -> Self {
        let (_, kind) = e;
        Error::Parse(format!("{:?}", kind))
    }
}
