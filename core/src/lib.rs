use serde::{
    de, de::MapAccess, de::Visitor, ser::SerializeStruct, Deserialize, Deserializer, Serialize,
    Serializer,
};
use std::fmt;

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct I2(pub I2Content);

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum I2Content {
    A
}

impl I2 {
    pub fn new(value: String) -> Result<Self, String> {
        if value == "A" {
            Ok(I2(I2Content::A))
        } else {
            Err(value)
        }
    }
}

struct I2Visitor;

impl<'de> Visitor<'de> for I2Visitor {
    type Value = I2;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("I2")
    }
    fn visit_map<V>(self, mut map: V) -> Result<I2, V::Error>
    where
        V: MapAccess<'de>,
    {
        map.next_key::<String>()?;
        Ok(I2::new(map.next_value::<String>()?).map_err(de::Error::custom)?)
    }
}

impl<'de> Deserialize<'de> for I2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("I2", &["value"], I2Visitor)
    }
}

impl Serialize for I2 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("I2", 1)?;
        state.serialize_field("value", "A")?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    #[test]
    fn test_serde() {
        let i2 = I2(I2Content::A);
        assert_tokens(&i2, &[Token::Struct { name: "I2", len: 1 }, Token::Str("value"), Token::Str("A"), Token::StructEnd]);
    }

    #[test]
    fn test_serde_roundtrip() {
        let i2 = I2(I2Content::A);
        let serialized = serde_json::to_string(&i2).unwrap();
        assert_eq!(serialized, r#"{"value":"A"}"#);
        let deserialized: I2 = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, i2);
    }

    #[test]
    fn test_serde_error() {
        assert_de_tokens_error::<I2>(
            &[Token::Struct { name: "I2", len: 1 }, Token::Str("value"), Token::Str("B"), Token::StructEnd],
            "B",
        );
    }
}