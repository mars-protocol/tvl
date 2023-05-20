mod asset;
mod error;
mod price;
mod red_bank;

use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;

use crate::{
    error::Result,
    price::{print_prices, query_prices},
    red_bank::{print_red_bank_tvl, query_red_bank_tvl},
};

const OSMOSIS_GRPC: &str = "https://osmosis-grpc.polkachu.com:12590";

#[tokio::main]
async fn main() -> Result<()> {
    println!("querying prices from coingecko...");
    let prices = query_prices().await?;
    print_prices(&prices)?;

    let mut client = QueryClient::connect(OSMOSIS_GRPC).await?;

    println!("querying red bank tvl...");
    let red_bank_tvl = query_red_bank_tvl(&mut client).await?;
    print_red_bank_tvl(&red_bank_tvl, &prices)?;

    Ok(())
}
