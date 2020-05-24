//! Bensor is a simple and minimal library for parsing [bencode](https://en.wikipedia.org/wiki/Bencode)
//! encoding, written in pure Rust with zero dependencies. Bensor provides high level API to
//! use with TryInto and TryFrom traits from Rust standard library.
//!
//! # Examples
//!
//! Parsing integer from slice of bytes.
//!
//! ```
//! use std::convert::TryInto;
//! use bensor::Bencode;
//!
//! let slice: &[u8] = &[105, 51, 50, 101];
//! let integer: Bencode = slice.try_into().unwrap();
//! assert_eq!(integer, Bencode::Integer(32));
//! ```
//!
//! Parsing list from literal string.
//! ```
//! use std::convert::TryInto;
//! use bensor::Bencode;
//!
//! let left: Bencode = "l3:loli100e4:ruste".try_into().unwrap();
//! let right = {
//!     let mut res = Vec::new();
//!     res.push(Bencode::ByteString(String::from("lol")));
//!     res.push(Bencode::Integer(100));
//!     res.push(Bencode::ByteString(String::from("rust")));
//!     Bencode::List(res)
//! };
//! assert_eq!(left, right)
//! ```

mod lexer;
mod parser;

use std::convert::TryFrom;
use std::{error, fmt};

pub use parser::Bencode;

/// Error wrapper for errors from both lexer and parser modules.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Parser(parser::Error),
    Lexer(lexer::Error),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Parser(err) => Some(err),
            Error::Lexer(err) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parser(err) => {
                write!(f, "Parses Error: ")?;
                err.fmt(f)
            }
            Error::Lexer(err) => {
                write!(f, "Lexer Error: ")?;
                err.fmt(f)
            }
        }
    }
}

fn from_bytes(data: &[u8]) -> Result<Bencode, Error> {
    let tokens = lexer::parse(data).map_err(Error::Lexer)?;
    parser::parse(tokens).map_err(Error::Parser)
}

impl<'a> TryFrom<&'a [u8]> for Bencode {
    type Error = Error;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        from_bytes(data)
    }
}

fn from_str(data: &str) -> Result<Bencode, Error> {
    from_bytes(data.as_bytes())
}

impl<'a> TryFrom<&'a str> for Bencode {
    type Error = Error;

    fn try_from(data: &'a str) -> Result<Self, Self::Error> {
        from_str(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_from_str() {
        use std::convert::TryInto;
        let left: Bencode = "d3:bar4:spam3:fooi42ee".try_into().unwrap();

        let mut dict = HashMap::new();
        dict.insert("bar".into(), Bencode::ByteString("spam".into()));
        dict.insert("foo".into(), Bencode::Integer(42));
        let right = Bencode::Dictionary(dict);

        assert_eq!(left, right);
    }

    #[test]
    fn test_from_bytes() {
        use std::convert::TryInto;
        let left: Bencode = "l4:spami42ei666e5:tumore".as_bytes().try_into().unwrap();

        let right = Bencode::List(vec![
            Bencode::ByteString("spam".into()),
            Bencode::Integer(42),
            Bencode::Integer(666),
            Bencode::ByteString("tumor".into()),
        ]);

        assert_eq!(left, right);
    }
}
