//! Parses module contains data structures and procedures
//! related to parsing tokenized input.
use crate::lexer::Token;

use std::collections::HashMap;
use std::{error, fmt};

/// Represents possible complications that can occur during parsing tokenized data.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Empty token container provided.
    NoTokens,
    /// There is end token without need.
    InvalidEndToken,
    /// There is the list without explicit end token.
    NoEndList,
    /// There is a attempt to use type other than ByteString
    /// as key in the dictionary.
    InvalidDictionaryKey,
    /// There is the dictionary without explicit end token.
    NoEndDictionary,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoTokens => write!(f, "There are no tokens in the given vector."),
            Error::InvalidEndToken => write!(f, "Too many end characters in given data."),
            Error::NoEndList => write!(f, "There is list without end character in given data."),
            Error::InvalidDictionaryKey => {
                write!(f, "Dictionaries can only have byte strings as keys.")
            }
            Error::NoEndDictionary => write!(
                f,
                "There is dictionary without end character in given data."
            ),
        }
    }
}

/// Bencode is recursive data structure which
/// works as representation of all possible data
/// that can be encoded with bencoding.
#[derive(Debug, Clone, PartialEq)]
pub enum Bencode {
    /// an be positive or negative.
    Integer(i32),
    /// Fixed-length string of bytes.
    ByteString(String),
    /// List of bencoded values.
    List(Vec<Bencode>),
    /// Associative array where keys can be only strings
    /// and values can be any of bencoding data structures.
    Dictionary(HashMap<String, Bencode>),
}

pub(crate) fn parse(tokens: Vec<Token>) -> Result<Bencode, Error> {
    let mut tokens: Vec<Token> = tokens.into_iter().rev().collect();
    match tokens.pop() {
        Some(token) => parse_token(token, &mut tokens),
        None => Err(Error::NoTokens),
    }
}

fn parse_token(t: Token, tokens: &mut Vec<Token>) -> Result<Bencode, Error> {
    match t {
        Token::Dictionary => parse_dict(tokens, &mut HashMap::new()),
        Token::List => parse_list(tokens, &mut Vec::new()),
        Token::Integer(val) => Ok(Bencode::Integer(val)),
        Token::ByteString(val) => Ok(Bencode::ByteString(val)),
        Token::End => Err(Error::InvalidEndToken),
    }
}

fn parse_list(tokens: &mut Vec<Token>, list: &mut Vec<Bencode>) -> Result<Bencode, Error> {
    match tokens.pop() {
        Some(Token::End) => Ok(Bencode::List(list.clone())),
        Some(token) => {
            list.push(parse_token(token, tokens)?);
            parse_list(tokens, list)
        }
        None => Err(Error::NoEndList),
    }
}

fn parse_dict(
    tokens: &mut Vec<Token>,
    dict: &mut HashMap<String, Bencode>,
) -> Result<Bencode, Error> {
    match tokens.pop() {
        Some(Token::ByteString(key)) => {
            let val = match tokens.pop() {
                Some(token) => parse_token(token, tokens)?,
                None => return Err(Error::NoEndDictionary),
            };
            dict.insert(key, val);
            parse_dict(tokens, dict)
        }
        Some(Token::End) => Ok(Bencode::Dictionary(dict.clone())),
        _ => Err(Error::InvalidDictionaryKey),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;

    #[test]
    fn test_parse_list() {
        let tokens = vec![
            Token::List,
            Token::Integer(55),
            Token::ByteString("str".into()),
            Token::End,
        ];
        let left = parse(tokens).unwrap();
        let right = Bencode::List(vec![
            Bencode::Integer(55),
            Bencode::ByteString("str".into()),
        ]);
        assert_eq!(left, right);
    }

    #[test]
    fn test_parse_nested_list() {
        let tokens = vec![
            Token::List,
            Token::Integer(55),
            Token::List,
            Token::ByteString("str".into()),
            Token::End,
            Token::End,
        ];
        let left = parse(tokens).unwrap();
        let right = Bencode::List(vec![
            Bencode::Integer(55),
            Bencode::List(vec![Bencode::ByteString("str".into())]),
        ]);
        assert_eq!(left, right);
    }

    #[test]
    fn test_parse_dict() {
        let tokens = vec![
            Token::Dictionary,
            Token::ByteString("bar".into()),
            Token::ByteString("spam".into()),
            Token::ByteString("foo".into()),
            Token::Integer(42),
            Token::End,
        ];
        let left = parse(tokens).unwrap();

        let dict = {
            let mut dict = HashMap::new();
            dict.insert("bar".into(), Bencode::ByteString("spam".into()));
            dict.insert("foo".into(), Bencode::Integer(42));
            dict
        };
        let right = Bencode::Dictionary(dict);

        assert_eq!(left, right);
    }

    #[test]
    fn test_parse_nested_dict() {
        let tokens = vec![
            Token::Dictionary,
            Token::ByteString("bar".into()),
            Token::ByteString("spam".into()),
            Token::ByteString("foo".into()),
            Token::Dictionary,
            Token::ByteString("nested".into()),
            Token::Integer(-123),
            Token::End,
            Token::End,
        ];
        let left = parse(tokens).unwrap();

        let dict = {
            let mut dict = HashMap::new();
            dict.insert("bar".into(), Bencode::ByteString("spam".into()));

            let mut nested_dict = HashMap::new();
            nested_dict.insert("nested".into(), Bencode::Integer(-123));

            dict.insert("foo".into(), Bencode::Dictionary(nested_dict));
            dict
        };

        let right = Bencode::Dictionary(dict);
        assert_eq!(left, right);
    }
}
