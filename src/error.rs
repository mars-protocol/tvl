#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    GRPCStatus(#[from] tonic::Status),

    #[error(transparent)]
    GRPCTransport(#[from] tonic::transport::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Std(#[from] cosmwasm_std::StdError),

    #[error("no asset found with denom or id `{denom_or_id}`")]
    AssetNotFound {
        denom_or_id: String,
    },

    #[error("denom is not an osmosis gamm token: `{denom}`")]
    NotGammToken {
        denom: String,
    },

    #[error("osmosis pool not found with id `{pool_id}`")]
    PoolNotFound {
        pool_id: u64,
    },

    #[error("no price found for asset `{symbol}`")]
    PriceNotFound {
        symbol: String,
    },

    #[error("token for pool `{pool_id}` is undefined")]
    TokenUndefined {
        pool_id: u64,
    },

    #[error("total shares for pool `{pool_id}` is undefined")]
    TotalSharesUndefined {
        pool_id: u64,
    },
}

pub type Result<T> = core::result::Result<T, Error>;
