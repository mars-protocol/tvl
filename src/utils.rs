use std::time::{SystemTime, UNIX_EPOCH};

use cosmos_sdk_proto::cosmwasm::wasm::v1::{query_client::QueryClient, QuerySmartContractStateRequest};
use cosmwasm_std::{from_slice, to_vec, Uint128};
use serde::{de::DeserializeOwned, ser::Serialize};
use tonic::transport::Channel;

use crate::error::Result;

pub async fn query_wasm_smart<M, R>(client: &mut QueryClient<Channel>, contract: &str, msg: &M) -> Result<R>
where
    M: Serialize + ?Sized,
    R: DeserializeOwned,
{
    let res_bytes = client
        .smart_contract_state(QuerySmartContractStateRequest {
            address: contract.into(),
            query_data: to_vec(&msg)?,
        })
        .await?
        .into_inner()
        .data;

    from_slice(&res_bytes).map_err(Into::into)
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}

pub fn shift_decimals(amount_raw: Uint128, decimals: u32) -> f64 {
    amount_raw.u128() as f64 / 10usize.pow(decimals) as f64
}
