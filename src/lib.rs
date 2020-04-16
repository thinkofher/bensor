pub mod lexer {
    use std::cmp::PartialEq;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Token {
        Dictionary,
        List,
        Integer(i32),
        ByteString(String),
        End,
    }

    impl Token {
        pub fn shift(self) -> usize {
            calc_shift(self)
        }
    }

    pub fn parse(slice: &[u8]) -> Vec<Token> {
        let mut index = 0;
        let mut ret = Vec::new();
        loop {
            match tokenize(&slice[index..]) {
                Some(token) => {
                    index += token.clone().shift();
                    ret.push(token);
                }
                None => break ret,
            }
        }
    }

    fn calc_shift(token: Token) -> usize {
        match token {
            Token::Dictionary | Token::List | Token::End => 1, // single characters
            Token::Integer(num) => str_len(num) + 2,           // with 'i' and 'e'

            // size of string + size of bytes that contains length information + size of ':'
            Token::ByteString(string) => string.len() + str_len(string.len()) + 1,
        }
    }

    fn read_until(slice: &[u8], end: char) -> Vec<u8> {
        slice
            .into_iter()
            .take_while(|&c| *c as char != end)
            .cloned()
            .collect()
    }

    fn read_int(slice: &[u8]) -> Result<i32, std::num::ParseIntError> {
        read_until(slice, 'e')
            .into_iter()
            .map(|c| c as char)
            .collect::<String>()
            .parse()
    }

    fn read_len(slice: &[u8]) -> Result<usize, std::num::ParseIntError> {
        read_until(slice, ':')
            .into_iter()
            .map(|c| c as char)
            .collect::<String>()
            .parse()
    }

    fn str_len(d: impl std::fmt::Display) -> usize {
        format!("{}", d).chars().count()
    }

    fn read_byte_string(slice: &[u8]) -> Option<String> {
        let size = match read_len(slice) {
            Ok(size) => size,
            Err(_) => return None,
        };
        let shift = str_len(size) + 1;
        let shifted_slice = &slice[shift..shift + size];

        Some(shifted_slice.into_iter().map(|&c| c as char).collect())
    }

    const DICTIONARY_BYTE: char = 'd';
    const LIST_BYTE: char = 'l';
    const END_BYTE: char = 'e';
    const INTEGER_BYTE: char = 'i';
    const SLICE_RANGE_START: char = '0';
    const SLICE_RANGE_END: char = '9';

    fn tokenize(slice: &[u8]) -> Option<Token> {
        match slice.first() {
            Some(byte) => match *byte as char {
                DICTIONARY_BYTE => Token::Dictionary.into(),
                LIST_BYTE => Token::List.into(),
                END_BYTE => Token::End.into(),
                INTEGER_BYTE => match read_int(&slice[1..]) {
                    Ok(num) => Token::Integer(num).into(),
                    _ => None,
                },
                SLICE_RANGE_START..=SLICE_RANGE_END => {
                    read_byte_string(slice).map(Token::ByteString)
                }
                _ => None,
            },
            None => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_calc_shift() {
            let size = calc_shift(Token::Integer(-666));
            assert_eq!(size, 6);

            let size = calc_shift(Token::Integer(666));
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
            let bytes = "5:abcdefgh".as_bytes();
            assert_eq!(read_byte_string(bytes), Some(String::from("abcde")));
        }

        #[test]
        fn test_tokenize_int() {
            let bytes = "i1234e".as_bytes();

            let left = tokenize(bytes).unwrap();
            let right = Token::Integer(1234);

            assert_eq!(left, right);
        }

        #[test]
        fn test_tokenize_byte_string() {
            let bytes = "6:abcdefgh".as_bytes();
            let left = tokenize(bytes).unwrap();
            let right = Token::ByteString("abcdef".into());

            assert_eq!(left, right);
        }

        #[test]
        fn test_parse() {
            let bytes = "d3:bar4:spam3:fooi42ee".as_bytes();
            let left = parse(bytes);
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
}

pub mod parser {
    use crate::lexer::Token;
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Bencode {
        Integer(i32),
        ByteString(String),
        List(Vec<Bencode>),
        Dictionary(HashMap<String, Bencode>),
    }

    pub fn parse(tokens: Vec<Token>) -> Option<Bencode> {
        let mut tokens: Vec<Token> = tokens.into_iter().rev().collect();
        match tokens.pop() {
            Some(token) => parse_token(token, &mut tokens),
            None => None,
        }
    }

    fn parse_token(t: Token, tokens: &mut Vec<Token>) -> Option<Bencode> {
        match t {
            Token::Dictionary => parse_dict(tokens, HashMap::new()),
            Token::List => parse_list(tokens, Vec::new()),
            Token::Integer(val) => Bencode::Integer(val).into(),
            Token::ByteString(val) => Bencode::ByteString(val).into(),
            _ => None,
        }
    }

    fn parse_list(tokens: &mut Vec<Token>, list: Vec<Bencode>) -> Option<Bencode> {
        match tokens.pop() {
            Some(Token::End) => Bencode::List(list).into(),
            Some(token) => {
                let mut list = list;
                list.push(parse_token(token, tokens)?);
                parse_list(tokens, list)
            }
            None => None,
        }
    }

    fn parse_dict(tokens: &mut Vec<Token>, dict: HashMap<String, Bencode>) -> Option<Bencode> {
        let key = match tokens.pop() {
            Some(Token::ByteString(key)) => key,
            Some(Token::End) => return Bencode::Dictionary(dict).into(),
            _ => return None,
        };

        let val = match tokens.pop() {
            Some(token) => parse_token(token, tokens)?,
            None => return None,
        };

        let mut dict = dict;
        dict.insert(key, val);

        parse_dict(tokens, dict)
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

            let mut dict = HashMap::new();
            dict.insert("bar".into(), Bencode::ByteString("spam".into()));
            dict.insert("foo".into(), Bencode::Integer(42));
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

            let mut dict = HashMap::new();
            dict.insert("bar".into(), Bencode::ByteString("spam".into()));
            let mut nested_dict = HashMap::new();
            nested_dict.insert("nested".into(), Bencode::Integer(-123));
            dict.insert("foo".into(), Bencode::Dictionary(nested_dict));

            let right = Bencode::Dictionary(dict);
            assert_eq!(left, right);
        }
    }
}
