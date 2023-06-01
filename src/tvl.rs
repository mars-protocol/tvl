use std::collections::HashMap;

use prettytable::{row, Table};

use crate::{
    asset::Asset,
    error::Result,
    format,
    prices::{price_of_asset, Prices},
};

#[derive(Default, Clone)]
pub struct TVL {
    pub deposits: HashMap<&'static Asset, f64>,
    pub borrows: HashMap<&'static Asset, f64>,
}

struct Row {
    pub symbol: &'static str,
    pub amount: f64,
    pub value: f64,
}

pub fn print_tvl(tvl: &TVL, prices: &Prices) -> Result<()> {
    let mut deposits = tvl
        .deposits
        .iter()
        .map(|(asset, amount)| {
            Ok(Row {
                symbol: &asset.symbol,
                amount: *amount,
                value: amount * price_of_asset(prices, asset)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut borrows = tvl
        .borrows
        .iter()
        .map(|(asset, amount)| {
            Ok(Row {
                symbol: &asset.symbol,
                amount: *amount,
                value: amount * price_of_asset(prices, asset)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    sort_rows_by_value(&mut deposits);
    sort_rows_by_value(&mut borrows);

    println!("DEPOSITS:");
    print_rows(&deposits);

    println!("BORROWS:");
    print_rows(&borrows);

    Ok(())
}

fn sort_rows_by_value(rows: &mut [Row]) {
    // f64 doesn't implement Ord, so we can't use sort_by_key
    // this unwrap panics if both f64 values are NaN
    // https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by
    rows.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
}

fn print_rows(rows: &[Row]) {
    let total_value = rows.iter().fold(0., |curr, acc| curr + acc.value);

    let mut table = Table::new();

    for row in rows {
        if row.amount > 0. {
            table.add_row(row![
                row.symbol,
                r->format::amount(row.amount),
                r->format::value(row.value),
            ]);
        }
    }

    table.add_row(row![
        "Total",
        "",
        r->format::value(total_value),
    ]);

    table.set_titles(row!["Token", "Amount", "Value ($)"]);
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    table.printstd();
}
