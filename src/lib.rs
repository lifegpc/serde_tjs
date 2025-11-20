//! # Serde TJS
//! A Serde serialization/deserialization library for TJS2 data.
//! ```
//! use serde::{Deserialize, Serialize};
//! use serde_tjs::Result;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Person {
//!     name: String,
//!     age: u8,
//!     phones: Vec<String>,
//! }
//!
//! fn typed_example() -> Result<()> {
//!     // Some TJS2 input data as a &str. Maybe this comes from the user.
//!     let data = r#"
//!         (const) %[
//!             "name" => "John Doe",
//!             "age" => 43,
//!             "phones" => (const) [
//!                 "+44 1234567",
//!                 "+44 2345678"
//!             ]
//!         ]"#;
//!
//!     // Parse the string of data into a Person object. This is exactly the
//!     // same function as the one that produced serde_tjs::Value above, but
//!     // now we are asking it for a Person as output.
//!     let p: Person = serde_tjs::from_str(data)?;
//!
//!     // Do things just like with any other Rust data structure.
//!     println!("Please call {} at the number {}", p.name, p.phones[0]);
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     typed_example().unwrap();
//! # }
//! ```
mod de;
mod error;
mod parser;
mod ser;
mod value;

pub use crate::de::{from_slice, from_str, from_value, parse_value};
pub use crate::error::{Error, Result};
pub use crate::ser::{
    to_string, to_string_pretty, to_string_with_options, to_value, to_vec, to_vec_pretty,
    to_vec_with_options, to_writer, to_writer_pretty, to_writer_with_options,
};
pub use crate::value::{SerializeOptions, Value};

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{SerializeOptions, Value, from_str, parse_value};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct SampleStruct {
        name: String,
        score: i32,
        flags: Vec<bool>,
    }

    #[test]
    fn dump_float() {
        let value = 1f64;
        let s = crate::to_string(&value).expect("serialize");
        assert_eq!(s, "1.0");
        let value = 300f64;
        let s = crate::to_string(&value).expect("serialize");
        assert_eq!(s, "300.0");
        let value = 3e10f64;
        let s = crate::to_string(&value).expect("serialize");
        assert_eq!(s, "30000000000.0");
        let value = 3e20f64;
        let s = crate::to_string(&value).expect("serialize");
        assert_eq!(s, "300000000000000000000.0");
        let value = f64::INFINITY;
        let s = crate::to_string(&value).expect("serialize");
        assert_eq!(s, "Infinity");
        let value = f64::NEG_INFINITY;
        let s = crate::to_string(&value).expect("serialize");
        assert_eq!(s, "-Infinity");
        let value = f64::NAN;
        let s = crate::to_string(&value).expect("serialize");
        assert_eq!(s, "NaN");
        let value = -10f64;
        let s = crate::to_string(&value).expect("serialize");
        assert_eq!(s, "-10.0");
    }

    #[test]
    fn parse_save_struct_sample() {
        let input = r#"(const) [
            1,
            2,
            (const) [4, 5],
            (const) %[
                "a" => 1,
                "b" => 2
            ],
            "文字列"
        ]"#;

        let value = parse_value(input).expect("failed to parse");
        match value {
            Value::Array(items) => {
                assert_eq!(items.len(), 5);
                assert_eq!(items[0], Value::Integer(1));
                let mut expected = indexmap::IndexMap::new();
                expected.insert("a".to_string(), Value::Integer(1));
                expected.insert("b".to_string(), Value::Integer(2));
                assert_eq!(items[3], Value::Dictionary(expected));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn serde_roundtrip() {
        let data = SampleStruct {
            name: "kirikiri".to_string(),
            score: 42,
            flags: vec![true, false],
        };

        let serialized = crate::to_string(&data).expect("serialize");
        let restored: SampleStruct = from_str(&serialized).expect("deserialize");
        assert_eq!(data, restored);
    }

    #[test]
    fn const_hint_toggle() {
        let value = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
        let mut options = SerializeOptions {
            const_hint: true,
            indent: None,
        };
        let with_const = value.to_string_with_options(&options);
        assert!(with_const.starts_with("(const)"));

        options.const_hint = false;
        let without_const = value.to_string_with_options(&options);
        assert!(without_const.starts_with("["));
    }

    #[test]
    fn pretty_serialization_inserts_indentation() {
        let data = vec![1, 2, 3];
        let pretty = crate::to_string_pretty(&data).expect("pretty serialize");
        assert!(pretty.contains("[\n  1,"));
        assert!(pretty.ends_with("\n]"));
    }

    #[test]
    fn vec_and_writer_helpers_match() {
        let data = SampleStruct {
            name: "helpers".to_string(),
            score: 7,
            flags: vec![false, true],
        };

        let text = crate::to_string(&data).expect("serialize");
        let bytes = crate::to_vec(&data).expect("to_vec");
        assert_eq!(text.as_bytes(), bytes.as_slice());

        let pretty_text = crate::to_string_pretty(&data).expect("pretty serialize");
        let pretty_bytes = crate::to_vec_pretty(&data).expect("to_vec_pretty");
        assert_eq!(pretty_text.as_bytes(), pretty_bytes.as_slice());

        let mut buffer = Vec::new();
        crate::to_writer(&mut buffer, &data).expect("to_writer");
        assert_eq!(buffer, bytes);
    }
}
