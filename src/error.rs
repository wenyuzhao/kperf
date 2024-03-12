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
    #[error("failed to fetch counter values")]
    CounterFetchError,
    #[error("unknown error")]
    Unknown,
}
