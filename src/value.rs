use std::fmt::{self, Write};

use indexmap::IndexMap;

/// Representation of TJS data values.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Void,
    Null,
    Bool(bool),
    Integer(i64),
    Real(f64),
    String(String),
    Octet(Vec<u8>),
    Array(Vec<Value>),
    Dictionary(IndexMap<String, Value>),
}

/// Options used when printing or serializing [`Value`] instances.
#[derive(Debug, Clone)]
pub struct SerializeOptions {
    pub const_hint: bool,
    pub indent: Option<usize>,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self {
            const_hint: true,
            indent: None,
        }
    }
}

impl Value {
    /// Serializes the [`Value`] into a TJS expression (without additional whitespace).
    pub fn to_string_with_options(&self, options: &SerializeOptions) -> String {
        let mut output = String::new();
        // Writing to a `String` cannot fail.
        let _ = self.write_with_options(&mut output, options);
        output
    }

    pub(crate) fn write_with_options<W: Write>(
        &self,
        writer: &mut W,
        options: &SerializeOptions,
    ) -> fmt::Result {
        self.write_internal(writer, options, 0)
    }

    fn write_internal<W: Write>(
        &self,
        writer: &mut W,
        options: &SerializeOptions,
        depth: usize,
    ) -> fmt::Result {
        match self {
            Value::Void => writer.write_str("void"),
            Value::Null => writer.write_str("null"),
            Value::Bool(true) => writer.write_str("true"),
            Value::Bool(false) => writer.write_str("false"),
            Value::Integer(num) => write!(writer, "{}", num),
            Value::Real(num) => {
                if num.is_nan() {
                    writer.write_str("NaN")
                } else if num.is_infinite() {
                    if num.is_sign_negative() {
                        writer.write_str("-Infinity")
                    } else {
                        writer.write_str("Infinity")
                    }
                } else if num.fract() == 0.0 || num.fract() == 1.0 {
                    write!(writer, "{}.0", num)
                } else {
                    write!(writer, "{}", num)
                }
            }
            Value::String(text) => write_string(writer, text),
            Value::Octet(bytes) => write_octet(writer, bytes),
            Value::Array(items) => {
                if options.const_hint {
                    writer.write_str("(const) ")?;
                }
                writer.write_char('[')?;
                if let Some(indent) = options.indent {
                    if !items.is_empty() {
                        writer.write_char('\n')?;
                        for (idx, item) in items.iter().enumerate() {
                            if idx > 0 {
                                writer.write_str(",\n")?;
                            }
                            write_indent(writer, indent, depth + 1)?;
                            item.write_internal(writer, options, depth + 1)?;
                        }
                        writer.write_char('\n')?;
                        write_indent(writer, indent, depth)?;
                    }
                } else {
                    for (idx, item) in items.iter().enumerate() {
                        if idx > 0 {
                            writer.write_str(", ")?;
                        }
                        item.write_internal(writer, options, depth + 1)?;
                    }
                }
                writer.write_char(']')
            }
            Value::Dictionary(entries) => {
                if options.const_hint {
                    writer.write_str("(const) ")?;
                }
                writer.write_str("%[")?;
                if let Some(indent) = options.indent {
                    if !entries.is_empty() {
                        writer.write_char('\n')?;
                        for (idx, (key, value)) in entries.iter().enumerate() {
                            if idx > 0 {
                                writer.write_str(",\n")?;
                            }
                            write_indent(writer, indent, depth + 1)?;
                            write_string(writer, key)?;
                            writer.write_str(" => ")?;
                            value.write_internal(writer, options, depth + 1)?;
                        }
                        writer.write_char('\n')?;
                        write_indent(writer, indent, depth)?;
                    }
                } else {
                    for (idx, (key, value)) in entries.iter().enumerate() {
                        if idx > 0 {
                            writer.write_str(", ")?;
                        }
                        write_string(writer, key)?;
                        writer.write_str(" => ")?;
                        value.write_internal(writer, options, depth + 1)?;
                    }
                }
                writer.write_char(']')
            }
        }
    }
}

fn write_indent<W: Write>(writer: &mut W, indent: usize, depth: usize) -> fmt::Result {
    for _ in 0..(indent * depth) {
        writer.write_char(' ')?;
    }
    Ok(())
}

fn write_string<W: Write>(writer: &mut W, text: &str) -> fmt::Result {
    writer.write_char('"')?;
    for ch in text.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '"' => writer.write_str("\\\"")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\x08' => writer.write_str("\\b")?,
            '\x0c' => writer.write_str("\\f")?,
            ch if ch.is_control() => {
                let code = ch as u32;
                if code <= 0xFF {
                    write!(writer, "\\x{:02x}", code)?;
                } else {
                    write!(writer, "\\u{:04x}", code)?;
                }
            }
            ch => writer.write_char(ch)?,
        }
    }
    writer.write_char('"')
}

fn write_octet<W: Write>(writer: &mut W, bytes: &[u8]) -> fmt::Result {
    writer.write_str("<%")?;
    if !bytes.is_empty() {
        writer.write_char(' ')?;
    }
    for (idx, byte) in bytes.iter().enumerate() {
        if idx > 0 {
            writer.write_char(' ')?;
        }
        write!(writer, "{:02x}", byte)?;
    }
    if !bytes.is_empty() {
        writer.write_char(' ')?;
    }
    writer.write_str("%>")
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Real(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_owned())
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::Array(value)
    }
}

impl From<IndexMap<String, Value>> for Value {
    fn from(value: IndexMap<String, Value>) -> Self {
        Value::Dictionary(value)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_with_options(f, &SerializeOptions::default())
    }
}
