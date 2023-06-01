use serde::Serialize;

#[derive(Serialize, PartialEq, Eq, Hash)]
pub struct Asset {
    pub symbol:       &'static str,
    pub denom:        &'static str,
    pub coingecko_id: &'static str,
    pub decimals:     u32,
}

pub const ASSETS: &[Asset] = &[
    Asset {
        symbol:       "ATOM",
        denom:        "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2",
        coingecko_id: "cosmos",
        decimals:     6,
    },
    Asset {
        symbol:       "AXL",
        denom:        "ibc/903A61A498756EA560B85A85132D3AEE21B5DEDD41213725D22ABF276EA6945E",
        coingecko_id: "axelar",
        decimals:     6,
    },
    Asset {
        symbol:       "OSMO",
        denom:        "uosmo",
        coingecko_id: "osmosis",
        decimals:     6,
    },
    Asset {
        symbol:       "stATOM",
        denom:        "ibc/C140AFD542AE77BD7DCC83F13FDD8C5E5BB8C4929785E6EC2F4C636F98F17901",
        coingecko_id: "stride-staked-atom",
        decimals:     6,
    },
    Asset {
        symbol:       "USDC.axl",
        denom:        "ibc/D189335C6E4A68B513C10AB227BF1C1D38C746766278BA3EEB4FB14124F1D858",
        coingecko_id: "usd-coin",
        decimals:     6,
    },
    Asset {
        symbol:       "WBTC.axl",
        denom:        "ibc/D1542AA8762DB13087D8364F3EA6509FD6F009A34F00426AF9E4F9FA85CBBF1F",
        coingecko_id: "wrapped-bitcoin",
        decimals:     8,
    },
    Asset {
        symbol:       "WETH.axl",
        denom:        "ibc/EA1D43981D5C9A1C4AAEA9C23BB1D4FA126BA9BC7020A25E0AE4AA841EA25DC5",
        coingecko_id: "ethereum",
        decimals:     18,
    },
];

pub fn asset_by_denom(denom: &str) -> Option<&'static Asset> {
    ASSETS
        .iter()
        .find(|asset| asset.denom == denom)
}

pub fn asset_by_coingecko_id(coingecko_id: &str) -> Option<&'static Asset> {
    ASSETS
        .iter()
        .find(|asset| asset.coingecko_id == coingecko_id)
}
