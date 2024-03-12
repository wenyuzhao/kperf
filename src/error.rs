use thiserror::Error;

#[derive(Error, Debug)]
pub enum KPerfError {
    #[error("permission denied")]
    PermissionDenied,
    #[error("failed to initialize kperf")]
    InitError,
    #[error("failed to deinitialize kperf")]
    DeinitError,
    #[error("failed to enable kperf event")]
    InvalidEvent,
    #[error("failed to disable kperf")]
    DisableCountingFailed,
    #[error("failed to fetch counter values")]
    FetchCountersFailed,
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader { expected: String, found: String },
    #[error("unknown error")]
    Unknown,
}
