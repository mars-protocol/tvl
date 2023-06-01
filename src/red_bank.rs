use std::collections::HashMap;

use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;
use mars_red_bank::interest_rates::{get_underlying_debt_amount, get_underlying_liquidity_amount};
use mars_red_bank_types::red_bank::{Market, QueryMsg};
use tonic::transport::Channel;

use crate::{
    asset::asset_by_denom,
    error::Result,
    tvl::{TVL, TVLItem},
    utils::{current_timestamp, query_wasm_smart, shift_decimals},
    RED_BANK,
};

pub async fn query_red_bank_tvl(client: &mut QueryClient<Channel>) -> Result<TVL> {
    let mut markets: Vec<Market> = vec![];
    let mut start_after: Option<String> = None;

    loop {
        let new_markets: Vec<Market> = query_wasm_smart(
            client,
            RED_BANK,
            &QueryMsg::Markets {
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

    let mut tvl: TVL = HashMap::new();
    let current_timestamp = current_timestamp();

    markets
        .into_iter()
        .try_for_each(|market| -> Result<_> {
            let Some(asset) = asset_by_denom(&market.denom) else {
                return Ok(());
            };

            let deposited_raw = get_underlying_liquidity_amount(
                market.collateral_total_scaled,
                &market,
                current_timestamp,
            )?;

            let borrowed_raw = get_underlying_debt_amount(
                market.debt_total_scaled,
                &market,
                current_timestamp,
            )?;

            let item = TVLItem {
                deposited: shift_decimals(deposited_raw, asset.decimals),
                borrowed: shift_decimals(borrowed_raw, asset.decimals),
            };

            tvl.insert(asset, item);

            Ok(())
        })?;

    Ok(tvl)
}
