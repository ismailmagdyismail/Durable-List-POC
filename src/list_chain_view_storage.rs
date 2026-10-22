use crate::sequential_access::{SequentialReader, SequentialWriter};
use std::path::PathBuf;

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

impl SequentialWriter for ListChainViewStorage {
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

impl SequentialReader for ListChainViewStorage {
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
