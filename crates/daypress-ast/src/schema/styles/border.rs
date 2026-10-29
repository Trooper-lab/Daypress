use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct BorderConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

impl<'de> Deserialize<'de> for BorderConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Value::deserialize(deserializer)?;
        match val {
            Value::Object(map) => {
                let get_opt = |k: &str| -> Option<String> {
                    map.get(k)
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                };
                Ok(BorderConfig {
                    radius: get_opt("radius"),
                    width: get_opt("width"),
                    color: get_opt("color"),
                    style: get_opt("style"),
                })
            }
            Value::String(s) => {
                let parts: Vec<&str> = s.split_whitespace().collect();
                match parts.as_slice() {
                    [w, st, c, ..] => Ok(BorderConfig {
                        radius: None,
                        width: Some((*w).to_string()),
                        style: Some((*st).to_string()),
                        color: Some((*c).to_string()),
                    }),
                    [st] => Ok(BorderConfig {
                        radius: None,
                        width: None,
                        style: Some((*st).to_string()),
                        color: None,
                    }),
                    _ => Ok(BorderConfig::default()),
                }
            }
            _ => Ok(BorderConfig::default()),
        }
    }
}
