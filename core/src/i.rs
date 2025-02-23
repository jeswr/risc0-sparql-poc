use std::result;

use serde::{
    de, de::MapAccess, de::Visitor, ser::SerializeStruct, Deserialize, Deserializer, Serialize,
    Serializer,
};
use std::io::Write;
use std::{fmt, str};
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct I {
    pub result_string: String,
}

impl I {
    pub fn new(result_string: String) -> Self {
        Self { result_string }
    }
}

impl<'de> Deserialize<'de> for I {
    fn deserialize<D>(deserializer: D) -> result::Result<I, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result_string = String::deserialize(deserializer)?;
        Ok(I { result_string })
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct I2(I2Content);

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
enum I2Content {
    A,
    B
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
        formatter.write_str("struct I2")
    }
    fn visit_map<V>(self, mut map: V) -> Result<I2, V::Error>
    where
        V: MapAccess<'de>,
    {
        let key = map.next_key::<String>()?;
        if key != Some("value".to_string()) {
            if let Some(val) = key {
                return Err(de::Error::unknown_field(&val, &["value"]));
            }
            return Err(de::Error::missing_field("value"));
        }
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
