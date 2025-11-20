use std::io::Write as IoWrite;

use indexmap::IndexMap;
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTupleVariant,
};

use crate::error::{Error, Result};
use crate::value::{SerializeOptions, Value};

/// Convert a `T` into `serde_tjs::Value` which is an enum that can represent any valid TJS2 data.
pub fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    value.serialize(ValueSerializer)
}

/// Serialize the given data structure as a String of TJS2 text.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    to_string_with_options(value, &SerializeOptions::default())
}

/// Serialize the given data structure as a pretty-printed String of TJS2 text.
pub fn to_string_pretty<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let options = pretty_options();
    to_string_with_options(value, &options)
}

/// Serialize the given data structure as a String of TJS2 text with custom options.
pub fn to_string_with_options<T>(value: &T, options: &SerializeOptions) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let value = value.serialize(ValueSerializer)?;
    Ok(value.to_string_with_options(options))
}

/// Serialize the given data structure as a `Vec<u8>` of TJS2 text.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    to_vec_with_options(value, &SerializeOptions::default())
}

/// Serialize the given data structure as a pretty-printed `Vec<u8>` of TJS2 text.
pub fn to_vec_pretty<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let options = pretty_options();
    to_vec_with_options(value, &options)
}

/// Serialize the given data structure as a `Vec<u8>` of TJS2 text with custom options.
pub fn to_vec_with_options<T>(value: &T, options: &SerializeOptions) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut buffer = Vec::new();
    to_writer_with_options(&mut buffer, value, options)?;
    Ok(buffer)
}

/// Serialize the given data structure by writing TJS2 text into the provided writer.
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: IoWrite,
    T: ?Sized + Serialize,
{
    to_writer_with_options(writer, value, &SerializeOptions::default())
}

/// Serialize the given data structure as a pretty-printed TJS2 text into the provided writer.
pub fn to_writer_pretty<W, T>(writer: W, value: &T) -> Result<()>
where
    W: IoWrite,
    T: ?Sized + Serialize,
{
    let options = pretty_options();
    to_writer_with_options(writer, value, &options)
}

/// Serialize the given data structure with custom options by writing TJS2 text into the provided writer.
pub fn to_writer_with_options<W, T>(
    mut writer: W,
    value: &T,
    options: &SerializeOptions,
) -> Result<()>
where
    W: IoWrite,
    T: ?Sized + Serialize,
{
    let value = value.serialize(ValueSerializer)?;
    let output = value.to_string_with_options(options);
    writer
        .write_all(output.as_bytes())
        .map_err(|err| Error::new(err.to_string()))
}

fn pretty_options() -> SerializeOptions {
    SerializeOptions {
        indent: Some(2),
        ..SerializeOptions::default()
    }
}

pub struct ValueSerializer;

impl serde::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i16(self, v: i16) -> Result<Value> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i32(self, v: i32) -> Result<Value> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(Value::Integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Value> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u16(self, v: u16) -> Result<Value> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u32(self, v: u32) -> Result<Value> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u64(self, v: u64) -> Result<Value> {
        if v <= i64::MAX as u64 {
            Ok(Value::Integer(v as i64))
        } else {
            Ok(Value::Real(v as f64))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Value> {
        Ok(Value::Real(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Value> {
        Ok(Value::Real(v))
    }

    fn serialize_char(self, v: char) -> Result<Value> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        Ok(Value::Octet(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Value> {
        Ok(Value::Void)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Void)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        Ok(Value::Void)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        Ok(Value::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        let mut map = IndexMap::new();
        map.insert(variant.to_owned(), value.serialize(ValueSerializer)?);
        Ok(Value::Dictionary(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer> {
        Ok(SeqSerializer {
            elements: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SeqSerializer> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TupleVariantSerializer> {
        Ok(TupleVariantSerializer {
            name: variant.to_owned(),
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<MapSerializer> {
        Ok(MapSerializer {
            entries: IndexMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<MapSerializer> {
        self.serialize_map(None)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<StructVariantSerializer> {
        Ok(StructVariantSerializer {
            name: variant.to_owned(),
            map: IndexMap::new(),
        })
    }

    fn collect_str<T: ?Sized + std::fmt::Display>(self, value: &T) -> Result<Value> {
        Ok(Value::String(value.to_string()))
    }
}

pub struct SeqSerializer {
    elements: Vec<Value>,
}

impl SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.elements))
    }
}

impl serde::ser::SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        SerializeSeq::end(self)
    }
}

pub struct TupleVariantSerializer {
    name: String,
    fields: Vec<Value>,
}

impl SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.fields.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut map = IndexMap::new();
        map.insert(self.name, Value::Array(self.fields));
        Ok(Value::Dictionary(map))
    }
}

pub struct MapSerializer {
    entries: IndexMap<String, Value>,
    next_key: Option<String>,
}

impl SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(MapKeySerializer)?;
        self.next_key = Some(key);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| Error::new("value serialized before key"))?;
        let value = value.serialize(ValueSerializer)?;
        self.entries.insert(key, value);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Dictionary(self.entries))
    }
}

impl SerializeStruct for MapSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(ValueSerializer)?;
        self.entries.insert(key.to_owned(), value);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Dictionary(self.entries))
    }
}

pub struct StructVariantSerializer {
    name: String,
    map: IndexMap<String, Value>,
}

impl SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(ValueSerializer)?;
        self.map.insert(key.to_owned(), value);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut map = IndexMap::new();
        map.insert(self.name, Value::Dictionary(self.map));
        Ok(Value::Dictionary(map))
    }
}

struct MapKeySerializer;

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = ser::Impossible<String, Error>;
    type SerializeTuple = ser::Impossible<String, Error>;
    type SerializeTupleStruct = ser::Impossible<String, Error>;
    type SerializeTupleVariant = ser::Impossible<String, Error>;
    type SerializeMap = ser::Impossible<String, Error>;
    type SerializeStruct = ser::Impossible<String, Error>;
    type SerializeStructVariant = ser::Impossible<String, Error>;

    fn serialize_bool(self, v: bool) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_i8(self, v: i8) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_i64(self, v: i64) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<String> {
        Ok(v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<String> {
        Ok(v.to_owned())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<String> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_none(self) -> Result<String> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_unit(self) -> Result<String> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<String> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::new("dictionary keys must be strings"))
    }

    fn collect_str<T: ?Sized + std::fmt::Display>(self, value: &T) -> Result<String> {
        Ok(value.to_string())
    }
}
