use std::collections::HashMap;
use std::fmt;

use anyhow::Context;
pub use reqwest::Url;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::codec::{RpcRequest, RpcResponse, TxCbor};
use crate::method::evaluate::{EvaluateRequestParams, Evaluation, EvaluationError};
use crate::method::pparams::{ProtocolParams, ProtocolParamsError};
use crate::method::rewards::{
    RewardAccountSummariesError, RewardAccountSummariesParams, RewardAccountSummary,
};
use crate::method::submit::{SubmitError, SubmitRequestParams, SubmitResult};
use crate::method::tip::{Tip, TipError};

pub struct OgmiosHttpClient {
    url: Url,
    client: reqwest::Client,
}

// TODO: handle reqwest error
impl OgmiosHttpClient {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    async fn request<
        T: Serialize + Clone + fmt::Debug,
        U: DeserializeOwned,
        E: DeserializeOwned,
    >(
        &self,
        method: &str,
        params: Option<T>,
    ) -> anyhow::Result<RpcResponse<U, E>> {
        let res = self
            .client
            .post(self.url.clone())
            .json(&RpcRequest {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params: params.clone(),
                id: None,
            })
            .send()
            .await
            .with_context(|| format!("Failed to send request for method '{}'", method))?;

        let status = res.status();
        let response_text = res
            .text()
            .await
            .with_context(|| format!("Failed to read response body for method '{}'", method))?;

        serde_json::from_str(&response_text).with_context(|| {
            format!(
                "Failed to deserialize JSON response for method '{}'\n- Response status: {}\n- Response body:\n{}\n- Request body:\n{}",
                method,
                status,
                response_text,
                serde_json::to_string_pretty(&RpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: method.to_string(),
                    params,
                    id: None,
                })
                .unwrap()
            )
        })
    }

    pub async fn evaluate(&self, tx_cbor: &[u8]) -> Result<Vec<Evaluation>, EvaluationError> {
        let params = EvaluateRequestParams {
            transaction: TxCbor {
                cbor: hex::encode(tx_cbor),
            },
        };
        self.request("evaluateTransaction", Some(params))
            .await
            .unwrap()
            .into()
    }

    pub async fn submit(&self, tx_cbor: &[u8]) -> Result<SubmitResult, SubmitError> {
        let params = SubmitRequestParams {
            transaction: TxCbor {
                cbor: hex::encode(tx_cbor),
            },
        };
        self.request("submitTransaction", Some(params))
            .await
            .unwrap()
            .into()
    }

    pub async fn protocol_params(&self) -> Result<ProtocolParams, ProtocolParamsError> {
        self.request("queryLedgerState/protocolParameters", None::<()>)
            .await
            .expect("failed to get protocol parameters")
            .into()
    }

    pub async fn query_tip(&self) -> Result<Tip, TipError> {
        self.request("queryLedgerState/tip", None::<()>)
            .await
            .expect("failed to get tip")
            .into()
    }

    pub async fn reward_account_summaries(
        &self,
        keys: Option<Vec<String>>,
        scripts: Option<Vec<String>>,
    ) -> Result<HashMap<String, RewardAccountSummary>, RewardAccountSummariesError> {
        let params = RewardAccountSummariesParams { keys, scripts };
        self.request("queryLedgerState/rewardAccountSummaries", Some(params))
            .await
            .unwrap()
            .into()
    }
}
