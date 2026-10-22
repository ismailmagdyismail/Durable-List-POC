use crate::storage_errors::StorageError;

pub struct HeapRecordsCodec;

impl HeapRecordsCodec {
    pub const RECORD_SIZE: usize = 4;

    pub fn encode_record(record: i32) -> [u8; Self::RECORD_SIZE] {
        record.to_le_bytes()
    }

    pub fn decode_record(bytes: &[u8]) -> Result<i32, StorageError> {
        Ok(i32::from_le_bytes(bytes.try_into().map_err(|_| {
            StorageError::HeapCorruptedRecord("Couldn't convert record to i32".to_string())
        })?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_decodes_i32_values_as_little_endian() {
        for record in [0, 1, -1, i32::MIN, i32::MAX] {
            let buffer = HeapRecordsCodec::encode_record(record);

            assert_eq!(buffer, record.to_le_bytes());
            assert_eq!(HeapRecordsCodec::decode_record(&buffer).unwrap(), record);
        }
    }

    #[test]
    fn rejects_records_that_are_not_four_bytes() {
        let error = HeapRecordsCodec::decode_record(&[0; 3]).unwrap_err();

        assert!(matches!(error, StorageError::HeapCorruptedRecord(_)));
    }
}
