//! Parses module contains data structures and procedures
//! related to parsing tokenized input.
use crate::lexer::Token;

use std::collections::HashMap;
use std::{error, fmt};

/// Bencode is recursive data structure which
/// works as representation of all possible data
/// that can be encoded with bencoding.
#[derive(Debug, Clone, PartialEq)]
pub enum Bencode {
    /// an be positive or negative.
    Integer(i64),
    /// Fixed-length string of bytes.
    ByteString(String),
    /// List of bencoded values.
    List(Vec<Bencode>),
    /// Associative array where keys can be only strings
    /// and values can be any of bencoding data structures.
    Dictionary(HashMap<String, Bencode>),
}

impl Bencode {
    /// Transforms `Bencode` into owned vector of bencoded bytes.
    ///
    /// # Examples
    ///
    /// Serialization of bencode integer.
    ///
    /// ```
    /// use bensor::Bencode;
    ///
    /// let left = Bencode::Integer(2015).into_bytes();
    /// let right = b"i2015e".to_vec();
    ///
    /// assert_eq!(left, right);
    /// ```
    ///
    /// Serialization of bencode dictionary.
    ///
    /// ```
    /// use bensor::Bencode;
    /// use std::collections::HashMap;
    ///
    /// let left = {
    ///     let mut d = HashMap::new();
    ///     d.insert("one".into(), Bencode::Integer(1));
    ///     d.insert("two".into(), Bencode::Integer(2));
    ///     Bencode::Dictionary(d).into_bytes()
    /// };
    /// let right = b"d3:onei1e3:twoi2ee".to_vec();
    /// assert_eq!(left, right);
    /// ```
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Bencode::Integer(n) => {
                let res = format!("i{}e", n);
                res.into_bytes()
            }
            Bencode::ByteString(s) => {
                let mut res = Vec::new();

                let mut string = s.into_bytes();
                let mut head = {
                    let s = format!("{}:", string.len());
                    s.into_bytes()
                };

                res.append(&mut head);
                res.append(&mut string);
                res
            }
            Bencode::List(vec) => {
                let mut res = Vec::new();
                res.push('l' as u8);
                vec.into_iter()
                    .map(|elem| elem.into_bytes())
                    .for_each(|elem| res.extend_from_slice(&elem));
                res.push('e' as u8);
                res
            }
            Bencode::Dictionary(map) => {
                let mut res = Vec::new();
                res.push('d' as u8);

                let sorted_map = {
                    let mut sorted_map = map.into_iter().collect::<Vec<(String, Bencode)>>();
                    sorted_map
                        .sort_by(|(first_key, _), (second_key, _)| first_key.cmp(&second_key));
                    sorted_map
                };
                sorted_map.into_iter().for_each(|(key, value)| {
                    let mut key_bytes = key.clone().into_bytes();
                    let mut head = {
                        let s = format!("{}:", key_bytes.len());
                        s.into_bytes()
                    };
                    res.append(&mut head);
                    res.append(&mut key_bytes);
                    res.append(&mut value.into_bytes());
                });

                res.push('e' as u8);
                res
            }
        }
    }
}

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

    #[test]
    fn test_list_into_bytes() {
        let left = {
            let mut v = Vec::new();
            v.push(Bencode::Integer(2137));
            v.push(Bencode::ByteString("Hello World!".into()));
            v.push(Bencode::Integer(2020));
            Bencode::List(v)
        }
        .into_bytes();

        let right = b"li2137e12:Hello World!i2020ee".to_vec();
        assert_eq!(left, right);
    }

    #[test]
    fn test_dict_into_bytes() {
        let left = {
            let mut h = HashMap::new();
            h.insert("current_year".into(), Bencode::Integer(2020));
            h.insert("power_level".into(), Bencode::Integer(9001));
            h.insert(
                "some_random_bytes".into(),
                Bencode::ByteString("welcome_my_dear_bytes".into()),
            );
            h.insert(
                "integer_list".into(),
                Bencode::List(vec![
                    Bencode::Integer(5),
                    Bencode::Integer(10),
                    Bencode::Integer(100),
                ]),
            );
            Bencode::Dictionary(h)
        }
        .into_bytes();

        let right = b"d12:current_yeari2020e12:integer_listli5ei10ei100ee11:power_leveli9001e17:some_random_bytes21:welcome_my_dear_bytese".to_vec();
        assert_eq!(left, right);
    }

    #[test]
    fn test_nested_dict_into_bytes() {
        let left = {
            let mut h = HashMap::new();
            let nested_dict = {
                let mut h = HashMap::new();
                h.insert("abc".into(), Bencode::Integer(123));
                h.insert("def".into(), Bencode::Integer(456));
                h
            };
            h.insert("nested".into(), Bencode::Dictionary(nested_dict));
            h.insert(
                "list".into(),
                Bencode::List(vec![
                    Bencode::Integer(1),
                    Bencode::Integer(2),
                    Bencode::Integer(1000),
                ]),
            );
            Bencode::Dictionary(h)
        }
        .into_bytes();

        let right = b"d4:listli1ei2ei1000ee6:nestedd3:abci123e3:defi456eee".to_vec();
        assert_eq!(left, right);
    }
}
