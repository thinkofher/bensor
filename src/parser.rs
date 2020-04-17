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
        Token::Dictionary => parse_dict(tokens, &mut HashMap::new()),
        Token::List => parse_list(tokens, &mut Vec::new()),
        Token::Integer(val) => Bencode::Integer(val).into(),
        Token::ByteString(val) => Bencode::ByteString(val).into(),
        _ => None,
    }
}

fn parse_list(tokens: &mut Vec<Token>, list: &mut Vec<Bencode>) -> Option<Bencode> {
    match tokens.pop() {
        Some(Token::End) => Bencode::List(list.clone()).into(),
        Some(token) => {
            list.push(parse_token(token, tokens)?);
            parse_list(tokens, list)
        }
        None => None,
    }
}

fn parse_dict(tokens: &mut Vec<Token>, dict: &mut HashMap<String, Bencode>) -> Option<Bencode> {
    match tokens.pop() {
        Some(Token::ByteString(key)) => {
            let val = parse_token(tokens.pop()?, tokens)?;
            dict.insert(key, val);
            parse_dict(tokens, dict)
        }
        Some(Token::End) => return Bencode::Dictionary(dict.clone()).into(),
        _ => return None,
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
