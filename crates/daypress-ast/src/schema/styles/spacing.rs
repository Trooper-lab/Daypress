use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct Spacing {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<String>,
}

impl<'de> Deserialize<'de> for Spacing {
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
                Ok(Spacing {
                    top: get_opt("top"),
                    right: get_opt("right"),
                    bottom: get_opt("bottom"),
                    left: get_opt("left"),
                })
            }
            Value::String(s) => {
                let parts: Vec<&str> = s.split_whitespace().collect();
                match parts.as_slice() {
                    [a] => Ok(Spacing {
                        top: Some((*a).to_string()),
                        right: Some((*a).to_string()),
                        bottom: Some((*a).to_string()),
                        left: Some((*a).to_string()),
                    }),
                    [tb, rl] => Ok(Spacing {
                        top: Some((*tb).to_string()),
                        right: Some((*rl).to_string()),
                        bottom: Some((*tb).to_string()),
                        left: Some((*rl).to_string()),
                    }),
                    [t, rl, b] => Ok(Spacing {
                        top: Some((*t).to_string()),
                        right: Some((*rl).to_string()),
                        bottom: Some((*b).to_string()),
                        left: Some((*rl).to_string()),
                    }),
                    [t, r, b, l, ..] => Ok(Spacing {
                        top: Some((*t).to_string()),
                        right: Some((*r).to_string()),
                        bottom: Some((*b).to_string()),
                        left: Some((*l).to_string()),
                    }),
                    _ => Ok(Spacing::default()),
                }
            }
            _ => Ok(Spacing::default()),
        }
    }
}
