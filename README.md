[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)

# Using const generics for input validation

This project is an experiment to see if using const generics for field validation, specifically text length, is relatively straightforward and could be used for things like data serialization or form validation.

This approach hopes to provide information about what input lengths are valid at the type level.

If you came here looking for something you can use in production, crates like [actix-web-validator](https://crates.io/actix-web-validator/) and [validator](https://crates.io/crates/validator) are better.

## Here is how it works

```rust
use serde::{Serialize, Deserialize}
use length_limited_field::{LengthLimitedField, LengthLimitedFieldError};

// defines a field with a minimum of 10, and maximum of 100 characters
type NameField = LengthLimitedName<1, 10>;

#[derive(Serialize, Deserialize)]
struct MyModel {
    name: NameField
}

impl MyModel {
    fn new(val: &str) -> Result<Self, LengthLimitedFieldError> {
        Ok(Self {
            name: NameField::new(val)?
        })
    }
}

fn main() {
    assert!(MyModel::new("ok").is_ok());
    assert!(MyModel::new("this_name_is_too_long").is_err());
}
```

## Limitations

It's still possible to construct weird or invalid field types, such as:

```rust
type WeirdFieldType = LengthLimitedName<0, 0>;
type WeirdFieldType = LengthLimitedName<100, 10>;
```

The type parameter names aren't named, which isn't very helpful for readability. Maybe a macro would be better:

```rust
macro_rules! LengthLimitedField{
    (min: $min:expr, max: $max:expr)=> {
        LengthLimitedField<$min, $max>
    };
}
```

It would be nice to combine that with some validation of the `min` and `max` values.
