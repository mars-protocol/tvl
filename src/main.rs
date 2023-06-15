mod asset;
mod error;
mod format;
mod prices;
mod tvl;
mod utils;

use std::collections::HashMap;

use cosmos_sdk_proto::cosmwasm::wasm::v1 as wasm;
use cosmwasm_std::{from_slice, to_vec, Coin, Empty, Uint128};
use cw_vault_standard::msg::VaultStandardQueryMsg;
use mars_red_bank::interest_rates::{get_underlying_debt_amount, get_underlying_liquidity_amount};
use mars_red_bank_types::red_bank::{self, Market, UserDebtResponse};
use mars_rover::{
    adapters::vault::VaultUnchecked,
    coins::Coins,
    msg::query::{QueryMsg as RoverQueryMsg, VaultInfoResponse},
};
use osmosis_proto::osmosis::gamm::v1beta1 as gamm;
use prost::Message;
use serde::{de::DeserializeOwned, ser::Serialize};
use tonic::transport::Channel;

use crate::{
    asset::{asset_by_denom, Asset},
    error::{Error, Result},
    prices::query_prices,
    tvl::{print_tvl, TVL},
    utils::{
        current_timestamp, decrease_amount, increase_amount, increase_amount_raw, parse_gamm_denom,
    },
};

const OSMOSIS_GRPC: &str = "http://backup1.larry.coffee:9090";

const RED_BANK: &str = "osmo1c3ljch9dfw5kf52nfwpxd2zmj2ese7agnx0p9tenkrryasrle5sqf3ftpg";

const ROVER: &str = "osmo1f2m24wktq0sw3c0lexlg7fv4kngwyttvzws3a3r3al9ld2s2pvds87jqvf";

#[tokio::main]
async fn main() -> Result<()> {
    println!("querying prices from coingecko...");
    let prices = query_prices().await?;
    println!("done!");

    println!("connecting to osmosis grpc...");
    let mut wasm_client = wasm::query_client::QueryClient::connect(OSMOSIS_GRPC).await?;
    let mut gamm_client = gamm::query_client::QueryClient::connect(OSMOSIS_GRPC).await?;
    println!("done!");

    println!("querying red bank markets...");
    let markets = query_red_bank_markets(&mut wasm_client).await?;
    println!("done!");

    println!("computing red bank tvl...");
    let red_bank_tvl = compute_red_bank_tvl(&markets)?;
    println!("done!");

    println!("computing rover tvl...");
    let rover_tvl = query_rover_tvl(&mut wasm_client, &mut gamm_client).await?;
    println!("done!");

    println!("computing total protocol tvl...");
    let protocol_tvl = compute_protocol_tvl(&red_bank_tvl, &rover_tvl);
    println!("done!");

    println!("------------------------------------ RED BANK ------------------------------------");
    print_tvl(&red_bank_tvl, &prices)?;

    println!("------------------------------------- ROVER --------------------------------------");
    print_tvl(&rover_tvl, &prices)?;

    println!("--------------------------------- TOTAL PROTOCOL ---------------------------------");
    print_tvl(&protocol_tvl, &prices)?;

    Ok(())
}

async fn query_red_bank_markets(
    wasm_client: &mut wasm::query_client::QueryClient<Channel>,
) -> Result<Vec<Market>> {
    let mut markets: Vec<Market> = vec![];
    let mut start_after: Option<String> = None;

    loop {
        let new_markets: Vec<Market> = query_wasm_smart(
            wasm_client,
            RED_BANK,
            &red_bank::QueryMsg::Markets {
                start_after: start_after.clone(),
                limit: Some(10), // the max limit
            },
        )
        .await?;

        let Some(last) = new_markets.last() else {
            break;
        };

        start_after = Some(last.denom.clone());
        markets.extend(new_markets);
    }

    Ok(markets)
}

fn compute_red_bank_tvl(markets: &[Market]) -> Result<TVL> {
    let mut tvl = TVL::default();
    let current_timestamp = current_timestamp();

    for market in markets {
        // red bank markets include gamm tokens, which aren't accepted for
        // deposit at red bank; they are just for storing risk params for
        // consumption by rover. here we skip them
        let Ok(asset) = asset_by_denom(&market.denom) else {
            continue;
        };

        let deposit_raw = get_underlying_liquidity_amount(
            market.collateral_total_scaled,
            &market,
            current_timestamp,
        )?;

        let borrow_raw = get_underlying_debt_amount(
            market.debt_total_scaled,
            &market,
            current_timestamp,
        )?;

        increase_amount_raw(&mut tvl.deposits, asset, deposit_raw.u128());
        increase_amount_raw(&mut tvl.borrows, asset, borrow_raw.u128());
    }

    Ok(tvl)
}

