use std::collections::HashMap;

use crate::{
    asset::{asset_by_coingecko_id, Asset, ASSETS},
    error::{Error, Result},
};

const COINGECKO_ROOT_URL: &str = "https://api.coingecko.com/api/v3/simple/price";

const CURRENCY: &str = "usd";

pub type Prices = HashMap<&'static Asset, f64>;

pub async fn query_prices() -> Result<Prices> {
    let mut prices = HashMap::new();

    let ids = ASSETS
        .iter()
        .map(|asset| asset.coingecko_id)
        .collect::<Vec<_>>()
        .join(",");

    reqwest::get(format!("{COINGECKO_ROOT_URL}?ids={ids}&vs_currencies={CURRENCY}"))
        .await?
        .json::<HashMap<String, HashMap<String, f64>>>() // id => (currency => price)
        .await?
        .into_iter()
        .try_for_each(|(coingecko_id, prices_by_currency)| -> Result<_> {
            let asset = asset_by_coingecko_id(&coingecko_id)?;

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

pub fn price_of_asset(prices: &Prices, asset: &'static Asset) -> Result<f64> {
    prices.get(asset).copied().ok_or_else(|| Error::PriceNotFound {
        symbol: asset.symbol.into(),
    })
}
