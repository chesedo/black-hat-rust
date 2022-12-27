use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Usage: tricoder <kerkour.com>")]
    CliUsage,
    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("tokio join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error("{0}: Invalid HTTP response")]
    InvalidHttpResponse(String),
}
