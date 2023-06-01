use std::collections::HashMap;

use serde::Serialize;

use crate::{
    asset::{asset_by_coingecko_id, Asset, ASSETS},
    error::{Error, Result},
};

const CURRENCY: &str = "usd";

pub type Prices = HashMap<&'static Asset, f64>;

pub async fn query_prices() -> Result<Prices> {
    let mut prices = HashMap::new();

    let ids = ASSETS
        .iter()
        .map(|asset| asset.coingecko_id)
        .collect::<Vec<_>>()
        .join(",");

    reqwest::get(format!("https://api.coingecko.com/api/v3/simple/price?ids={ids}&vs_currencies={CURRENCY}"))
        .await?
        .json::<HashMap<String, HashMap<String, f64>>>() // id => (currency => price)
        .await?
        .into_iter()
        .try_for_each(|(coingecko_id, prices_by_currency)| -> Result<_> {
            let asset = asset_by_coingecko_id(&coingecko_id)
                .ok_or_else(|| Error::AssetNotFound {
                    denom_or_id: coingecko_id,
                })?;

            let price = prices_by_currency
                .get(CURRENCY)
                .ok_or_else(|| Error::PriceNotFound {
                    symbol: asset.symbol.into(),
                })?;

            prices.insert(asset, *price);

            Ok(())
        })?;

    Ok(prices)
}

#[derive(Serialize)]
struct PrintablePrice {
    symbol: &'static str,
    price:  f64,
}

pub fn print_prices(prices: &Prices) -> Result<()> {
    let printable_prices = prices
        .iter()
        .map(|(asset, price)| PrintablePrice {
            symbol: asset.symbol,
            price:  *price,
        })
        .collect::<Vec<PrintablePrice>>();

    let prices_str = serde_json::to_string_pretty(&printable_prices)?;

    println!("{prices_str}");

    Ok(())
}
