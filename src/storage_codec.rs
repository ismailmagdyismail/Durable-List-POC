use crate::storage_errors::StorageError;

pub trait StorageEncoder {
    type ReaderType;
    type RecordType;

    fn new() -> Self::ReaderType;
    fn get_record_size(&self) -> u8;
    fn encode_record(&self, record: &Self::RecordType, slice: &mut [u8]);
}

pub trait StorageDecoder {
    type ReaderType;
    type RecordType;

    fn new() -> Self::ReaderType;
    fn get_record_size(&self) -> u8;
    fn decode_record(&self, slice: &[u8]) -> Result<Self::RecordType, StorageError>;
}
