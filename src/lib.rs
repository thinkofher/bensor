mod lexer;
mod parser;

use std::{error, fmt};

pub use parser::Bencode;

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

pub fn from_str(data: &str) -> Result<Bencode, Error> {
    from_bytes(data.as_bytes())
}

pub fn from_bytes(data: &[u8]) -> Result<Bencode, Error> {
    let tokens = lexer::parse(data).map_err(Error::Lexer)?;
    parser::parse(tokens).map_err(Error::Parser)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_from_str() {
        let data = "d3:bar4:spam3:fooi42ee";
        let left = from_str(data).unwrap();

        let mut dict = HashMap::new();
        dict.insert("bar".into(), Bencode::ByteString("spam".into()));
        dict.insert("foo".into(), Bencode::Integer(42));
        let right = Bencode::Dictionary(dict);

        assert_eq!(left, right);
    }

    #[test]
    fn test_from_bytes() {
        let data = "l4:spami42ei666e5:tumore".as_bytes();
        let left = from_bytes(data).unwrap();

        let right = Bencode::List(vec![
            Bencode::ByteString("spam".into()),
            Bencode::Integer(42),
            Bencode::Integer(666),
            Bencode::ByteString("tumor".into()),
        ]);

        assert_eq!(left, right);
    }
}