async fn query_rover_tvl(
    wasm_client: &mut wasm::query_client::QueryClient<Channel>,
    gamm_client: &mut gamm::query_client::QueryClient<Channel>,
) -> Result<TVL> {
    let mut tvl = TVL::default();
    // there isn't a Coins::new method so we have to initialize it like this
    let mut coins = Coins::try_from(vec![])?;
    let mut start_after: Option<VaultUnchecked> = None;

    loop {
        let vault_info_res: Vec<VaultInfoResponse> = query_wasm_smart(
            wasm_client,
            ROVER,
            &RoverQueryMsg::VaultsInfo {
                start_after: start_after.clone(),
                limit: None,
            },
        )
        .await?;

        for vault_info in &vault_info_res {
            // query the vault's base token
            let info: cw_vault_standard::VaultInfoResponse = query_wasm_smart(
                wasm_client,
                &vault_info.vault.address,
                &VaultStandardQueryMsg::<Empty>::Info {},
            )
            .await?;

            // query how many vault shares Rover holds
            let total_shares: Uint128 = query_wasm_smart(
                wasm_client,
                ROVER,
                &RoverQueryMsg::TotalVaultCoinBalance {
                    vault: vault_info.vault.clone(),
                },
            )
            .await?;

            // convert the vault shares to the underlying gamm token amount
            let amount: Uint128 = query_wasm_smart(
                wasm_client,
                &vault_info.vault.address,
                &VaultStandardQueryMsg::<Empty>::ConvertToAssets {
                    amount: total_shares,
                },
            )
            .await?;

            coins.add(&Coin {
                denom: info.base_token,
                amount,
            })?;
        }

        let Some(last) = vault_info_res.last() else {
            break;
        };

        start_after = Some(last.vault.clone());
    }

    // for each gamm token, convert it to underlying asset amounts and add to
    // Rover deposits
    for coin in coins.into_vec() {
        match asset_by_denom(&coin.denom) {
            Ok(asset) => {
                increase_amount_raw(&mut tvl.deposits, asset, coin.amount.u128());
            },
            Err(_) => {
                // for other assets, we only support osmosis gamm tokens
                let pool_id = parse_gamm_denom(&coin.denom)?;
                let pool = query_osmosis_pool(gamm_client, pool_id).await?;

                for (asset, reserve) in pool.reserves {
                    let amount_raw = reserve * coin.amount.u128() / pool.total_shares;
                    increase_amount_raw(&mut tvl.deposits, asset, amount_raw);
                }
            },
        }
    }

    // query debt owed by rover to red bank
    let mut start_after: Option<String> = None;

    loop {
        let debts_res: Vec<UserDebtResponse> = query_wasm_smart(
            wasm_client,
            RED_BANK,
            &red_bank::QueryMsg::UserDebts {
                user: ROVER.into(),
                start_after: start_after.clone(),
                limit: Some(10),
            },
        )
        .await?;

        for debt in &debts_res {
            let asset = asset_by_denom(&debt.denom)?;
            increase_amount_raw(&mut tvl.borrows, asset, debt.amount.u128());
        }

        let Some(last) = debts_res.last() else {
            break;
        };

        start_after = Some(last.denom.clone());
    }

    Ok(tvl)
}

fn compute_protocol_tvl(red_bank_tvl: &TVL, rover_tvl: &TVL) -> TVL {
    let mut protocol_tvl = red_bank_tvl.clone();

    for (asset, amount) in &rover_tvl.deposits {
        increase_amount(&mut protocol_tvl.deposits, asset, *amount);
    }

    for (asset, amount) in &rover_tvl.borrows {
        decrease_amount(&mut protocol_tvl.deposits, asset, *amount);
        decrease_amount(&mut protocol_tvl.borrows, asset, *amount);
    }

    protocol_tvl
}

pub struct PoolResponse {
    // denom => amount
    pub reserves: HashMap<&'static Asset, u128>,
    pub total_shares: u128,
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

            let asset = asset_by_denom(&token.denom)?;
            let amount = token.amount.parse()?;

            Ok((asset, amount))
        })
        .collect::<Result<_>>()?;

    let total_shares = pool
        .total_shares
        .ok_or_else(|| Error::TotalSharesUndefined {
            pool_id,
        })?
        .amount
        .parse()?;

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
