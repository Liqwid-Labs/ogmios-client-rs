use std::fmt;

use anyhow::Context;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
}; // Added futures_util imports
pub use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub mod codec;
pub mod evaluate;
pub mod pparams;
pub mod script;
pub mod submit;
pub mod utxo;

use codec::{RpcRequest, RpcResponse, TxCbor};
use evaluate::{EvaluateRequestParams, Evaluation, EvaluationError};
use pparams::{ProtocolParams, ProtocolParamsError};
use submit::{SubmitError, SubmitRequestParams, SubmitResult};

pub struct OgmiosClient {
    url: Url,
    client: reqwest::Client,
}

// TODO: handle reqwest error
impl OgmiosClient {
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
}

pub struct OgmiosWsClient {
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl OgmiosWsClient {
    pub async fn connect(url: Url) -> anyhow::Result<Self> {
        let (ws_stream, _) = connect_async(url.to_string()).await?;
        let (write, read) = ws_stream.split();
        Ok(Self { write, read })
    }

    pub async fn request<T: Serialize + fmt::Debug>(
        &mut self,
        method: &str,
        params: Option<T>,
    ) -> anyhow::Result<()> {
        let params = match params {
            Some(p) => serde_json::to_value(p)?,
            None => serde_json::Value::Object(serde_json::Map::new()),
        };
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params),
        };
        let text = serde_json::to_string(&req)?;
        self.write.send(Message::Text(text.into())).await?;
        Ok(())
    }

    pub async fn read_response<U: DeserializeOwned, E: DeserializeOwned>(
        &mut self,
    ) -> anyhow::Result<RpcResponse<U, E>> {
        while let Some(msg) = self.read.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let res: RpcResponse<U, E> = serde_json::from_str(&text).map_err(|e| {
                    tracing::error!("Failed to deserialize: {}. Raw message: {}", e, text);
                    e
                })?;
                return Ok(res);
            }
        }
        Err(anyhow::anyhow!("Connection closed"))
    }
}
