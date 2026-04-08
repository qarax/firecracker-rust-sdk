use http::Error as HttpError;
use hyper::Error as HyperError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to connect to Unix socket: {0}")]
    UnixSocketConnectError(#[from] IoError),
    #[error("hyper error: {0}")]
    HyperError(#[from] HyperError),
    #[error("HTTP client not initialized")]
    ClientNotInitialized,
    #[error("Firecracker API error, status: {0}, body: {1}")]
    FirecrackerApiError(u16, String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] HttpError),
    #[error("JSON (de)serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Unexpected error: {0}")]
    Other(String),
}
