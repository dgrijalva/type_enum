# A derive macro for enums

A common non-generic path for holding a value of one of a set of types is to just use an enum:

```rust
#[derive(TypeEnum)]
enum MyErrors {
    Serialization(serde_json::Error),
    Http(hyper::Error),
    Other(String),
}
```

This library is a helper for these types. It provides a derive macro which:

- automatically implements `From<T>` for each type present in the enum
- automatically implements the helper trait `TypeEnum<T>` for easily unpacking values

## Usage

```rust
use type_enum::TypeEnum;

#[derive(TypeEnum)]
enum Value {
    Text(String),
    Number(i64),
    Pair(u8, u8),
}

// Create values using From conversions
let text: Value = "hello".to_string().into();
let number: Value = 42i64.into();
let tuple: Value = (1u8, 2u8).into();

// Extract values using TypeEnum trait methods
assert_eq!(text.value(), Some(&"hello".to_string()));
assert_eq!(number.value(), Some(&42));
let wrong_type: Option<&String> = number.value();
assert_eq!(wrong_type, None); // wrong type

// Extract by value
let extracted: String = text.into_value().unwrap();
assert_eq!(extracted, "hello");

// Modify values
let mut text_val: Value = "hello".to_string().into();
if let Some(s) = text_val.value_mut() {
    s.push_str(" world");
}
assert_eq!(text_val.value(), Some(&"hello world".to_string()));
```

## Requirements

- Each variant of the enum must hold a unique type
- Tuple values are supported, but not struct style variants
- No unsafe code is used - everything is compile-time safe
- No `'static` bounds required
