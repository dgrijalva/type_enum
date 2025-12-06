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

- automatically implements `From<T>` for each type present in the enum.
- automatically implements the helper trait `TypeEnum` for easily unpacking values

Requirements:

- each variant of the enum must hold a unique type
- tuples values are supported, but not struct style variants

## Usage tips

One really common way I use this pattern is with error types:

```rust
fn do_cool_stuff() -> Result<Pants, MyErrors> {
    // ...
    // this conversion just works
    let data = serde_json::from_str(foo)?;
    // ...
}
```

You can use the helper traits to get some conditional unwrapping:

```rust
fn something() -> Option<Sting> {
    // ...
    let foo : &usize = possible_types.value()?;
    // ...
}

```

## A cool trick for function argument overloading

```rust
#[derive(TypeEnum)]
enum FooInputTypes{
  String(String),
  Num(u16),
}

fn foo(data : impl Into<FooInputTypes> ) {
  match data.into() {
    FooInputTypes::String(val) => println!("You gave me a string: {val}"),
    FooInputTypes::Num(val) => println!("You gave me a number: {val}"),
  }
}
```
