# Serde TJS
A Serde serialization/deserialization library for TJS2 data.
```rust
use serde::{Deserialize, Serialize};
use serde_tjs::Result;

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}

fn typed_example() -> Result<()> {
    // Some TJS2 input data as a &str. Maybe this comes from the user.
    let data = r#"
        (const) %[
            "name" => "John Doe",
            "age" => 43,
            "phones" => (const) [
                "+44 1234567",
                "+44 2345678"
            ]
        ]"#;

    // Parse the string of data into a Person object. This is exactly the
    // same function as the one that produced serde_tjs::Value above, but
    // now we are asking it for a Person as output.
    let p: Person = serde_tjs::from_str(data)?;

    // Do things just like with any other Rust data structure.
    println!("Please call {} at the number {}", p.name, p.phones[0]);

    Ok(())
}
```
