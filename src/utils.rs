use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::asset::Asset;

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}

pub fn shift_decimals(amount_raw: u128, decimals: u32) -> f64 {
    amount_raw as f64 / 10usize.pow(decimals) as f64
}

pub fn increase_amount(
    tvl: &mut HashMap<&'static Asset, f64>,
    asset: &'static Asset,
    amount: f64,
) {
    *tvl.entry(asset).or_insert(0.) += amount;
}

pub fn decrease_amount(
    tvl: &mut HashMap<&'static Asset, f64>,
    asset: &'static Asset,
    amount: f64,
) {
    *tvl.entry(asset).or_insert(0.) -= amount;
}

pub fn increase_amount_raw(
    tvl: &mut HashMap<&'static Asset, f64>,
    asset: &'static Asset,
    amount_raw: u128,
) {
    increase_amount(tvl, asset, shift_decimals(amount_raw, asset.decimals));
}
