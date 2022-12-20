use serde::de::Visitor;

pub(crate) struct SerdeVisitor;

impl<'de> Visitor<'de> for SerdeVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid UTF-8 string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}
