pub mod log;

use log::RawLogRecord;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to authenticate")]
    GoogleError(#[from] cloud_storage::Error),
    #[error("Failed to write file")]
    IOError(#[from] std::io::Error),
    #[error("Failed to convert {0:?} to LogRecord")]
    ConvertRawLogError(RawLogRecord),
    #[error("Failed to parse json")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Multiprocessing error")]
    MultiprocessingError(#[from] JoinError),
}
