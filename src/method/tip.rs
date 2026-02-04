use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::codec::*;
use crate::define_ogmios_error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tip {
    Point { slot: u64, id: String },
    Origin,
}

impl Serialize for Tip {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Tip::Point { slot, id } => {
                #[derive(Serialize)]
                struct Point<'a> {
                    slot: u64,
                    id: &'a str,
                }
                Point { slot: *slot, id }.serialize(serializer)
            }
            Tip::Origin => serializer.serialize_str("origin"),
        }
    }
}

impl<'de> Deserialize<'de> for Tip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Point {
            slot: u64,
            id: String,
        }

        let value = serde_json::Value::deserialize(deserializer)?;

        match &value {
            serde_json::Value::String(s) if s == "origin" => Ok(Tip::Origin),
            serde_json::Value::Object(_) => {
                let point: Point =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Tip::Point {
                    slot: point.slot,
                    id: point.id,
                })
            }
            _ => Err(serde::de::Error::custom(
                "expected \"origin\" or object with slot and id",
            )),
        }
    }
}

impl PartialOrd for Tip {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Tip::Point { slot: a, .. }, Tip::Point { slot: b, .. }) => a.partial_cmp(b),
            (Tip::Origin, Tip::Origin) => Some(std::cmp::Ordering::Equal),
            _ => None,
        }
    }
}

define_ogmios_error! {
    #[derive(Debug, Clone)]
    pub enum TipError {
        2001 => EraMismatch {
            query_era: Era,
            ledger_era: Era,
        },
        2002 => UnavailableInCurrentEra,
        2003 => StateAcquiredExpired(String)
        _ => Unknown { error: Value }
    }
}

pub type TipResponse = RpcResponse<Tip, TipError>;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn deserialize_point() {
        let json = json!({
            "slot": 1234,
            "id": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        });
        let tip: Tip = serde_json::from_value(json).unwrap();
        assert_eq!(
            tip,
            Tip::Point {
                slot: 1234,
                id: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            }
        );
    }

    #[test]
    fn serialize_point() {
        let tip = Tip::Point {
            slot: 1234,
            id: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        };
        let json = serde_json::to_value(tip).unwrap();
        assert_eq!(
            json,
            json!({
                "slot": 1234,
                "id": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            })
        );
    }

    #[test]
    fn deserialize_origin() {
        let json = json!("origin");
        let tip: Tip = serde_json::from_value(json).unwrap();
        assert_eq!(tip, Tip::Origin);
    }

    #[test]
    fn serialize_origin() {
        let tip = Tip::Origin;
        let json = serde_json::to_value(tip).unwrap();
        assert_eq!(json, json!("origin"));
    }
}
