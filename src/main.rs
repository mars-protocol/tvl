mod asset;
mod error;
mod price;
mod red_bank;
mod rover;
mod tvl;
mod utils;

use cosmos_sdk_proto::cosmwasm::wasm::v1 as wasm;
use osmosis_proto::osmosis::gamm::v1beta1 as gamm;

use crate::{
    error::Result,
    price::{print_prices, query_prices},
    red_bank::query_red_bank_tvl,
    rover::query_rover_tvl,
    tvl::print_tvl,
};

const OSMOSIS_GRPC: &str = "http://backup.larry.coffee:9090";

#[tokio::main]
async fn main() -> Result<()> {
    println!("querying prices from coingecko...");
    let prices = query_prices().await?;
    print_prices(&prices)?;

    println!("connecting to osmosis grpc...");
    let mut wasm_client = wasm::query_client::QueryClient::connect(OSMOSIS_GRPC).await?;
    let mut gamm_client = gamm::query_client::QueryClient::connect(OSMOSIS_GRPC).await?;
    println!("done!");

    println!("querying red bank tvl...");
    let red_bank_tvl = query_red_bank_tvl(&mut wasm_client).await?;
    print_tvl(&red_bank_tvl, &prices)?;

    println!("querying rover tvl...");
    let rover_tvl = query_rover_tvl(&mut wasm_client, &mut gamm_client).await?;
    print_tvl(&rover_tvl, &prices)?;

    Ok(())
}
