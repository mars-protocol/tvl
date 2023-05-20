use std::collections::HashMap;

use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;
use mars_red_bank::interest_rates::{get_underlying_debt_amount, get_underlying_liquidity_amount};
use mars_red_bank_types::red_bank::{Market, QueryMsg};
use serde::Serialize;
use tonic::transport::Channel;

use crate::{
    asset::{Asset, ASSETS},
    error::Result,
    price::Prices,
    utils::{current_timestamp, query_wasm_smart, shift_decimals},
};

const RED_BANK: &str = "osmo1c3ljch9dfw5kf52nfwpxd2zmj2ese7agnx0p9tenkrryasrle5sqf3ftpg";

pub type RedBankTVL = HashMap<&'static Asset, RedBankTVLItem>;

pub struct RedBankTVLItem {
    pub deposited: f64,
    pub borrowed:  f64,
}

pub async fn query_red_bank_tvl(client: &mut QueryClient<Channel>) -> Result<RedBankTVL> {
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

    let mut tvl: RedBankTVL = HashMap::new();
    let current_timestamp = current_timestamp();

    markets
        .into_iter()
        .try_for_each(|market| -> Result<_> {
            let Some(asset) = ASSETS.iter().find(|asset| asset.denom == market.denom) else {
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

            let item = RedBankTVLItem {
                deposited: shift_decimals(deposited_raw, asset.decimals),
                borrowed: shift_decimals(borrowed_raw, asset.decimals),
            };

            tvl.insert(asset, item);

            Ok(())
        })?;

    Ok(tvl)
}

#[derive(Serialize)]
struct PrintableRedBankTVL {
    symbol:        &'static str,
    deposited:     f64,
    deposited_usd: f64,
    borrowed:      f64,
    borrowed_usd:  f64,
}

pub fn print_red_bank_tvl(tvl: &RedBankTVL, prices: &Prices) -> Result<()> {
    let printable_tvl = tvl
        .iter()
        .filter_map(|(asset, item)| {
            let Some(price) = prices.get(asset) else {
                return None;
            };

            Some(PrintableRedBankTVL {
                symbol:        asset.symbol,
                deposited:     item.deposited,
                deposited_usd: item.deposited * price,
                borrowed:      item.borrowed,
                borrowed_usd:  item.borrowed * price,
            })
        })
        .collect::<Vec<_>>();

    let tvl_str = serde_json::to_string_pretty(&printable_tvl)?;
    println!("{tvl_str}");

    let total_deposited = printable_tvl.iter().fold(0f64, |acc, curr| acc + curr.deposited_usd);
    println!("total deposited: {total_deposited}");

    let total_borrowed = printable_tvl.iter().fold(0f64, |acc, curr| acc + curr.borrowed_usd);
    println!("total borrowed: {total_borrowed}");

    Ok(())
}
