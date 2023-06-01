use std::collections::HashMap;

use cosmos_sdk_proto::cosmwasm::wasm::v1 as wasm;
use cosmwasm_std::{Coin, Empty, Uint128};
use cw_vault_standard::msg::VaultStandardQueryMsg;
use mars_red_bank_types::red_bank::{self, UserDebtResponse};
use mars_rover::{
    adapters::vault::VaultUnchecked,
    coins::Coins,
    msg::query::{QueryMsg as RoverQueryMsg, VaultInfoResponse},
};
use osmosis_proto::osmosis::gamm::v1beta1 as gamm;
use tonic::transport::Channel;

use crate::{
    asset::{asset_by_denom, Asset},
    error::{Error, Result},
    tvl::{TVLItem, TVL},
    utils::{query_osmosis_pool, query_wasm_smart, shift_decimals},
    RED_BANK, ROVER,
};

pub async fn query_rover_tvl(
    wasm_client: &mut wasm::query_client::QueryClient<Channel>,
    gamm_client: &mut gamm::query_client::QueryClient<Channel>,
) -> Result<TVL> {
    let mut tvl = HashMap::new();
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

    for coin in coins.into_vec() {
        match asset_by_denom(&coin.denom) {
            Some(asset) => {
                increase_deposit(&mut tvl, asset, coin.amount);
            },
            None => {
                // for other assets, we only support osmosis gamm tokens
                let pool_id = parse_gamm_denom(&coin.denom)?;
                let pool = query_osmosis_pool(gamm_client, pool_id).await?;

                for (asset, reserve) in pool.reserves {
                    let amount_raw = reserve.checked_multiply_ratio(coin.amount, pool.total_shares)?;
                    increase_deposit(&mut tvl, asset, amount_raw);
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
            let asset = asset_by_denom(&debt.denom).ok_or_else(|| Error::AssetNotFound {
                denom_or_id: debt.denom.clone(),
            })?;

            increase_borrow(&mut tvl, asset, debt.amount);
        }

        let Some(last) = debts_res.last() else {
            break;
        };

        start_after = Some(last.denom.clone());
    }

    Ok(tvl)
}

fn parse_gamm_denom(denom: &str) -> Result<u64> {
    let parts = denom.split('/').collect::<Vec<_>>();

    if parts.len() != 3 || parts[0] != "gamm" || parts[1] != "pool" {
        return Err(Error::NotGammToken {
            denom: denom.into(),
        });
    }

    parts[2].parse().map_err(Into::into)
}

fn increase_deposit(tvl: &mut TVL, asset: &'static Asset, amount_raw: Uint128) {
    let item = tvl.entry(asset).or_insert_with(TVLItem::default);
    item.deposited += shift_decimals(amount_raw, asset.decimals)
}

fn increase_borrow(tvl: &mut TVL, asset: &'static Asset, amount_raw: Uint128) {
    let item = tvl.entry(asset).or_insert_with(TVLItem::default);
    item.borrowed += shift_decimals(amount_raw, asset.decimals)
}
