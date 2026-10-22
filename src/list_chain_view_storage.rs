use std::path::PathBuf;

use crate::storage_codec::{StorageDecoder, StorageEncoder};

pub const PREV_OFFSET: u8 = 8;
pub const NODE_OFFSET: u8 = 8;
pub const NEXT_OFFSET: u8 = 8;
pub const RECORD_SIZE: u8 = PREV_OFFSET + NODE_OFFSET + NEXT_OFFSET;
pub const LIST_CHAIN_FILE_NAME: &'static str = "list_chain.dat";

pub struct ListChainNode {
    pub prev_offset: u64,
    pub node_offset: u64,
    pub next_offset: u64,
}

#[derive(Debug)]
pub struct ListChainViewStorage {}

impl ListChainViewStorage {
    pub fn create_list_chain_file_path(storage_dir: &str) -> PathBuf {
        let mut file_path = PathBuf::from(storage_dir);
        file_path.push(LIST_CHAIN_FILE_NAME);
        file_path
    }
}

impl StorageEncoder for ListChainViewStorage {
    type ReaderType = ListChainViewStorage;
    type RecordType = ListChainNode;

    fn get_record_size(&self) -> u8 {
        RECORD_SIZE
    }

    fn new() -> Self::ReaderType {
        ListChainViewStorage {}
    }

    fn encode_record(&self, record: &Self::RecordType, slice: &mut [u8]) {
        let prev_offset_slice = &mut slice[0..PREV_OFFSET as usize];
        prev_offset_slice.copy_from_slice(&record.prev_offset.to_le_bytes());

        let node_offset_slice =
            &mut slice[PREV_OFFSET as usize..PREV_OFFSET as usize + NODE_OFFSET as usize];
        node_offset_slice.copy_from_slice(&record.node_offset.to_le_bytes());

        let next_offset_slice =
            &mut slice[PREV_OFFSET as usize + NODE_OFFSET as usize..RECORD_SIZE as usize];
        next_offset_slice.copy_from_slice(&record.next_offset.to_le_bytes());
    }
}

impl StorageDecoder for ListChainViewStorage {
    type ReaderType = ListChainViewStorage;
    type RecordType = ListChainNode;

    fn get_record_size(&self) -> u8 {
        RECORD_SIZE
    }

    fn new() -> Self::ReaderType {
        ListChainViewStorage {}
    }

    fn decode_record(
        &self,
        slice: &[u8],
    ) -> Result<Self::RecordType, crate::storage_errors::StorageError> {
        let prev_end = PREV_OFFSET as usize;
        let node_end = prev_end + NODE_OFFSET as usize;
        let next_end = RECORD_SIZE as usize;
        let prev_slice = slice.get(0..prev_end).ok_or_else(|| {
            crate::storage_errors::StorageError::ListChainCorruptedRecord(
                "Corrupted prev_offset".to_string(),
            )
        })?;
        let node_slice = slice.get(prev_end..node_end).ok_or_else(|| {
            crate::storage_errors::StorageError::ListChainCorruptedRecord(
                "Corrupted node_offset".to_string(),
            )
        })?;
        let next_slice = slice.get(node_end..next_end).ok_or_else(|| {
            crate::storage_errors::StorageError::ListChainCorruptedRecord(
                "Corrupted next_offset".to_string(),
            )
        })?;
        let prev_offset = u64::from_le_bytes(prev_slice.try_into().map_err(|_| {
            crate::storage_errors::StorageError::ListChainCorruptedRecord(
                "Corrupted prev_offset".to_string(),
            )
        })?);
        let node_offset = u64::from_le_bytes(node_slice.try_into().map_err(|_| {
            crate::storage_errors::StorageError::ListChainCorruptedRecord(
                "Corrupted node_offset".to_string(),
            )
        })?);
        let next_offset = u64::from_le_bytes(next_slice.try_into().map_err(|_| {
            crate::storage_errors::StorageError::ListChainCorruptedRecord(
                "Corrupted node_offset".to_string(),
            )
        })?);
        let chain_node = ListChainNode {
            prev_offset,
            node_offset,
            next_offset,
        };
        Ok(chain_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_errors::StorageError;

    #[test]
    fn creates_list_chain_path_in_storage_directory() {
        assert_eq!(
            ListChainViewStorage::create_list_chain_file_path("storage"),
            PathBuf::from("storage").join(LIST_CHAIN_FILE_NAME)
        );
    }

    #[test]
    fn reports_twenty_four_byte_records_for_reading_and_writing() {
        let writer = <ListChainViewStorage as StorageEncoder>::new();
        let reader = <ListChainViewStorage as StorageDecoder>::new();

        assert_eq!(StorageEncoder::get_record_size(&writer), RECORD_SIZE);
        assert_eq!(StorageDecoder::get_record_size(&reader), RECORD_SIZE);
    }

    #[test]
    fn encodes_and_decodes_node_offsets_as_little_endian() {
        let writer = <ListChainViewStorage as StorageEncoder>::new();
        let reader = <ListChainViewStorage as StorageDecoder>::new();
        let node = ListChainNode {
            prev_offset: 1,
            node_offset: u64::MAX,
            next_offset: 0x0102_0304_0506_0708,
        };
        let mut buffer = [0; RECORD_SIZE as usize];

        writer.encode_record(&node, &mut buffer);
        let decoded = reader.decode_record(&buffer).unwrap();

        assert_eq!(&buffer[0..8], &node.prev_offset.to_le_bytes());
        assert_eq!(&buffer[8..16], &node.node_offset.to_le_bytes());
        assert_eq!(&buffer[16..24], &node.next_offset.to_le_bytes());
        assert_eq!(decoded.prev_offset, node.prev_offset);
        assert_eq!(decoded.node_offset, node.node_offset);
        assert_eq!(decoded.next_offset, node.next_offset);
    }

    #[test]
    fn rejects_truncated_records() {
        let reader = <ListChainViewStorage as StorageDecoder>::new();

        for length in [0, 7, 8, 15, 16, 23] {
            let error = reader
                .decode_record(&vec![0; length])
                .err()
                .expect("truncated record should fail to decode");

            assert!(matches!(error, StorageError::ListChainCorruptedRecord(_)));
        }
    }
}
