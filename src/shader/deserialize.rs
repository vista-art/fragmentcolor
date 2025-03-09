use std::fmt;

use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::ShaderObject;

impl<'de> Deserialize<'de> for ShaderObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Source, // we only need the source to rebuild the Struct
        }

        struct ShaderVisitor;

        impl<'de> Visitor<'de> for ShaderVisitor {
            type Value = ShaderObject;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Shader")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut source: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Source => {
                            if source.is_some() {
                                return Err(de::Error::duplicate_field("source"));
                            }
                            source = Some(map.next_value()?);
                        }
                    }
                }
                let source = source.ok_or_else(|| de::Error::missing_field("source"))?;
                ShaderObject::new(&source).map_err(de::Error::custom)
            }
        }

        const FIELDS: &[&str] = &["source"];
        deserializer.deserialize_struct("Shader", FIELDS, ShaderVisitor)
    }
}
