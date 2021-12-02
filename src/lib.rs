use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::{convert::TryFrom, fmt, marker::PhantomData, ops::Deref};

#[derive(Debug)]
pub struct LengthLimitedField<const MIN: usize, const MAX: usize> {
    pub(crate) inner: String,
}

impl<const MIN: usize, const MAX: usize> LengthLimitedField<MIN, MAX> {
    pub fn new(value: &str) -> Result<Self, LengthLimitedFieldError> {
        Self::try_from(value)
    }
}

impl<const MIN: usize, const MAX: usize> Deref for LengthLimitedField<MIN, MAX> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LengthLimitedFieldError {
    #[error("Length of value {len:?} longer than {max:?}")]
    TooLong { len: usize, max: usize },
    #[error("Length of value {len:?} shorter than {min:?}")]
    TooShort { len: usize, min: usize },
}

impl<const MIN: usize, const MAX: usize> TryFrom<&str> for LengthLimitedField<MIN, MAX> {
    type Error = LengthLimitedFieldError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let max = MAX;
        let min = MIN;
        match value.len() {
            len if len > MAX => Err(LengthLimitedFieldError::TooLong { len, max }),
            len if len < MIN => Err(LengthLimitedFieldError::TooShort { len, min }),
            _ => Ok(LengthLimitedField {
                inner: value.to_string(),
            }),
        }
    }
}

impl<const MIN: usize, const MAX: usize> Serialize for LengthLimitedField<MIN, MAX> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

struct LengthLimitedFieldVisitor<const MIN: usize, const MAX: usize> {
    marker: PhantomData<fn() -> LengthLimitedField<MIN, MAX>>,
}

impl<const MIN: usize, const MAX: usize> LengthLimitedFieldVisitor<MIN, MAX> {
    fn new() -> Self {
        LengthLimitedFieldVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, const MIN: usize, const MAX: usize> Visitor<'de> for LengthLimitedFieldVisitor<MIN, MAX> {
    // The type that our Visitor is going to produce.
    type Value = LengthLimitedField<MIN, MAX>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "a string with length less than {} and greater than {}",
            MIN, MAX
        ))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        LengthLimitedField::try_from(v)
            .map_err(|error| serde::de::Error::custom(format!("{}", error)))
    }
}

impl<'de, const MIN: usize, const MAX: usize> Deserialize<'de> for LengthLimitedField<MIN, MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let field = deserializer.deserialize_string(LengthLimitedFieldVisitor::new())?;
        Ok(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type NameField = LengthLimitedField<10, 100>;

    #[derive(Debug, Serialize, Deserialize)]
    struct MyModel {
        name: NameField,
    }

    #[test]
    fn it_serializes() {
        let json = "{\"name\": \"morethantencharacters\"}";
        let serialized: MyModel = serde_json::from_str(&json).expect("should serialize");
        assert_eq!(*serialized.name, "morethantencharacters");
    }

    #[test]
    fn it_deserializes() {
        let name = "morethantencharacters";
        let name: NameField = NameField::new(name).expect("should construct it");
        let deserialized: String = serde_json::to_string(&MyModel { name }).expect("should deserialize");
        let json = "{\"name\":\"morethantencharacters\"}";
        assert_eq!(deserialized, json);
    }

    #[test]
    fn it_errors_too_short() {
        let name = "small";
        let res: Result<LengthLimitedField<6, 100>, _> = LengthLimitedField::new(name);
        match res {
            Ok(_) => panic!("shouldn't work"),
            Err(LengthLimitedFieldError::TooLong{..}) => panic!("Wrong error"),
            _ => {}
        };
    }

    #[test]
    fn it_errors_too_long() {
        let name = "small";
        let res: Result<LengthLimitedField<1, 4>, _> = LengthLimitedField::new(name);
        match res {
            Ok(_) => panic!("shouldn't work"),
            Err(LengthLimitedFieldError::TooShort{..}) => panic!("Wrong error"),
            _ => {}
        };
    }
}
