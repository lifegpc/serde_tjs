use std::str;

use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

use crate::error::{Error, Result};
use crate::parser;
use crate::value::Value;

/// Parse a `serde_tjs::Value` from tjs2 text.
pub fn parse_value(input: &str) -> Result<Value> {
    parser::parse_str(input)
}

/// Deserialize an instance of type `T` from a string of tjs2 text.
pub fn from_str<T>(input: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let value = parse_value(input)?;
    from_value(value)
}

/// Deserialize an instance of type `T`` from bytes of tjs2 text.
pub fn from_slice<T>(input: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let text = str::from_utf8(input)
        .map_err(|err| Error::new(format!("input is not valid UTF-8: {err}")))?;
    from_str(text)
}

/// Interpret a `serde_tjs::Value` as an instance of type `T`.
pub fn from_value<T>(value: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    T::deserialize(ValueDeserializer::new(value))
}

pub struct ValueDeserializer {
    value: Value,
}

impl ValueDeserializer {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = ValueDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer::new(self)
    }
}

impl<'de> de::Deserializer<'de> for ValueDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Void | Value::Null => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Integer(v) => visitor.visit_i64(v),
            Value::Real(v) => visitor.visit_f64(v),
            Value::String(v) => visitor.visit_string(v),
            Value::Octet(v) => visitor.visit_byte_buf(v),
            Value::Array(values) => {
                let seq = SeqDeserializer {
                    iter: values.into_iter(),
                };
                visitor.visit_seq(seq)
            }
            Value::Dictionary(map) => {
                let map = MapDeserializer {
                    iter: map.into_iter(),
                    value: None,
                };
                visitor.visit_map(map)
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Bool(v) => visitor.visit_bool(v),
            other => Err(Error::new(format!("expected bool, found {other:?}"))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Integer(v) => visitor.visit_i64(v),
            other => Err(Error::new(format!("expected integer, found {other:?}"))),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Integer(v) => visitor.visit_i128(v as i128),
            other => Err(Error::new(format!("expected integer, found {other:?}"))),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Integer(v) if v >= 0 => visitor.visit_u64(v as u64),
            other => Err(Error::new(format!(
                "expected unsigned integer, found {other:?}"
            ))),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Integer(v) if v >= 0 => visitor.visit_u128(v as u128),
            other => Err(Error::new(format!(
                "expected unsigned integer, found {other:?}"
            ))),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Real(v) => visitor.visit_f64(v),
            Value::Integer(v) => visitor.visit_f64(v as f64),
            other => Err(Error::new(format!("expected float, found {other:?}"))),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::String(s) => {
                let mut chars = s.chars();
                if let Some(ch) = chars.next() {
                    if chars.next().is_none() {
                        visitor.visit_char(ch)
                    } else {
                        Err(Error::new("expected single character"))
                    }
                } else {
                    Err(Error::new("expected single character"))
                }
            }
            other => Err(Error::new(format!("expected char, found {other:?}"))),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::String(v) => visitor.visit_string(v),
            other => Err(Error::new(format!("expected string, found {other:?}"))),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Octet(v) => visitor.visit_byte_buf(v),
            Value::String(s) => visitor.visit_byte_buf(s.into_bytes()),
            other => Err(Error::new(format!("expected byte buffer, found {other:?}"))),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Void | Value::Null => visitor.visit_none(),
            other => visitor.visit_some(ValueDeserializer::new(other)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Void | Value::Null => visitor.visit_unit(),
            other => Err(Error::new(format!("expected unit, found {other:?}"))),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(ValueDeserializer::new(self.value))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Array(values) => {
                let seq = SeqDeserializer {
                    iter: values.into_iter(),
                };
                visitor.visit_seq(seq)
            }
            other => Err(Error::new(format!("expected array, found {other:?}"))),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Dictionary(map) => {
                let access = MapDeserializer {
                    iter: map.into_iter(),
                    value: None,
                };
                visitor.visit_map(access)
            }
            other => Err(Error::new(format!("expected dictionary, found {other:?}"))),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::String(name) => visitor.visit_enum(EnumDeserializer {
                variant: name,
                value: None,
            }),
            Value::Dictionary(map) => {
                if map.len() != 1 {
                    return Err(Error::new(
                        "enum representation must contain exactly one entry",
                    ));
                }
                let (name, value) = map.into_iter().next().unwrap();
                visitor.visit_enum(EnumDeserializer {
                    variant: name,
                    value: Some(value),
                })
            }
            other => Err(Error::new(format!("expected enum, found {other:?}"))),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(ValueDeserializer::new(value)).map(Some),
            None => Ok(None),
        }
    }
}

struct MapDeserializer {
    iter: indexmap::map::IntoIter<String, Value>,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_deserializer = ValueDeserializer::new(Value::String(key));
                seed.deserialize(key_deserializer).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or_else(|| Error::new("value missing for key"))?;
        seed.deserialize(ValueDeserializer::new(value))
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(ValueDeserializer::new(Value::String(self.variant)))?;
        Ok((variant, VariantDeserializer { value: self.value }))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            None => Ok(()),
            Some(_) => Err(Error::new("expected unit variant")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(ValueDeserializer::new(value)),
            None => Err(Error::new("expected value for newtype variant")),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Array(values)) => {
                let seq = SeqDeserializer {
                    iter: values.into_iter(),
                };
                visitor.visit_seq(seq)
            }
            _ => Err(Error::new("tuple variant expected an array")),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Dictionary(map)) => {
                let access = MapDeserializer {
                    iter: map.into_iter(),
                    value: None,
                };
                visitor.visit_map(access)
            }
            _ => Err(Error::new("struct variant expected a dictionary")),
        }
    }
}
