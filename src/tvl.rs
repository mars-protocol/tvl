use std::collections::HashMap;
use serde::Serialize;

use crate::{asset::Asset, error::Result, price::Prices};

pub type TVL = HashMap<&'static Asset, TVLItem>;

#[derive(Default)]
pub struct TVLItem {
    pub deposited: f64,
    pub borrowed:  f64,
}

#[derive(Serialize)]
struct PrintableTVL {
    symbol:        &'static str,
    deposited:     f64,
    deposited_usd: f64,
    borrowed:      f64,
    borrowed_usd:  f64,
}

pub fn print_tvl(tvl: &TVL, prices: &Prices) -> Result<()> {
    let printable_tvl = tvl
        .iter()
        .filter_map(|(asset, item)| {
            let Some(price) = prices.get(asset) else {
                return None;
            };

            Some(PrintableTVL {
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
