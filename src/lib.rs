mod lexer;
mod parser;

pub use parser::Bencode;

pub fn from_str(data: &str) -> Option<Bencode> {
    from_bytes(data.as_bytes())
}

pub fn from_bytes(data: &[u8]) -> Option<Bencode> {
    let tokens = lexer::parse(data);
    parser::parse(tokens)
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
