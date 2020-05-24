use std::cmp::PartialEq;
use std::{error, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    ReadInt,
    ReadLen,
    ReadByteString,
    ReadFirstByte(char),
    EmptySlice,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ReadInt => write!(f, "Integers can only be composed of numeric characters."),
            Error::ReadLen => write!(f, "Length can only be composed of numberic characters."),
            Error::ReadByteString => write!(f, "The data contains a malformed string of bytes."),
            Error::ReadFirstByte(c) => {
                let msg = format!(
                    "Does not recognize the data structure with this beginning: \"{}\".",
                    c
                );
                f.write_str(msg.as_str())
            }
            Error::EmptySlice => write!(f, "Given slice is empty."),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Dictionary,
    List,
    Integer(i32),
    ByteString(String),
    End,
}

const INTEGER_START: &'static [u8] = b"i";
const INTEGER_END: &'static [u8] = b"e";

impl Token {
    fn shift(&self) -> usize {
        match self {
            Token::Dictionary | Token::List | Token::End => 1, // single characters
            Token::Integer(num) => INTEGER_START.len() + str_len(num) + INTEGER_END.len(),

            // size of bytes that contains length information + size of string + size of ":"
            Token::ByteString(string) => {
                let size = string.as_bytes().len();
                str_len(size) + size + b":".len()
            }
        }
    }
}

pub fn parse(slice: &[u8]) -> Result<Vec<Token>, Error> {
    let mut index = 0;
    let mut ret = Vec::new();
    loop {
        match tokenize(&slice[index..]) {
            Ok(token) => {
                index += token.clone().shift();
                ret.push(token);
            }
            Err(Error::EmptySlice) => break Ok(ret),
            Err(err) => break Err(err),
        }
    }
}

fn read_until(slice: &[u8], end: char) -> Vec<u8> {
    slice
        .into_iter()
        .take_while(|&c| *c as char != end)
        .cloned()
        .collect()
}

fn read_int(slice: &[u8]) -> Result<i32, Error> {
    read_until(slice, 'e')
        .into_iter()
        .map(|c| c as char)
        .collect::<String>()
        .parse()
        .map_err(|_| Error::ReadInt)
}

fn read_len(slice: &[u8]) -> Result<usize, Error> {
    read_until(slice, ':')
        .into_iter()
        .map(|c| c as char)
        .collect::<String>()
        .parse()
        .map_err(|_| Error::ReadLen)
}

fn str_len(d: impl std::fmt::Display) -> usize {
    format!("{}", d).chars().count()
}

const STRING_DELIMETER: &'static [u8] = b":";

fn read_byte_string(slice: &[u8]) -> Result<String, Error> {
    let size = read_len(slice).map_err(|_| Error::ReadByteString)?;
    let shift = str_len(size) + STRING_DELIMETER.len();
    let shifted_slice = &slice[shift..shift + size];

    Ok(shifted_slice.into_iter().map(|&c| c as char).collect())
}

const DICTIONARY_BYTE: char = 'd';
const LIST_BYTE: char = 'l';
const END_BYTE: char = 'e';
const INTEGER_BYTE: char = 'i';
const SLICE_RANGE_START: char = '0';
const SLICE_RANGE_END: char = '9';

fn tokenize(slice: &[u8]) -> Result<Token, Error> {
    match slice.first() {
        Some(byte) => match *byte as char {
            DICTIONARY_BYTE => Ok(Token::Dictionary),
            LIST_BYTE => Ok(Token::List),
            END_BYTE => Ok(Token::End),
            INTEGER_BYTE => read_int(&slice[1..]).map(Token::Integer),
            SLICE_RANGE_START..=SLICE_RANGE_END => read_byte_string(slice).map(Token::ByteString),
            c => Err(Error::ReadFirstByte(c)),
        },
        None => Err(Error::EmptySlice),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_shift() {
        let size = Token::Integer(-666).shift();
        assert_eq!(size, 6);

        let size = Token::Integer(666).shift();
        assert_eq!(size, 5);
    }

    #[test]
    fn test_str_len() {
        assert_eq!(str_len(123), 3);
        assert_eq!(str_len(1234), 4);
        assert_eq!(str_len(12345), 5);
    }

    #[test]
    fn test_read_byte_string() {
        let bytes = b"5:abcdefgh";
        assert_eq!(read_byte_string(bytes), Ok(String::from("abcde")));
    }

    #[test]
    fn test_tokenize_int() {
        let bytes = b"i1234e";

        let left = tokenize(bytes).unwrap();
        let right = Token::Integer(1234);

        assert_eq!(left, right);
    }

    #[test]
    fn test_tokenize_byte_string() {
        let bytes = b"6:abcdefgh";
        let left = tokenize(bytes).unwrap();
        let right = Token::ByteString("abcdef".into());

        assert_eq!(left, right);
    }

    #[test]
    fn test_parse() {
        let bytes = b"d3:bar4:spam3:fooi42ee";
        let left = parse(bytes).unwrap();
        let right = vec![
            Token::Dictionary,
            Token::ByteString("bar".into()),
            Token::ByteString("spam".into()),
            Token::ByteString("foo".into()),
            Token::Integer(42),
            Token::End,
        ];
        assert_eq!(left, right);
    }
}
