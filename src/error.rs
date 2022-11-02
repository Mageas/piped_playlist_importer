use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipedPlaylistImporterError {
    #[error("Unable to parse the filename for {0}")]
    FileName(std::path::PathBuf),

    #[error("Unable to list files for {1}: {0}")]
    ListFiles(#[source] std::io::Error, String),

    #[error("Unable to read lines from {1}: {0}")]
    ReadLines(#[source] std::io::Error, String),

    #[error("Unable to read : {0}")]
    ReadFile(#[from] std::io::Error),

    #[error("Unable to deserialize : {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error("Unable contact '{1}' : {0}")]
    ContactApi(#[source] reqwest::Error, String),

    #[error("{0}")]
    GeneralRequest(#[from] reqwest::Error),

    #[error("[{0}] Unable to contact '{1}' : {2}")]
    Request(reqwest::StatusCode, String, String),
}

pub type PipedPlaylistImporterResult<T = ()> = Result<T, PipedPlaylistImporterError>;
