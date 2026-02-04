use serde::{Deserialize, Serialize};

use crate::codec::*;
use crate::define_ogmios_error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tip {
    Point {
        #[serde(rename = "slot")]
        slot: u64,
        #[serde(rename = "id")]
        id: String,
    },
    #[serde(rename = "origin")]
    Origin,
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
