use serde::{Deserialize, Serialize, Serializer};

use crate::codec::{RpcResponse, RpcSuccess, Tx, TxPointer};
use crate::define_ogmios_error;

// Acquire Mempool

#[derive(Debug, Clone, Deserialize)]
pub struct AcquireMempoolResult {
    /// Always set to "mempool"
    pub acquired: String,
    /// Slot number of the mempool snapshot
    pub slot: u64,
}

pub type AcquireMempoolResponse = RpcSuccess<AcquireMempoolResult>;

// Next Transaction

#[derive(Debug, Clone)]
pub struct NextTransaction {}

impl Serialize for NextTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("fields", "all")?;
        map.end()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MempoolTransaction {
    Tx(Tx),
    TxPointer(TxPointer),
}

#[derive(Debug, Clone, Deserialize)]
pub struct NextTransactionResult {
    pub transaction: Option<MempoolTransaction>,
}

define_ogmios_error! {
    #[derive(Debug, Clone)]
    pub enum MempoolError {
        4000 => MustAcquireMempoolFirst,
        _ => Unknown { error: Value }
    }
}

pub type NextTransactionResponse = RpcResponse<NextTransactionResult, MempoolError>;
