# Derive Display

A derive for the `Display` trait.

```rust
#[macro_use] extern crate display_derive;

#[derive(Display)]
#[display(fmt = "Error code: {}", code)]
struct RecordError {
    code: u32,
}

#[derive(Display)]
enum EnumError {
    #[display(fmt = "Error code: {}", code)]
    StructVariant {
        code: i32,
    },
    #[display(fmt = "Error: {}", _0)]
    TupleVariant(&'static str),
    #[display(fmt = "An error has occurred.")]
    UnitVariant,
}
```
