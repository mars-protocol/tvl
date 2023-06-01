use std::{
    collections::HashMap,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use cosmos_sdk_proto::cosmwasm::wasm::v1 as wasm;
use cosmwasm_std::{from_slice, to_vec, Uint128};
use osmosis_proto::osmosis::gamm::v1beta1 as gamm;
use prost::Message;
use serde::{de::DeserializeOwned, ser::Serialize};
use tonic::transport::Channel;

use crate::{
    asset::{asset_by_denom, Asset},
    error::{Error, Result},
};

pub struct PoolResponse {
    // denom => amount
    pub reserves: HashMap<&'static Asset, Uint128>,
    pub total_shares: Uint128,
}

pub async fn query_osmosis_pool(
    client: &mut gamm::query_client::QueryClient<Channel>,
    pool_id: u64,
) -> Result<PoolResponse> {
    // NOTE: this query will be deprecated in v16. use poolmanager module instead
    let pool_any = client
        .pool(gamm::QueryPoolRequest {
            pool_id,
        })
        .await?
        .into_inner()
        .pool
        .ok_or(Error::PoolNotFound {
            pool_id,
        })?;

    let pool = gamm::Pool::decode(pool_any.value.as_slice())?;

    let reserves = pool
        .pool_assets
        .iter()
        .map(|pool_asset| {
            let token = pool_asset.token.as_ref().ok_or_else(|| Error::TokenUndefined {
                pool_id,
            })?;

            let asset = asset_by_denom(&token.denom).ok_or_else(|| Error::AssetNotFound {
                denom_or_id: token.denom.clone(),
            })?;

            let amount = Uint128::from_str(&token.amount)?;

            Ok((asset, amount))
        })
        .collect::<Result<_>>()?;

    let total_shares_coin = pool.total_shares.ok_or_else(|| Error::TotalSharesUndefined {
        pool_id,
    })?;

    let total_shares = Uint128::from_str(&total_shares_coin.amount)?;

    Ok(PoolResponse {
        reserves,
        total_shares,
    })
}

pub async fn query_wasm_smart<M, R>(
    client: &mut wasm::query_client::QueryClient<Channel>,
    contract: &str,
    msg: &M,
) -> Result<R>
where
    M: Serialize + ?Sized,
    R: DeserializeOwned,
{
    let res_bytes = client
        .smart_contract_state(wasm::QuerySmartContractStateRequest {
            address: contract.into(),
            query_data: to_vec(&msg)?,
        })
        .await?
        .into_inner()
        .data;

    from_slice(&res_bytes).map_err(Into::into)
}

pub fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards").as_secs()
}

pub fn shift_decimals(amount_raw: Uint128, decimals: u32) -> f64 {
    amount_raw.u128() as f64 / 10usize.pow(decimals) as f64
}
