pub struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Map(Vec<(Value, Value)>),
    List(Vec<Value>),
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    pub fn at_eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    pub fn at(&self, tok: &str) -> bool {
        &self.src[self.pos..self.pos + tok.len()] == tok
    }

    pub fn goto(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn eat(&mut self, tok: &str) -> bool {
        let is_at = self.at(tok);
        if is_at {
            self.goto(self.pos + tok.len());
        }
        is_at
    }

    pub fn eat_ws(&mut self) {
        while !self.at_eof() && self.current().is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    pub fn current(&self) -> char {
        self.src.as_bytes().get(self.pos).copied().unwrap_or(b'\0') as char
    }

    pub fn advance(&mut self) {
        self.goto(self.pos + 1);
    }

    pub fn eat_current(&mut self) -> char {
        let curr = self.current();
        self.advance();
        curr
    }

    pub fn parse_list_or_map(&mut self) -> Value {
        let mut first = true;
        let mut list = Vec::new();
        let mut map = Vec::new();
        let mut is_map = false;
        loop {
            self.eat_ws();
            let has_comma = self.eat(",");
            self.eat_ws();
            if first {
                assert!(!has_comma, ", not allowed before first item")
            }
            if self.eat("}") {
                break;
            }
            if !first {
                assert!(has_comma, "expected , after list item");
            }

            self.eat_ws();
            if self.eat("[") {
                if first {
                    is_map = true;
                } else {
                    assert!(is_map, "can't mix list and map");
                }
            } else {
                assert!(!is_map, "can't mix list and map");
            }
            if is_map {
                let k = self.parse_value();
                self.eat_ws();
                assert!(self.eat("]"), "expected a ]");
                self.eat_ws();
                assert!(self.eat("="), "expected a = after list key");
                let v = self.parse_value();
                map.push((k, v));
            } else {
                list.push(self.parse_value());
            }
            first = false;
        }
        if is_map {
            Value::Map(map)
        } else {
            Value::List(list)
        }
    }

    pub fn parse_string(&mut self) -> String {
        let mut s = String::new();
        while !self.at_eof() && !self.at("\"") {
            if self.eat("\\") {
                let re = match self.eat_current() {
                    '\\' => '\\',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => unimplemented!("unknown escape"),
                };
                s.push(re);
            } else {
                s.push(self.eat_current());
            }
        }
        assert!(self.eat("\""));
        s
    }

    pub fn parse_number(&mut self) -> f64 {
        let start = self.pos;
        while !self.at_eof() {
            let curr = self.current();
            if curr.is_ascii_digit() || curr == '.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.src[start..self.pos].parse().unwrap()
    }

    pub fn parse_value(&mut self) -> Value {
        self.eat_ws();
        if self.eat("{") {
            self.parse_list_or_map()
        } else if self.eat("\"") {
            Value::String(self.parse_string())
        } else {
            Value::Number(self.parse_number())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_parser(text: &str, expect_value: Value) -> bool {
        let mut p = Parser::new(text);
        let parsed_value = p.parse_value();
        expect_value == parsed_value
    }
    impl<'a> From<&'a str> for Value {
        fn from(v: &'a str) -> Self {
            Self::String(v.to_owned())
        }
    }

    impl From<f64> for Value {
        fn from(n: f64) -> Self {
            Self::Number(n)
        }
    }

    macro_rules! val {
        ({
            $($k:tt => $v:tt),*
        }) => {
            Value::Map(vec![$((val!($k), val!($v))),*])
        };
        ([$($va:tt),*]) => {{
            Value::List(vec![$(val!($va)),*])
        }};
        ($s:literal) => {
            Value::from($s)
        }
    }
    #[test]
    fn parse_number() {
        assert!(check_parser("1", val!(1.)));
    }

    #[test]
    fn parse_simple_string() {
        assert!(check_parser(r#""hello""#, val!("hello"),))
    }

    #[test]
    fn parse_string_escape() {
        assert!(check_parser(r#""\n\t\r\n""#, val!("\n\t\r\n"),))
    }

    #[test]
    fn parse_empty_list() {
        assert!(check_parser(r#"{}"#, val!([])))
    }

    #[test]
    fn parse_list_of_numbers() {
        assert!(check_parser(
            r#"{1  , 2, 5,4,  3,2,3}"#,
            val!([1., 2., 5., 4., 3., 2., 3.])
        ))
    }

    #[test]
    fn parse_list_of_string() {
        assert!(check_parser(
            r#"{"abc"   , "cd", "e", "f"}"#,
            val!(["abc", "cd", "e", "f"])
        ))
    }
    #[test]
    fn parse_list_of_lists() {
        assert!(check_parser(r#"{{}, {}, {}}"#, val!([[], [], []])))
    }

    #[test]
    fn parse_list_hetero() {
        assert!(check_parser(
            r#"{{        }, 1       ,     "xyz",       {  1, "bb"} , 2.5 }"#,
            val!([[], 1., "xyz", [1., "bb"], 2.5])
        ))
    }

    #[test]
    fn parse_map() {
        assert!(check_parser(
            "{\n   [1] = 2,  [2] = 4,\n}",
            val!({
                1. => 2.,
                2. => 4.
            })
        ))
    }
}
