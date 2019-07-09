use snafu::{Backtrace, ErrorCompat, Snafu};
use std::io;
use std::result;
use std::string;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("io error: {}", source))]
    IO {
        source: io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("failed to decode utf8: {}", source))]
    UTF8Decode {
        source: string::FromUtf8Error,
        backtrace: Backtrace,
    },

    #[snafu(display("curl error: {}", source))]
    Curl {
        source: curl::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("rmp encode error: {}", source))]
    RMPEncode {
        source: rmp_serde::encode::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("rmp decode error: {}", source))]
    RMPDecode {
        source: rmp_serde::decode::Error,
        backtrace: Backtrace,
    },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO {
            source: err,
            backtrace: Backtrace::new(),
        }
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Self {
        Error::UTF8Decode {
            source: err,
            backtrace: Backtrace::new(),
        }
    }
}

impl From<curl::Error> for Error {
    fn from(err: curl::Error) -> Self {
        Error::Curl {
            source: err,
            backtrace: Backtrace::new(),
        }
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Error::RMPEncode {
            source: err,
            backtrace: Backtrace::new(),
        }
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(err: rmp_serde::decode::Error) -> Self {
        Error::RMPDecode {
            source: err,
            backtrace: Backtrace::new(),
        }
    }
}

pub fn display_error(err: Error) {
    eprintln!("{}", err);

    if let Some(backtrace) = err.backtrace() {
        eprintln!("backtrace:\n{}", backtrace);
    }
}
