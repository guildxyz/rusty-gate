use thiserror::Error;

#[derive(Error, Debug)]
pub enum CheckableError {
    #[error("Missing field `{0}`")]
    MissingField(String),
}
