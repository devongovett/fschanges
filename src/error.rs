use std::fmt;

#[derive(Debug)]
pub struct ParcelWatcherError {
    pub reason: String,
}

impl std::error::Error for ParcelWatcherError {}

impl fmt::Display for ParcelWatcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl From<notify::Error> for ParcelWatcherError {
    fn from(e: notify::Error) -> Self {
        return ParcelWatcherError {
            reason: format!("{:?}", e),
        };
    }
}

impl From<ParcelWatcherError> for napi::Error {
    fn from(e: ParcelWatcherError) -> Self {
        return napi::Error {
            status: napi::Status::GenericFailure,
            reason: e.reason,
        };
    }
}
