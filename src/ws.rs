use std::fmt;

use anyhow::{Context, bail};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
pub use reqwest::Url;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::codec::{Id, RpcRequest, RpcResponseIdentifier};
use crate::method::mempool::{AcquireMempoolResult, NextTransactionResponse};

#[derive(Debug)]
pub struct OgmiosWsClient {
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    messages: Vec<(RpcResponseIdentifier, String)>,
}

impl OgmiosWsClient {
    pub async fn connect(url: Url) -> anyhow::Result<Self> {
        let (ws_stream, _) = connect_async(url.to_string()).await?;
        let (write, read) = ws_stream.split();
        Ok(Self {
            write,
            read,
            messages: vec![],
        })
    }

    pub async fn request<T: Serialize + fmt::Debug, U: DeserializeOwned>(
        &mut self,
        method: &str,
        params: Option<T>,
    ) -> anyhow::Result<U> {
        let id = self.send_request(method, params).await?;
        self.read_response(method, id).await
    }

    pub async fn send_request<T: Serialize + fmt::Debug>(
        &mut self,
        method: &str,
        params: Option<T>,
    ) -> anyhow::Result<Id> {
        let params = match params {
            Some(p) => serde_json::to_value(p)?,
            None => serde_json::Value::Object(serde_json::Map::new()),
        };
        let id = Id::default();
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params),
            id: Some(id.clone()),
        };

        let text = serde_json::to_string(&req)?;
        self.write.send(Message::Text(text.into())).await?;

        Ok(id)
    }

    pub async fn read_response<T: DeserializeOwned>(
        &mut self,
        method: &str,
        id: Id,
    ) -> anyhow::Result<T> {
        // Check buffered messages first
        let identifier = RpcResponseIdentifier {
            method: method.to_string(),
            id: Some(id),
        };
        if let Some(msg) = self
            .messages
            .extract_if(.., |msg| msg.0 == identifier)
            .next()
        {
            let res = serde_json::from_str(&msg.1).context("failed to deserialize")?;
            return Ok(res);
        }

        // Wait for new messages
        while let Some(msg) = self.read.next().await.transpose()? {
            match msg {
                Message::Text(text) => {
                    let new_identifier: RpcResponseIdentifier =
                        serde_json::from_str(&text).context("failed to deserialize")?;
                    if new_identifier == identifier {
                        let res: T =
                            serde_json::from_str(&text).context("failed to deserialize")?;
                        return Ok(res);
                    } else {
                        self.messages.push((new_identifier, text.to_string()));
                    }
                }
                _ => bail!("Unexpected message type received from ogmios: {:?}", msg),
            }
        }

        bail!("Connection closed")
    }

    pub async fn acquire_mempool(&mut self) -> anyhow::Result<AcquireMempoolResult> {
        self.request("acquireMempool", None::<()>).await
    }

    pub async fn next_mempool_tx(&mut self) -> anyhow::Result<NextTransactionResponse> {
        self.request("nextTransaction", None::<()>).await
    }
}
