use thiserror::Error;

#[derive(Error, Debug)]
pub enum EternityError {
    #[error(transparent)]
    Wasmer(#[from] wasmer_runtime::error::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    FernError(#[from] fern::InitError),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    URLError(#[from] url::ParseError),
    #[error(transparent)]
    MatrixSDKError(#[from] matrix_sdk::Error),
    #[error(transparent)]
    RumaIdentifiersError(#[from] matrix_sdk::identifiers::Error),
    #[error(transparent)]
    LogError(#[from] log::SetLoggerError),
    #[error(transparent)]
    WasmerResolveError(#[from] wasmer_runtime::error::ResolveError),
    #[error("unknown Eternity error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, EternityError>;
