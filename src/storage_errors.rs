#[derive(Debug)]
pub enum StorageError {
    HeadDataFile(String),
    HeapAppend(String),
    HeapSeek(String),
    HeapFlush(String),
    HeapFSync(String),
    HeapRead(String),
    HeapCorruptedRecord(String),
    ListChainCorruptedRecord(String),
}

impl StorageError {
    pub fn to_string(&self) -> String {
        match self {
            StorageError::HeadDataFile(msg) => format!("HeadDataFile: {}", msg),
            StorageError::HeapAppend(msg) => format!("HeapAppend: {}", msg),
            StorageError::HeapSeek(msg) => format!("HeapSeek: {}", msg),
            StorageError::HeapFlush(msg) => format!("HeapFlush: {}", msg),
            StorageError::HeapFSync(msg) => format!("HeapFSync: {}", msg),
            StorageError::HeapRead(msg) => format!("HeapRead: {}", msg),
            StorageError::HeapCorruptedRecord(msg) => {
                format!("HeapCorruptedRecord: {}", msg)
            },
            StorageError::ListChainCorruptedRecord(msg) => {
                format!("ListChainCorruptedRecord: {}", msg)
            }
        }
    }
}
