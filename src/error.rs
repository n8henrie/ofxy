#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error(transparent)]
    Sgmlish(#[from] sgmlish::Error),

    #[error(transparent)]
    Normalization(#[from] sgmlish::transforms::NormalizationError),

    #[error(transparent)]
    SgmlishDe(#[from] sgmlish::de::DeserializationError),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}
