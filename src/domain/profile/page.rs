use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{KeyConfig, SideConfig};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Page {
    pub name: String,
    #[serde(default, with = "key_map")]
    #[cfg_attr(
        feature = "specta",
        specta(type = std::collections::BTreeMap<String, KeyConfig>)
    )]
    pub keys: BTreeMap<u8, KeyConfig>,
    /// Bundle ids (for example `com.google.Chrome`). The page shows when one is in front.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apps: Vec<String>,
    /// Content for the side screen next to keys 5, 10 and 15. Absent means no side content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<SideConfig>,
}

/// TOML tables need string keys, so key numbers are written as `"1"`, `"14"`, etc.
/// Output order is the string order of the keys (`"1", "10", ...`). It is still deterministic.
mod key_map {
    use std::collections::BTreeMap;

    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::KeyConfig;

    pub fn serialize<S: Serializer>(
        map: &BTreeMap<u8, KeyConfig>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let text: BTreeMap<String, &KeyConfig> =
            map.iter().map(|(k, v)| (k.to_string(), v)).collect();
        text.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<u8, KeyConfig>, D::Error> {
        let text: BTreeMap<String, KeyConfig> = BTreeMap::deserialize(deserializer)?;
        text.into_iter()
            .map(|(k, v)| {
                k.parse::<u8>()
                    .map(|key| (key, v))
                    .map_err(D::Error::custom)
            })
            .collect()
    }
}

impl Page {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            keys: BTreeMap::new(),
            apps: Vec::new(),
            side: None,
        }
    }
}
