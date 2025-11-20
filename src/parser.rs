use indexmap::IndexMap;

use crate::error::{Error, Result};
use crate::value::Value;

pub fn parse_str(input: &str) -> Result<Value> {
    let mut parser = Parser::new(input);
    parser.skip_ws()?;
    let value = parser.parse_value()?;
    parser.skip_ws()?;
    if parser.is_eof() {
        Ok(value)
    } else {
        Err(Error::with_position(
            "unexpected trailing characters",
            parser.position,
        ))
    }
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            position: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.bytes.len()
    }

    fn skip_ws(&mut self) -> Result<()> {
        loop {
            let Some(ch) = self.peek_char() else {
                return Ok(());
            };
            if ch.is_whitespace() {
                self.advance_char(ch);
                continue;
            }
            if ch == '/' {
                if self.consume_exact("//") {
                    while let Some(c) = self.next_char() {
                        if c == '\n' {
                            break;
                        }
                    }
                    continue;
                } else if self.consume_exact("/*") {
                    let rest = &self.input[self.position..];
                    if let Some(idx) = rest.find("*/") {
                        self.position += idx + 2;
                    } else {
                        return Err(Error::with_position(
                            "unterminated block comment",
                            self.position,
                        ));
                    }
                    continue;
                }
            }
            return Ok(());
        }
    }

    fn parse_value(&mut self) -> Result<Value> {
        self.skip_ws()?;
        self.consume_const_hint();
        self.skip_ws()?;
        match self.peek_byte() {
            Some(b'[') => self.parse_array(),
            Some(b'%') => self.parse_dictionary(),
            Some(b'"') | Some(b'\'') => self.parse_string().map(Value::String),
            Some(b'<') if self.starts_with("<%") => self.parse_octet(),
            Some(b't') | Some(b'f') | Some(b'n') | Some(b'v') | Some(b'I') | Some(b'N') => {
                self.parse_literal()
            }
            Some(b'+') | Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(_) => Err(Error::with_position("unexpected token", self.position)),
            None => Err(Error::with_position(
                "unexpected end of input",
                self.position,
            )),
        }
    }

    fn parse_array(&mut self) -> Result<Value> {
        self.expect_char('[')?;
        let mut items = Vec::new();
        loop {
            self.skip_ws()?;
            if self.consume_ascii(']') {
                break;
            }
            let value = self.parse_value()?;
            items.push(value);
            self.skip_ws()?;
            if self.consume_ascii(',') {
                continue;
            } else if self.consume_ascii(']') {
                break;
            } else {
                return Err(Error::with_position("expected ',' or ']'", self.position));
            }
        }
        Ok(Value::Array(items))
    }

    fn parse_dictionary(&mut self) -> Result<Value> {
        self.expect_char('%')?;
        self.skip_ws()?;
        self.expect_char('[')?;
        let mut entries = IndexMap::new();
        loop {
            self.skip_ws()?;
            if self.consume_ascii(']') {
                break;
            }
            let key = self.parse_dict_key()?;
            self.skip_ws()?;
            if self.consume_exact("=>") {
                // ok
            } else if self.consume_ascii(':') {
                // legacy form
            } else {
                return Err(Error::with_position(
                    "expected '=>' after key",
                    self.position,
                ));
            }
            let value = self.parse_value()?;
            entries.insert(key, value);
            self.skip_ws()?;
            if self.consume_ascii(',') {
                continue;
            } else if self.consume_ascii(']') {
                break;
            } else {
                return Err(Error::with_position("expected ',' or ']'", self.position));
            }
        }
        Ok(Value::Dictionary(entries))
    }

    fn parse_dict_key(&mut self) -> Result<String> {
        match self.peek_byte() {
            Some(b'"') | Some(b'\'') => self.parse_string(),
            _ => self
                .parse_identifier()
                .ok_or_else(|| Error::with_position("expected dictionary key", self.position)),
        }
    }

    fn parse_literal(&mut self) -> Result<Value> {
        if self.consume_exact("true") {
            Ok(Value::Bool(true))
        } else if self.consume_exact("false") {
            Ok(Value::Bool(false))
        } else if self.consume_exact("null") {
            Ok(Value::Null)
        } else if self.consume_exact("void") {
            Ok(Value::Void)
        } else if self.consume_exact("NaN") {
            Ok(Value::Real(f64::NAN))
        } else if self.consume_exact("Infinity") {
            Ok(Value::Real(f64::INFINITY))
        } else {
            Err(Error::with_position("unknown literal", self.position))
        }
    }

    fn parse_number(&mut self) -> Result<Value> {
        if self.starts_with("-Infinity") {
            self.position += "-Infinity".len();
            return Ok(Value::Real(f64::NEG_INFINITY));
        }
        if self.starts_with("+Infinity") {
            self.position += "+Infinity".len();
            return Ok(Value::Real(f64::INFINITY));
        }
        if self.starts_with("+NaN") || self.starts_with("-NaN") {
            self.position += 4;
            return Ok(Value::Real(f64::NAN));
        }

        let start = self.position;
        let negative = if self.consume_ascii('-') {
            true
        } else {
            self.consume_ascii('+');
            false
        };

        if self.starts_with("0x") || self.starts_with("0X") {
            self.position += 2;
            let digits_start = self.position;
            self.consume_digits(|b| b.is_ascii_hexdigit());
            if self.position == digits_start {
                return Err(Error::with_position("expected hex digits", self.position));
            }
            let digits = &self.input[digits_start..self.position];
            let unsigned = i128::from_str_radix(digits, 16)
                .map_err(|_| Error::with_position("invalid hex number", digits_start))?;
            let signed = if negative { -unsigned } else { unsigned };
            if signed < i128::from(i64::MIN) || signed > i128::from(i64::MAX) {
                return Err(Error::with_position("integer overflow", start));
            }
            return Ok(Value::Integer(signed as i64));
        }

        let mut seen_digit = false;
        while let Some(byte) = self.peek_byte() {
            if byte.is_ascii_digit() {
                seen_digit = true;
                self.position += 1;
            } else {
                break;
            }
        }

        let mut is_float = false;
        if self.consume_ascii('.') {
            is_float = true;
            while let Some(byte) = self.peek_byte() {
                if byte.is_ascii_digit() {
                    seen_digit = true;
                    self.position += 1;
                } else {
                    break;
                }
            }
        }

        if matches!(self.peek_byte(), Some(b'e' | b'E')) {
            is_float = true;
            self.position += 1;
            if matches!(self.peek_byte(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            let mut exp_digits = 0;
            while let Some(byte) = self.peek_byte() {
                if byte.is_ascii_digit() {
                    exp_digits += 1;
                    self.position += 1;
                } else {
                    break;
                }
            }
            if exp_digits == 0 {
                return Err(Error::with_position(
                    "expected exponent digits",
                    self.position,
                ));
            }
        }

        if !seen_digit {
            return Err(Error::with_position("expected number", start));
        }

        let slice = &self.input[start..self.position];
        if is_float {
            let value = slice
                .parse::<f64>()
                .map_err(|_| Error::with_position("invalid number", start))?;
            Ok(Value::Real(value))
        } else {
            match slice.parse::<i64>() {
                Ok(num) => Ok(Value::Integer(num)),
                Err(_) => slice
                    .parse::<f64>()
                    .map(Value::Real)
                    .map_err(|_| Error::with_position("invalid number", start)),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        let quote = self
            .next_byte()
            .ok_or_else(|| Error::with_position("unexpected end of input", self.position))?
            as char;
        let mut output = String::new();
        loop {
            let ch = self
                .next_char()
                .ok_or_else(|| Error::with_position("unterminated string", self.position))?;
            if ch == quote {
                break;
            }
            if ch == '\\' {
                output.push(self.parse_escape()?);
            } else {
                output.push(ch);
            }
        }
        Ok(output)
    }

    fn parse_escape(&mut self) -> Result<char> {
        let ch = self
            .next_char()
            .ok_or_else(|| Error::with_position("unterminated escape", self.position))?;
        Ok(match ch {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'b' => '\x08',
            'f' => '\x0c',
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            '0' => '\0',
            'x' => {
                let value = self.read_hex_digits(2)?;
                char::from_u32(value as u32)
                    .ok_or_else(|| Error::with_position("invalid hex escape", self.position))?
            }
            'u' => {
                let value = self.read_hex_digits(4)?;
                char::from_u32(value as u32)
                    .ok_or_else(|| Error::with_position("invalid unicode escape", self.position))?
            }
            other => other,
        })
    }

    fn read_hex_digits(&mut self, count: usize) -> Result<u32> {
        let mut value = 0u32;
        for _ in 0..count {
            let digit = self
                .peek_byte()
                .ok_or_else(|| Error::with_position("unexpected end of input", self.position))?;
            let parsed = hex_value(digit)
                .ok_or_else(|| Error::with_position("invalid hex digit", self.position))?;
            self.position += 1;
            value = (value << 4) | parsed as u32;
        }
        Ok(value)
    }

    fn parse_octet(&mut self) -> Result<Value> {
        self.expect_str("<%")?;
        let mut bytes = Vec::new();
        loop {
            self.skip_inline_ws();
            if self.starts_with("%>") {
                self.position += 2;
                break;
            }
            let high = self.read_octet_digit()?;
            let low = self.read_octet_digit()?;
            bytes.push((high << 4) | low);
        }
        Ok(Value::Octet(bytes))
    }

    fn read_octet_digit(&mut self) -> Result<u8> {
        let digit = self
            .peek_byte()
            .ok_or_else(|| Error::with_position("unexpected end of input", self.position))?;
        if let Some(value) = hex_value(digit) {
            self.position += 1;
            Ok(value)
        } else {
            Err(Error::with_position("invalid octet digit", self.position))
        }
    }

    fn parse_identifier(&mut self) -> Option<String> {
        let start = self.position;
        let Some(first) = self.peek_byte() else {
            return None;
        };
        if !is_ident_start(first) {
            return None;
        }
        self.position += 1;
        self.consume_digits(is_ident_continue);
        Some(self.input[start..self.position].to_string())
    }

    fn consume_digits<F: Fn(u8) -> bool>(&mut self, predicate: F) {
        while let Some(byte) = self.peek_byte() {
            if predicate(byte) {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn consume_const_hint(&mut self) {
        loop {
            self.skip_inline_ws();
            if self.consume_exact("(const)") {
                continue;
            }
            if self.starts_with("const") {
                let next_pos = self.position + "const".len();
                let is_hint = match self.input[next_pos..].chars().next() {
                    None => true,
                    Some(ch) => ch.is_whitespace() || ch == '[' || ch == '%',
                };
                if is_hint {
                    self.position = next_pos;
                    continue;
                }
            }
            break;
        }
    }

    fn skip_inline_ws(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.advance_char(ch);
            } else {
                break;
            }
        }
    }

    fn starts_with(&self, token: &str) -> bool {
        self.input[self.position..].starts_with(token)
    }

    fn consume_exact(&mut self, token: &str) -> bool {
        if self.starts_with(token) {
            self.position += token.len();
            true
        } else {
            false
        }
    }

    fn expect_char(&mut self, ch: char) -> Result<()> {
        match self.peek_byte() {
            Some(byte) if byte == ch as u8 => {
                self.position += 1;
                Ok(())
            }
            _ => Err(Error::with_position(
                format!("expected '{ch}'"),
                self.position,
            )),
        }
    }

    fn expect_str(&mut self, token: &str) -> Result<()> {
        if self.consume_exact(token) {
            Ok(())
        } else {
            Err(Error::with_position(
                format!("expected '{token}'"),
                self.position,
            ))
        }
    }

    fn consume_ascii(&mut self, ch: char) -> bool {
        if self.peek_byte() == Some(ch as u8) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance_char(&mut self, ch: char) {
        self.position += ch.len_utf8();
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.position += 1;
        Some(byte)
    }
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn is_ident_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

fn is_ident_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}
