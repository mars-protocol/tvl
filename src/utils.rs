use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    asset::Asset,
    error::{Error, Result},
};

pub fn parse_gamm_denom(denom: &str) -> Result<u64> {
    let parts = denom.split('/').collect::<Vec<_>>();

    if parts.len() != 3 || parts[0] != "gamm" || parts[1] != "pool" {
        return Err(Error::NotGammToken {
            denom: denom.into(),
        });
    }

    parts[2].parse().map_err(Into::into)
}

pub fn increase_amount_raw(
    tvl: &mut HashMap<&'static Asset, f64>,
    asset: &'static Asset,
    amount_raw: u128,
) {
    *tvl.entry(asset).or_insert(0.) += shift_decimals(amount_raw, asset.decimals);
}

pub fn decrease_amount_raw(
    tvl: &mut HashMap<&'static Asset, f64>,
    asset: &'static Asset,
    amount_raw: u128,
) {
    *tvl.entry(asset).or_insert(0.) -= shift_decimals(amount_raw, asset.decimals);
}

pub fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards").as_secs()
}

pub fn shift_decimals(amount_raw: u128, decimals: u32) -> f64 {
    amount_raw as f64 / 10usize.pow(decimals) as f64
}
