use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::codec::{AdaBalance, Era, RpcRequest, RpcResponse};
use crate::define_ogmios_error;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardAccountSummariesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Vec<String>>,
}

pub type RewardAccountSummariesRequest = RpcRequest<RewardAccountSummariesParams>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardAccountSummary {
    pub delegate: Option<Delegate>,
    pub rewards: AdaBalance,
    pub deposit: AdaBalance,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delegate {
    pub id: String,
    pub vrf: Option<String>,
}

define_ogmios_error! {
    #[derive(Debug, Clone)]
    pub enum RewardAccountSummariesError {
        2001 => EraMismatch {
            query_era: Era,
            ledger_era: Era,
        },
        2002 => UnavailableInCurrentEra,
        2003 => StateAcquiredExpired(String)
        _ => Unknown { error: Value }
    }
}

pub type RewardAccountSummariesResponse =
    RpcResponse<HashMap<String, RewardAccountSummary>, RewardAccountSummariesError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_account_summary_deserialization() {
        let json = r#"{
            "stake1u9x9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m": {
                "delegate": {
                    "id": "pool1z76v00n84p94uph9098g86a3ex5v3w3v3w3v3w3v3w3v3w3v3w3",
                    "vrf": "0000000000000000000000000000000000000000000000000000000000000000"
                },
                "rewards": { "ada": { "lovelace": 1000000 } },
                "deposit": { "ada": { "lovelace": 2000000 } }
            }
        }"#;

        let summaries: HashMap<String, RewardAccountSummary> =
            serde_json::from_str(json).expect("failed to deserialize");

        assert_eq!(summaries.len(), 1);
        let summary = summaries
            .get("stake1u9x9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m")
            .unwrap();
        assert_eq!(summary.rewards.lovelace, 1000000);
        assert_eq!(summary.deposit.lovelace, 2000000);
        assert!(summary.delegate.is_some());
        assert_eq!(
            summary.delegate.as_ref().unwrap().id,
            "pool1z76v00n84p94uph9098g86a3ex5v3w3v3w3v3w3v3w3v3w3v3w3"
        );
    }

    #[test]
    fn test_reward_account_summary_null_delegate() {
        let json = r#"{
            "stake1u9x9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m": {
                "delegate": null,
                "rewards": { "ada": { "lovelace": 1000000 } },
                "deposit": { "ada": { "lovelace": 2000000 } }
            }
        }"#;

        let summaries: HashMap<String, RewardAccountSummary> =
            serde_json::from_str(json).expect("failed to deserialize");

        let summary = summaries
            .get("stake1u9x9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m9m")
            .unwrap();
        assert!(summary.delegate.is_none());
    }

    #[test]
    fn test_rpc_response_actual_data_deserialization() {
        let json = r#"{"jsonrpc":"2.0","method":"queryLedgerState/rewardAccountSummaries","result":{"af71729c838c1f33529fbd5d72564468fb530febd289976b3733f448":{"delegate":{"id":"pool1prc9hna2mgamtspchrygc66s9n4tlkvh39e3t9zccef4kzc3ns2"},"rewards":{"ada":{"lovelace":7737851}},"deposit":{"ada":{"lovelace":2000000}}}},"id":null}"#;
        let response: RewardAccountSummariesResponse =
            serde_json::from_str(json).expect("Failed to deserialize RpcResponse from actual data");

        let result: Result<HashMap<String, RewardAccountSummary>, RewardAccountSummariesError> =
            response.into();
        assert!(result.is_ok());
        let summaries = result.unwrap();
        assert_eq!(summaries.len(), 1);
        let summary = summaries.get("af71729c838c1f33529fbd5d72564468fb530febd289976b3733f448").unwrap();
        assert_eq!(summary.rewards.lovelace, 7737851);
        assert_eq!(summary.deposit.lovelace, 2000000);
        assert!(summary.delegate.is_some());
        assert_eq!(summary.delegate.as_ref().unwrap().id, "pool1prc9hna2mgamtspchrygc66s9n4tlkvh39e3t9zccef4kzc3ns2");
    }

    #[test]
    fn test_params_serialization() {
        let params = RewardAccountSummariesParams {
            keys: Some(vec!["key1".to_string()]),
            scripts: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert_eq!(json, r#"{"keys":["key1"]}"#);

        let params = RewardAccountSummariesParams {
            keys: None,
            scripts: Some(vec!["script1".to_string()]),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert_eq!(json, r#"{"scripts":["script1"]}"#);
    }
}
