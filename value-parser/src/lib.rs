pub struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
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
        self.pos + tok.len() <= self.src.len() && &self.src[self.pos..self.pos + tok.len()] == tok
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

    pub fn parse_ident(&mut self) -> String {
        let start = self.pos;
        while self.current().is_ascii_alphanumeric() {
            self.advance();
        }
        self.src[start..self.pos].to_owned()
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
            if self.current().is_ascii_alphabetic() {
                is_map = true;
                let k = Value::String(self.parse_ident());
                self.eat_ws();
                assert!(self.eat("="), "expected a = after field");
                let v = self.parse_value();
                map.push((k, v));
            } else if is_map {
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
        assert!(self.eat("\""), "missing closing \"");
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

    pub fn remove_reference(&mut self) {
        while !self.at_eof() && !self.eat(":") {
            self.advance()
        }
    }

    pub fn parse_value(&mut self) -> Value {
        self.eat_ws();
        if self.eat("{") {
            self.parse_list_or_map()
        } else if self.eat("\"") {
            Value::String(self.parse_string())
        } else if self.current().is_ascii_digit() {
            Value::Number(self.parse_number())
        } else if self.eat("true") {
            Value::Bool(true)
        } else if self.eat("false") {
            Value::Bool(false)
        } else if self.eat("@0x") {
            self.remove_reference();
            self.parse_value()
        } else {
            panic!("expected a value");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_value_completely(text: &str) -> Value {
        let mut p = Parser::new(text);
        let val = p.parse_value();
        assert!(p.at_eof(), "parser didn't parse complete input");
        val
    }

    fn check_parser(text: &str, expect_value: Value) {
        let parsed_value = parse_value_completely(text);
        assert_eq!(parsed_value, expect_value);
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

    impl From<bool> for Value {
        fn from(b: bool) -> Self {
            Self::Bool(b)
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
    fn bool() {
        check_parser("true", val!(true));
        check_parser("false", val!(false));
    }

    #[test]
    fn number() {
        check_parser("1", val!(1.));
    }

    #[test]
    fn decimal() {
        check_parser("1.25", val!(1.25));
    }

    #[test]
    #[should_panic]
    fn no_double_together_decimals() {
        parse_value_completely("1..5");
    }

    #[test]
    #[should_panic]
    fn no_double_decimals() {
        parse_value_completely("1.5.2");
    }

    #[test]
    fn simple_string() {
        check_parser(r#""hello""#, val!("hello"))
    }

    #[test]
    fn string_escape() {
        check_parser(r#""\n\t\r\n""#, val!("\n\t\r\n"))
    }

    #[test]
    #[should_panic(expected = "unknown escape")]
    fn string_unending_escape() {
        parse_value_completely(r#""\"#);
    }

    #[test]
    #[should_panic(expected = "missing closing \"")]
    fn string_unclosed() {
        parse_value_completely("\"hello");
    }

    #[test]
    fn empty_list() {
        check_parser(r#"{}"#, val!([]))
    }

    #[test]
    fn list_of_numbers() {
        check_parser(
            r#"{1  , 2, 5,4,  3,2,3}"#,
            val!([1., 2., 5., 4., 3., 2., 3.]),
        )
    }
    #[test]
    fn list_single_string() {
        check_parser(r#"{"abc"}"#, val!(["abc"]))
    }

    #[test]
    fn list_of_string() {
        check_parser(
            r#"{"abc"   , "cd", "e", "f"}"#,
            val!(["abc", "cd", "e", "f"]),
        )
    }
    #[test]
    fn list_of_lists() {
        check_parser(r#"{{}, {}, {}}"#, val!([[], [], []]))
    }

    #[test]
    fn list_hetero() {
        check_parser(
            r#"{{        }, 1       ,     "xyz",       {  1, "bb"} , 2.5 }"#,
            val!([[], 1., "xyz", [1., "bb"], 2.5]),
        )
    }

    #[test]
    fn list_with_trailing_comma() {
        check_parser(r#"{5,}"#, val!([5.]))
    }

    #[test]
    #[should_panic(expected = ", not allowed before first item")]
    fn list_with_leading_comma_and_element() {
        parse_value_completely(r#"{,5}"#);
    }

    #[test]
    #[should_panic(expected = ", not allowed before first item")]
    fn list_first_comma_not_allowed() {
        parse_value_completely(r#"{,}"#);
    }

    #[test]
    #[should_panic(expected = ", not allowed before first item")]
    fn map_first_comma_not_allowed() {
        parse_value_completely(r#"{,[5] => 2}"#);
    }

    #[test]
    fn map_simple() {
        check_parser(
            "{\n   [1] = 2,  [2] = 4,\n}",
            val!({
                1. => 2.,
                2. => 4.
            }),
        )
    }

    #[test]
    fn map_single_key() {
        check_parser(
            "{[1] = 2}",
            val!({
                1. => 2.
            }),
        )
    }

    #[test]
    fn map_string() {
        check_parser(
            r#"{["1"] = "8",  ["5"] = "2"}"#,
            val!({
                "1" => "8",
                "5" => "2"
            }),
        )
    }

    #[test]
    #[should_panic(expected = "expected a value")]
    fn map_no_value() {
        parse_value_completely("{[1] =}");
    }

    #[test]
    #[should_panic(expected = "expected a ]")]
    fn map_unbalance_bracket() {
        parse_value_completely("{[1 =}");
    }

    #[test]
    #[should_panic(expected = "expected a =")]
    fn map_missing_eq() {
        parse_value_completely("{[1] 1}");
    }

    #[test]
    fn empty_curlies_is_list() {
        check_parser(r#"{}"#, val!([]))
    }

    #[test]
    #[should_panic(expected = "can't mix list and map")]
    fn mix_list_and_map() {
        parse_value_completely("{[5] = 2, 5}");
    }

    #[test]
    fn map_string_vec() {
        check_parser(
            r#"{["1"] = {1, 2},  ["5"] = {5, 6}}"#,
            val!({
                "1" => [1., 2.],
                "5" => [5., 6.]
            }),
        )
    }

    #[test]
    fn map_nested() {
        check_parser(
            r#"{["1"] = {[1] = 2},  ["5"] = {[3] = 4}}"#,
            val!({
                "1" => { 1. => 2. },
                "5" => { 3. => 4. }
            }),
        )
    }

    #[test]
    fn map_with_list_keys() {
        check_parser(
            r#"{[{1, 2}] = 1,  [{3, 4}] = {[3] = 4}}"#,
            val!({
                [1., 2.] => 1.,
                [3., 4.] => { 3. => 4. }
            }),
        )
    }

    #[test]
    fn list_of_map() {
        check_parser(
            r#"{{[1] = 2}, {[3] = 4, [5] = 6}}"#,
            val!([
                {1. => 2.},
                {3. => 4., 5. => 6.}
            ]),
        )
    }

    #[test]
    fn structure() {
        check_parser(r#"{ x = 5 }"#, val!({"x" => 5.}))
    }

    #[test]
    fn structure_field_numbers() {
        check_parser(r#"{ x5xe = 5 }"#, val!({"x5xe" => 5.}))
    }

    #[test]
    fn mix_struct_and_map() {
        check_parser(r#"{ x5xe = 5, [3] = 2 }"#, val!({"x5xe" => 5., 3. => 2. }))
    }

    #[test]
    #[should_panic(expected = "can't mix list and map")]
    fn mix_struct_and_list() {
        parse_value_completely("{x = 2, 5}");
    }

    #[test]
    fn reference_number() {
        check_parser(r#"@0x7fffffffde44: 1"#, val!(1.))
    }

    #[test]
    fn reference_remove() {
        let mut p = Parser::new("@0x83fd: foobar_random_stuff");
        p.remove_reference();
        assert!(p.pos == 8);
        assert!(p.current() == ' ');
    }
}
