use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    Open(String),
    Seek(String),
    Read(String),
    Write(String),
    Flush(String),
    Sync(String),
    InvalidRecordSize,
    TruncatedRecord { expected: usize, actual: usize },
    HeapCorruptedRecord(String),
    ListChainCorruptedRecord(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Open(message) => write!(formatter, "Open: {message}"),
            StorageError::Seek(message) => write!(formatter, "Seek: {message}"),
            StorageError::Read(message) => write!(formatter, "Read: {message}"),
            StorageError::Write(message) => write!(formatter, "Write: {message}"),
            StorageError::Flush(message) => write!(formatter, "Flush: {message}"),
            StorageError::Sync(message) => write!(formatter, "Sync: {message}"),
            StorageError::InvalidRecordSize => write!(formatter, "InvalidRecordSize"),
            StorageError::TruncatedRecord { expected, actual } => {
                write!(
                    formatter,
                    "TruncatedRecord: expected {expected} bytes, read {actual}"
                )
            }
            StorageError::HeapCorruptedRecord(message) => {
                write!(formatter, "HeapCorruptedRecord: {message}")
            }
            StorageError::ListChainCorruptedRecord(message) => {
                write!(formatter, "ListChainCorruptedRecord: {message}")
            }
        }
    }
}

impl std::error::Error for StorageError {}
