use serde::{
    de::MapAccess, de::Visitor, ser::SerializeStruct, Deserialize, Deserializer, Serialize,
    Serializer,
};
use std::fmt;

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct I2();

struct I2Visitor;

impl<'de> Visitor<'de> for I2Visitor {
    type Value = I2;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("I2")
    }
    fn visit_map<V>(self, mut _map: V) -> Result<I2, V::Error>
    where
        V: MapAccess<'de>,
    {
        Ok(I2())
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
        serializer.serialize_struct("I2", 0)?.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn test_serde() {
        let i2 = I2();
        assert_tokens(
            &i2,
            &[Token::Struct { name: "I2", len: 0 }, Token::StructEnd],
        );
    }

    #[test]
    fn test_serde_roundtrip() {
        let i2 = I2();
        let serialized = serde_json::to_string(&i2).unwrap();
        assert_eq!(serialized, r#"{}"#);
        let deserialized: I2 = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, i2);
    }

    #[test]
    fn test_serde_roundtrip_from_reader() {
        let i2 = I2();
        let serialized = serde_json::to_string(&i2).unwrap();
        let reader = std::io::Cursor::new(serialized);
        let deserialized: I2 = serde_json::from_reader(reader).unwrap();
        assert_eq!(deserialized, i2);
    }
}
