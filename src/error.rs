#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    GRPCStatus(#[from] tonic::Status),

    #[error(transparent)]
    GRPCTransport(#[from] tonic::transport::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Std(#[from] cosmwasm_std::StdError),

    #[error("no asset found with denom or id `{denom_or_id}`")]
    AssetNotFound {
        denom_or_id: String,
    },

    #[error("no price found for asset `{symbol}`")]
    PriceNotFound {
        symbol: String,
    },
}

pub type Result<T> = core::result::Result<T, Error>;
