use thiserror::Error;

#[derive(Error, Debug)]
pub enum CheckableError {
    #[error("Missing field `{0}`")]
    MissingField(String),
    #[error("Chain `{0}` is not supported")]
    NoSuchChain(String),
    #[error("No address attached to user `{0}`")]
    MissingUserAddress(String),
    #[error("No address attached to requirement `id: {0}`")]
    MissingTokenAddress(String),
}
