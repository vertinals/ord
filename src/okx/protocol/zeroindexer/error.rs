
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum JSONError {
    #[error("invalid content type")]
    InvalidContentType,

    #[error("unsupport content type")]
    UnSupportContentType,

    #[error("invalid json string")]
    InvalidJson,

    #[error("not zeroindexer json")]
    NotZeroIndexerJson,

}