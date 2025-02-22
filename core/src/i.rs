use std::result;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq)]
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

impl Serialize for I {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.result_string.serialize(serializer)
    }
}
